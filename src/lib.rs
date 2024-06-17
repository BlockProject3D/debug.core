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

// The reason why this is needed is because the 3 examples of usage of the Logger struct requires
// some context to not make it confusing.
#![allow(clippy::needless_doctest_main)]
#![warn(missing_docs)]

//! An async flexible logger framework designed for BP3D software.

mod easy_termcolor;
mod internal;
mod log_msg;
pub mod util;
mod builder;
mod level;
pub mod handler;

use bp3d_os::dirs::App;
use crossbeam_channel::Receiver;
use std::path::PathBuf;

pub use log_msg::{LogMsg, Location};
pub use builder::*;
pub use internal::Logger;
pub use level::Level;

/// The log buffer type.
pub type LogBuffer = Receiver<LogMsg>;

/// Trait to allow getting a log directory from either a bp3d_os::dirs::App or a String.
pub trait GetLogs {
    /// Gets the log directory as a PathBuf.
    ///
    /// Returns None if no directory could be computed.
    fn get_logs(self) -> Option<PathBuf>;
}

impl<'a> GetLogs for &'a String {
    fn get_logs(self) -> Option<PathBuf> {
        self.as_str().get_logs()
    }
}

impl<'a, 'b> GetLogs for &'a App<'b> {
    fn get_logs(self) -> Option<PathBuf> {
        self.get_logs().map(|v| v.create())?.ok().map(|v| v.into())
    }
}

impl<'a> GetLogs for &'a str {
    fn get_logs(self) -> Option<PathBuf> {
        let app = App::new(self);
        app.get_logs().map(|v| v.create())?.ok().map(|v| v.into())
    }
}

/// Represents a logger guard.
///
/// WARNING: Once this guard is dropped messages are no longer captured.
pub struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
    }
}

/// Runs a closure in scope of a logger configuration, then free the given logger configuration
/// and return closure result.
pub fn with_logger<R, F: FnOnce() -> R>(logger: Builder, f: F) -> R {
    let _guard = logger.start();
    f()
}
