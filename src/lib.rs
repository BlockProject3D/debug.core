// Copyright (c) 2021, BlockProject 3D
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

mod backend;
mod internal;

use bp3d_fs::dirs::App;
use crossbeam_channel::Receiver;
use log::{Level, Log};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use crate::backend::ENABLE_STDOUT;

/// Represents a log message in the [LogBuffer](crate::LogBuffer).
#[derive(Clone)]
pub struct LogMsg {
    /// The message string.
    pub msg: String,

    /// The crate name that issued this log.
    pub target: String,

    /// The log level.
    pub level: Level,
}

/// The log buffer type.
pub type LogBuffer = Receiver<LogMsg>;

/// Trait to allow getting a log directory from either a bp3d_fs::dirs::App or a String.
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
        self.get_logs().map(|v| v.into()).ok()
    }
}

impl<'a> GetLogs for &'a str {
    fn get_logs(self) -> Option<PathBuf> {
        let app = App::new(self);
        app.get_logs().map(|v| v.into()).ok()
    }
}

/// Enum of the different color settings when printing to stdout/stderr.
#[derive(Debug, Copy, Clone)]
pub enum Colors {
    /// Color printing is always enabled.
    Enabled,

    /// Color printing is always disabled.
    Disabled,

    /// Color printing is automatic (if current terminal is a tty, print with colors, otherwise
    /// print without colors).
    Auto
}

impl Default for Colors {
    fn default() -> Self {
        Self::Disabled
    }
}

/// The base logger builder/initializer.
///
/// # Examples
///
/// The following example shows basic initialization of this logger.
/// ```
/// use bp3d_logger::Logger;
/// use log::info;
/// use log::LevelFilter;
///
/// fn main() {
///     let _guard = Logger::new().add_stdout().add_file("my-app").start();
///     log::set_max_level(LevelFilter::Info);
///     //...
///     info!("Example message");
/// }
/// ```
///
/// The following example shows initialization of this logger with a return value.
/// ```
/// use bp3d_logger::Logger;
/// use bp3d_logger::with_logger;
/// use log::info;
/// use log::LevelFilter;
///
/// fn main() {
///     let code = with_logger(Logger::new().add_stdout().add_file("my-app"), {
///         log::set_max_level(LevelFilter::Info);
///         //...
///         info!("Example message");
///         0
///     });
///     std::process::exit(code);
/// }
/// ```
///
/// The following example shows initialization of this logger and use of the log buffer.
/// ```
/// use bp3d_logger::Logger;
/// use log::info;
/// use log::LevelFilter;
///
/// fn main() {
///     let _guard = Logger::new().add_stdout().add_file("my-app").start();
///     log::set_max_level(LevelFilter::Info);
///     bp3d_logger::enable_log_buffer(); // Enable log redirect pump into application channel.
///     //... application code with log redirect pump.
///     info!("Example message");
///     let l = bp3d_logger::get_log_buffer().recv().unwrap();// Capture the last log message.
///     println!("Last log message: {}", l.msg);
///     bp3d_logger::disable_log_buffer();
///     //... application code without log redirect pump.
/// }
/// ```
pub struct Logger {
    colors: Colors,
    smart_stderr: bool,
    std: Option<backend::StdBackend>,
    file: Option<backend::FileBackend>,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            colors: Colors::default(),
            smart_stderr: true,
            std: None,
            file: None
        }
    }
}

impl Logger {
    /// Creates a new instance of a logger builder.
    pub fn new() -> Logger {
        Logger::default()
    }

    /// Sets the colors state when logging to stdout/stderr.
    ///
    /// The default behavior is to disable colors.
    pub fn colors(mut self, state: Colors) -> Self {
        self.colors = state;
        self
    }

    /// Enables or disables automatic redirection of error logs to stderr.
    ///
    /// The default for this flag is true.
    pub fn smart_stderr(mut self, flag: bool) -> Self {
        self.smart_stderr = flag;
        self
    }

    /// Enables stdout logging.
    pub fn add_stdout(mut self) -> Self {
        self.std = Some(backend::StdBackend::new(self.smart_stderr, self.colors));
        self
    }

