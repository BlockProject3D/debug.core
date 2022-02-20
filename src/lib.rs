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

mod backend;
mod internal;

use bp3d_fs::dirs::App;
use crossbeam_channel::Receiver;
use log::{Level, Log};

#[derive(Clone)]
pub struct LogMsg {
    msg: String,
    target: String,
    level: Level
}

pub type LogBuffer = Receiver<LogMsg>;

pub trait ToApp<'a>
{
    fn to_app(self) -> App<'a>;
}

impl<'a> ToApp<'a> for App<'a>
{
    fn to_app(self) -> App<'a> {
        self
    }
}

impl<'a> ToApp<'a> for &'a str
{
    fn to_app(self) -> App<'a> {
        App::new(self)
    }
}

#[derive(Default)]
pub struct Logger
{
    std: Option<backend::StdBackend>,
    file: Option<backend::FileBackend>
}

impl Logger
{
    pub fn new() -> Logger {
        Logger::default()
    }

    pub fn add_stdout(mut self) -> Self
    {
        self.std = Some(backend::StdBackend::new(true));
        self
    }

    pub fn add_file<'a, T: ToApp<'a>>(mut self, app: T) -> Self
    {
        let app = app.to_app();
        if let Ok(logs) = app.get_logs() {
            self.file = Some(backend::FileBackend::new(logs));
        } else {
            eprintln!("Failed to obtain application log directory");
        }
        self
    }

    pub fn build(self) -> LoggerGuard {
        let _ = log::set_logger(&*BP3D_LOGGER); // Ignore the error
        // (we can't do anything if there's already a logger set;
        // unfortunately that is a limitation of the log crate)

        BP3D_LOGGER.start_new_thread(self); // Re-start the logging thread with the new configuration.
        LoggerGuard {}
    }
}

#[must_use]
pub struct LoggerGuard {}

impl LoggerGuard {
    pub fn start(&self) {
        BP3D_LOGGER.enable(true);
    }
}

impl Drop for LoggerGuard {
    fn drop(&mut self) {
        // Disable the logger so further log requests are dropped.
        BP3D_LOGGER.enable(false);
        // Send termination command and join with logging thread.
        BP3D_LOGGER.terminate();
        // Clear by force all content of in memory log buffer.
        BP3D_LOGGER.clear_log_buffer();
    }
}

lazy_static::lazy_static! {
    static ref BP3D_LOGGER: internal::LoggerImpl = internal::LoggerImpl::new();
}

pub fn enable_log_buffer()
{
    BP3D_LOGGER.enable_log_buffer(true);
}

pub fn disable_log_buffer()
{
    BP3D_LOGGER.enable_log_buffer(false);
    BP3D_LOGGER.clear_log_buffer();
}

pub fn get_log_buffer() -> LogBuffer
{
    BP3D_LOGGER.get_log_buffer()
}
