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

use crate::GetLogs;
use crate::handler::{FileHandler, Handler, StdHandler};
use crate::internal::Logger;
use crate::level::LevelFilter;

/// Enum of the different color settings when printing to stdout/stderr.
#[derive(Debug, Copy, Clone)]
pub enum Colors {
    /// Color printing is always enabled.
    Enabled,

    /// Color printing is always disabled.
    Disabled,

    /// Color printing is automatic (if current terminal is a tty, print with colors, otherwise
    /// print without colors).
    Auto,
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
/// use bp3d_logger::{Builder, Level, LogMsg, Location};
///
/// fn main() {
///     let logger = Builder::new().add_stdout().add_file("my-app").start();
///     logger.log(&LogMsg::from_msg(Location::new("bp3d-logger", "test.c", 1), Level::Info, "Example message"));
/// }
/// ```
///
/// The following example shows initialization of this logger and use of the log buffer.
/// ```
/// use bp3d_logger::{Builder, Level, LogMsg, Location, LevelFilter, handler::{LogQueue, LogQueueHandler}};
///
/// fn main() {
///     let queue = LogQueue::default();
///     let logger = Builder::new().add_stdout().add_handler(LogQueueHandler::new(queue.clone())).start();
///
///     //... application code with log redirect pump.///
///     logger.log(&LogMsg::from_msg(Location::new("bp3d-logger", "test.c", 1), Level::Info, "Example message"));
///     logger.set_filter(LevelFilter::None);
///     logger.log(&LogMsg::from_msg(Location::new("bp3d-logger", "test.c", 1), Level::Info, "Dropped message"));
///     logger.raw_log(&LogMsg::from_msg(Location::new("bp3d-logger", "test.c", 1), Level::Info, "Example message 1"));
///     logger.set_filter(LevelFilter::Info);
///
///     logger.flush();
///     let l = queue.pop().unwrap(); // Capture the last log message.
///     // We can't test for equality because log messages contains a timestamp...
///     assert_eq!(l.msg(), "Example message");
///     let l = queue.pop().unwrap();
///     assert_eq!(l.msg(), "Example message 1");
///     //... application code without log redirect pump.
/// }
/// ```
pub struct Builder {
    pub(crate) colors: Colors,
    pub(crate) smart_stderr: bool,
    pub(crate) buf_size: Option<usize>,
    pub(crate) handlers: Vec<Box<dyn Handler>>,
    pub(crate) filter: LevelFilter
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            colors: Colors::default(),
            smart_stderr: true,
            buf_size: None,
            handlers: Vec::new(),
            filter: LevelFilter::Info
        }
    }
}

impl Builder {
    /// Creates a new instance of a logger builder.
    pub fn new() -> Builder {
        Builder::default()
    }

    /// Sets the colors state when logging to stdout/stderr.
    ///
    /// The default behavior is to disable colors.
    pub fn colors(mut self, state: Colors) -> Self {
        self.colors = state;
        self
    }

    /// Sets the default level filter when initializing the logger.
    ///
    /// The default is [Info](LevelFilter::Info).
    pub fn filter(mut self, filter: LevelFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Enables or disables automatic redirection of error logs to stderr.
    ///
    /// The default for this flag is true.
    pub fn smart_stderr(mut self, flag: bool) -> Self {
        self.smart_stderr = flag;
        self
    }

    /// Sets the buffer size.
    ///
    /// # Arguments
    ///
    /// * `buf_size`: the buffer size.
    ///
    /// returns: Builder
    pub fn buffer_size(mut self, buf_size: usize) -> Self {
        self.buf_size = Some(buf_size);
        self
    }

    /// Adds a new log [Handler](Handler).
    ///
    /// # Arguments
    ///
    /// * `handler`: the new handler implementation to add.
    ///
    /// returns: Builder
    pub fn add_handler<T: Handler + 'static>(mut self, handler: T) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }

    /// Enables stdout logging.
    pub fn add_stdout(self) -> Self {
        let motherfuckingrust = self.smart_stderr;
        let motherfuckingrust1 = self.colors;
        self.add_handler(StdHandler::new(motherfuckingrust, motherfuckingrust1))
    }

    /// Enables file logging to the given application.
    ///
    /// The application is given as a reference to [GetLogs](GetLogs) to allow obtaining
    /// a log directory from various sources.
    ///
    /// If the log directory could not be found the function prints an error to stderr.
    pub fn add_file<T: GetLogs>(self, app: T) -> Self {
        if let Some(logs) = app.get_logs() {
            self.add_handler(FileHandler::new(logs))
        } else {
            eprintln!("Failed to obtain application log directory");
            self
        }
    }

    /// Initializes the log implementation with this current configuration.
    ///
    /// NOTE: This returns an instance of [Logger](Logger) which is the main entry point for all
    /// logging based operations. This instance also acts as a guard to flush all log buffers
    /// before returning. It is necessary to flush log buffers because this implementation
    /// uses threads to avoid blocking the main thread when issuing logs.
    ///
    /// NOTE 2: There are no safety concerns with running twice this function in the same
    /// application, only that calling this function may be slow due to thread management.
    pub fn start(self) -> Logger {
        Logger::new(self)
    }
}