    /// Enables file logging to the given application.
    ///
    /// The application is given as a reference to [GetLogs](crate::GetLogs) to allow obtaining
    /// a log directory from various sources.
    ///
    /// If the log directory could not be found the function prints an error to stderr.
    pub fn add_file<T: GetLogs>(mut self, app: T) -> Self {
        if let Some(logs) = app.get_logs() {
            self.file = Some(backend::FileBackend::new(logs));
        } else {
            eprintln!("Failed to obtain application log directory");
        }
        self
    }

    /// Initializes the log implementation with this current configuration.
    ///
    /// NOTE: This returns a guard to flush all log buffers before returning. It is
    /// necessary to flush log buffers because this implementation uses threads
    /// to avoid blocking the main thread when issuing logs.
    ///
    /// NOTE 2: There are no safety concerns with running twice this function in the same
    /// application, only that calling this function may be slow due to thread management.
    pub fn start(self) -> Guard {
        let _ = log::set_logger(&*BP3D_LOGGER); // Ignore the error
        // (we can't do anything if there's already a logger set;
        // unfortunately that is a limitation of the log crate)

        BP3D_LOGGER.start_new_thread(self); // Re-start the logging thread with the new configuration.
        BP3D_LOGGER.enable(true); // Enable logging.
        Guard
    }

    /// Initializes the log implementation with this current configuration.
    ///
    /// NOTE: Since version 1.1.0 this is a redirect to bp3d_logger::with_logger.
    #[deprecated(since = "1.1.0", note = "please use bp3d_logger::with_logger")]
    pub fn run<R, F: FnOnce() -> R>(self, f: F) -> R {
        with_logger(self, f)
    }
}

/// Represents a logger guard.
///
/// WARNING: Once this guard is dropped messages are no longer captured.
pub struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        // Disable the logger so further log requests are dropped.
        BP3D_LOGGER.enable(false);
        // Send termination command and join with logging thread.
        BP3D_LOGGER.terminate();
        // Disable log buffer.
        BP3D_LOGGER.enable_log_buffer(false);
        // Clear by force all content of in memory log buffer.
        BP3D_LOGGER.clear_log_buffer();
    }
}

static BP3D_LOGGER: Lazy<internal::LoggerImpl> = Lazy::new(internal::LoggerImpl::new);

/// Enables the log redirect pump.
pub fn enable_log_buffer() {
    BP3D_LOGGER.enable_log_buffer(true);
}

/// Disables the log redirect pump.
pub fn disable_log_buffer() {
    BP3D_LOGGER.enable_log_buffer(false);
    BP3D_LOGGER.clear_log_buffer();
}

/// Enables the stdout/stderr logger.
pub fn enable_stdout() {
    ENABLE_STDOUT.store(true, Ordering::Release);
}

/// Disables the stdout/stderr logger.
pub fn disable_stdout() {
    ENABLE_STDOUT.store(false, Ordering::Release);
}

/// Returns the buffer from the log redirect pump.
pub fn get_log_buffer() -> LogBuffer {
    BP3D_LOGGER.get_log_buffer()
}

/// Low-level log function. This injects log messages directly into the logging thread channel.
///
/// This function applies basic formatting depending on the backend:
/// - For stdout/stderr backend the format is <target> \[level\] msg
/// - For file backend the format is \[level\] msg and the message is recorded in the file
/// corresponding to the log target.
pub fn raw_log(msg: LogMsg) {
    BP3D_LOGGER.low_level_log(msg)
}

/// Shortcut to the flush command to avoid having to call behind the dyn interface.
pub fn flush() {
    BP3D_LOGGER.flush();
}

/// Returns true if the logger is currently enabled and is capturing log messages.
pub fn enabled() -> bool {
    BP3D_LOGGER.is_enabled()
}

/// Runs a closure in scope of a logger configuration, then free the given logger configuration
/// and return closure result.
pub fn with_logger<R, F: FnOnce() -> R>(logger: Logger, f: F) -> R {
    let _guard = logger.start();
    f()
}
