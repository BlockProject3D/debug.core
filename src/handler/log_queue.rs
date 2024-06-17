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

use crate::handler::{Flag, Handler};
use crate::LogMsg;
use crossbeam_queue::ArrayQueue;
use std::sync::Arc;

const DEFAULT_BUF_SIZE: usize = 32;

/// A log queue.
///
/// The default size of the log queue is 32 log messages, that is 32 * 1024 = 32768 bytes.
#[derive(Clone)]
pub struct LogQueue(Arc<ArrayQueue<LogMsg>>);

impl Default for LogQueue {
    fn default() -> Self {
        Self::new(DEFAULT_BUF_SIZE)
    }
}

impl LogQueue {
    /// Creates a new [LogQueue](LogQueue).
    ///
    /// The queue acts as a ring-buffer, when it is full, new logs are inserted replacing older
    /// logs.
    ///
    /// # Arguments
    ///
    /// * `buffer_size`: the size of the buffer.
    ///
    /// returns: LogBuffer
    pub fn new(buffer_size: usize) -> Self {
        Self(Arc::new(ArrayQueue::new(buffer_size)))
    }

    /// Pops an element from the queue if any.
    pub fn pop(&self) -> Option<LogMsg> {
        self.0.pop()
    }

    /// Clears the log queue.
    pub fn clear(&self) {
        while self.pop().is_some() {}
    }
}

/// A basic handler which redirects log messages to a queue.
pub struct LogQueueHandler {
    queue: LogQueue,
}

impl LogQueueHandler {
    /// Creates a new [LogQueueHandler](LogQueueHandler)
    ///
    /// # Arguments
    ///
    /// * `queue`: the queue to record log messages into.
    ///
    /// returns: LogQueueHandler
    pub fn new(queue: LogQueue) -> Self {
        Self { queue }
    }
}

impl Handler for LogQueueHandler {
    fn install(&mut self, _: &Flag) {}

    fn write(&mut self, msg: &LogMsg) {
        self.queue.0.force_push(msg.clone());
    }

    fn flush(&mut self) {}
}
