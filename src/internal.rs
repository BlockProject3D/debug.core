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

use crate::{LogMsg, Builder};
use crossbeam_channel::{bounded, Receiver, Sender};
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicU8, Ordering};
use crate::handler::{Flag, Handler};
use crate::level::LevelFilter;

const BUF_SIZE: usize = 16; // The maximum count of log messages in the channel.

//Disable large_enum_variant as using a Box will inevitably cause a small allocation on a critical path,
//allocating in a critical code path will most likely result in degraded performance.
//And yes, logging is a critical path when using bp3d-tracing.
#[allow(clippy::large_enum_variant)]
enum Command {
    Flush,
    Log(LogMsg),
    Terminate
}

struct Thread {
    handlers: Vec<Box<dyn Handler>>,
    recv_ch: Receiver<Command>,
    enable_stdout: Flag
}

impl Thread {
    pub fn new(
        handlers: Vec<Box<dyn Handler>>,
        recv_ch: Receiver<Command>,
        enable_stdout: Flag
    ) -> Thread {
        Thread {
            handlers,
            recv_ch,
            enable_stdout
        }
    }

    fn exec_commad(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::Terminate => true,
            Command::Flush => {
                for v in &mut self.handlers {
                    v.flush();
                }
                false
            }
            Command::Log(buffer) => {
                for v in &mut self.handlers {
                    v.write(&buffer);
                }
                false
            }
        }
    }

    pub fn run(mut self) {
        for v in &mut self.handlers {
            v.install(&self.enable_stdout);
        }
        while let Ok(v) = self.recv_ch.recv() {
            let flag = self.exec_commad(v);
            if flag {
                // The thread has requested to exit itself; drop out of the main loop.
                break;
            }
        }
    }
}

/// The main Logger type allows to control the entire logger state and submit messages for logging.
pub struct Logger {
    send_ch: Sender<Command>,
    level: AtomicU8,
    enable_stdout: Flag,
    thread: ManuallyDrop<std::thread::JoinHandle<()>>,
}

impl Logger {
    pub(crate) fn new(builder: Builder) -> Logger {
        let buf_size = builder.buf_size.unwrap_or(BUF_SIZE);
        let (send_ch, recv_ch) = bounded(buf_size);
        let recv_ch1 = recv_ch.clone();
        let enable_stdout = Flag::new(true);
        let enable_stdout1 = enable_stdout.clone();
        let thread = std::thread::spawn(move || {
            let thread = Thread::new(builder.handlers, recv_ch1, enable_stdout1);
            thread.run();
        });
        Logger {
            thread: ManuallyDrop::new(thread),
            send_ch,
            level: AtomicU8::new(builder.filter as u8),
            enable_stdout
        }
    }

    /// Enables the stdout/stderr logger.
    ///
    /// # Arguments
    ///
    /// * `flag`: true to enable stdout, false to disable stdout.
    pub fn enable_stdout(&self, flag: bool) {
        self.enable_stdout.set(flag);
    }

    /// Low-level log function. This injects log messages directly into the logging thread channel.
    ///
    /// This function applies basic formatting depending on the backend:
    /// - For stdout/stderr backend the format is \<target\> \[level\] msg.
    /// - For file backend the format is \[level\] msg and the message is recorded in the file
    /// corresponding to the log target.
    ///
    /// WARNING: For optimization reasons, this function does not check and thus does neither honor
    /// the enabled flag nor the current log level. For a checked log function,
    /// use [checked_log](Self::log).
    #[inline]
    pub fn raw_log(&self, msg: &LogMsg) {
        unsafe {
            // This cannot panic as send_ch is owned by LoggerImpl which is intended
            // to be statically allocated.
            self.send_ch
                .send(Command::Log(msg.clone()))
                .unwrap_unchecked();
        }
    }

    /// Main log function. This injects log messages into the logging thread channel only if
    /// this logger is enabled.
    ///
    /// This function calls the [raw_log](Self::raw_log) function only when this logger is enabled.
    #[inline]
    pub fn log(&self, msg: &LogMsg) {
        if self.filter() >= msg.level().as_level_filter() {
            self.raw_log(msg);
        }
    }

    /// Returns the filter level of this logger instance.
    pub fn filter(&self) -> LevelFilter {
        unsafe { LevelFilter::from_u8(self.level.load(Ordering::Acquire)).unwrap_unchecked() }
    }

    /// Sets the new level filter for this logger.
    ///
    /// # Arguments
    ///
    /// * `filter`: the new [LevelFilter](LevelFilter).
    pub fn set_filter(&self, filter: LevelFilter) {
        self.level.store(filter as u8, Ordering::Release);
    }

    /// Returns true if the logger is currently enabled and is capturing log messages.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.filter() > LevelFilter::None
    }

    /// Flushes all pending messages.
    pub fn flush(&self) {
        if !self.is_enabled() {
            return;
        }
        unsafe {
            // This cannot panic as send_ch is owned by LoggerImpl which is intended
            // to be statically allocated.
            self.send_ch.send(Command::Flush).unwrap_unchecked();
            while !self.send_ch.is_empty() {}
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        // Disable this Logger.
        self.set_filter(LevelFilter::None);

        // Send termination command and join with logging thread.
        // This cannot panic as send_ch is owned by LoggerImpl which is intended
        // to be statically allocated.
        unsafe {
            self.send_ch.send(Command::Flush).unwrap_unchecked();
            self.send_ch.send(Command::Terminate).unwrap_unchecked();
        }

        // Join the logging thread; this will lock until the thread is completely terminated.
        let thread = unsafe { ManuallyDrop::into_inner(std::ptr::read(&self.thread)) };
        thread.join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::Builder;

    fn ensure_send_sync<T: Send + Sync>(_: T) {}

    #[test]
    fn basic_test() {
        let logger = Builder::new().start();
        ensure_send_sync(logger);
    }
}
