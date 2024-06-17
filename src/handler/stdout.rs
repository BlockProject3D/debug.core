// Copyright (c) 2024, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use crate::easy_termcolor::{color, EasyTermColor};
use crate::handler::{Flag, Handler};
use crate::util::write_time;
use crate::{Colors, Level, Location, LogMsg};
use bp3d_os::time::LocalUtcOffset;
use bp3d_util::format::FixedBufStr;
use std::io::IsTerminal;
use std::mem::MaybeUninit;
use termcolor::{ColorChoice, ColorSpec, StandardStream};
use time::{OffsetDateTime, UtcOffset};

/// A simple stdout/stderr handler which redirects error messages to stderr and other messages to
/// stdout.
pub struct StdHandler {
    smart_stderr: bool,
    colors: Colors,
    enable: MaybeUninit<Flag>,
}

fn format_time_str(time: &OffsetDateTime) -> FixedBufStr<128> {
    let offset = UtcOffset::local_offset_at(*time);
    let time = offset.map(|v| time.to_offset(v)).unwrap_or(*time);
    let mut time_str = FixedBufStr::<128>::new();
    write_time(&mut time_str, time);
    time_str
}

fn write_msg(
    stream: StandardStream,
    location: &Location,
    time: &OffsetDateTime,
    msg: &str,
    level: Level,
) {
    let (target, module) = location.get_target_module();
    let t = ColorSpec::new().set_bold(true).clone();
    let time_str = format_time_str(time);
    EasyTermColor(stream)
        .write('<')
        .color(t)
        .write(target)
        .reset()
        .write("> ")
        .write('[')
        .color(color(level))
        .write(level)
        .reset()
        .write("] ")
        .write(time_str.str())
        .write(" ")
        .write(module)
        .write(": ")
        .write(msg)
        .lf();
}

enum Stream {
    Stdout,
    Stderr,
}

impl Stream {
    pub fn isatty(&self) -> bool {
        match self {
            Stream::Stdout => std::io::stdout().is_terminal(),
            Stream::Stderr => std::io::stderr().is_terminal(),
        }
    }
}

impl StdHandler {
    /// Creates a new [StdHandler](StdHandler).
    ///
    /// # Arguments
    ///
    /// * `smart_stderr`: true to enable redirecting error logs to stderr, false otherwise.
    /// * `colors`: the printing color policy.
    ///
    /// returns: StdHandler
    pub fn new(smart_stderr: bool, colors: Colors) -> StdHandler {
        StdHandler {
            smart_stderr,
            colors,
            enable: MaybeUninit::uninit(),
        }
    }

    fn get_stream(&self, level: Level) -> Stream {
        match self.smart_stderr {
            false => Stream::Stdout,
            true => match level {
                Level::Error => Stream::Stderr,
                _ => Stream::Stdout,
            },
        }
    }
}

impl Handler for StdHandler {
    fn install(&mut self, enable_stdout: &Flag) {
        self.enable.write(enable_stdout.clone());
    }

    fn write(&mut self, msg: &LogMsg) {
        if !unsafe { self.enable.assume_init_ref().is_enabled() } {
            // Skip logging if temporarily disabled.
            return;
        }
        let stream = self.get_stream(msg.level());
        let use_termcolor = match self.colors {
            Colors::Disabled => false,
            Colors::Enabled => true,
            Colors::Auto => stream.isatty(),
        };
        match use_termcolor {
            true => {
                let val = match stream {
                    Stream::Stderr => StandardStream::stderr(ColorChoice::Always),
                    _ => StandardStream::stdout(ColorChoice::Always),
                };
                write_msg(val, msg.location(), msg.time(), msg.msg(), msg.level());
            }
            false => {
                let (target, module) = msg.location().get_target_module();
                let time_str = format_time_str(msg.time());
                match stream {
                    Stream::Stderr => eprintln!(
                        "<{}> [{}] {} {}: {}",
                        target,
                        msg.level(),
                        time_str.str(),
                        module,
                        msg.msg()
                    ),
                    _ => println!(
                        "<{}> [{}] {} {}: {}",
                        target,
                        msg.level(),
                        time_str.str(),
                        module,
                        msg.msg()
                    ),
                };
            }
        };
    }

    fn flush(&mut self) {}
}
