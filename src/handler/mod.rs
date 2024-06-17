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

//! The log handler system, with default provided handlers.

mod file;
mod log_queue;
mod stdout;

use crate::LogMsg;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A dynamic atomic flag.
#[derive(Clone)]
pub struct Flag(Arc<AtomicBool>);

impl Flag {
    /// Creates a new flag.
    ///
    /// # Arguments
    ///
    /// * `initial`: the initial value of this flag.
    ///
    /// returns: Flag
    pub fn new(initial: bool) -> Self {
        Self(Arc::new(AtomicBool::new(initial)))
    }

    /// Returns true if this flag is ON, false otherwise.
    pub fn is_enabled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    /// Sets this flag.
    pub fn set(&self, flag: bool) {
        self.0.store(flag, Ordering::Release);
    }
}

/// The main handler trait.
pub trait Handler: Send {
    /// Called when the handler is installed in the async logging thread.
    ///
    /// # Arguments
    ///
    /// * `enable_stdout`: boolean flag to know if printing to stdout is allowed.
    fn install(&mut self, enable_stdout: &Flag);

    /// Called when a message is being written.
    ///
    /// # Arguments
    ///
    /// * `msg`: the log message which was emitted as a [LogMsg](LogMsg).
    fn write(&mut self, msg: &LogMsg);

    /// Called when the flush command is received in the async logging thread.
    fn flush(&mut self);
}

pub use file::FileHandler;
pub use log_queue::{LogQueue, LogQueueHandler};
pub use stdout::StdHandler;
