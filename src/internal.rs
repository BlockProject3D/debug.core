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

use crate::backend::Backend;
use crate::{LogMsg, Builder, Level};
use crossbeam_channel::{bounded, Receiver, Sender};
use crossbeam_queue::ArrayQueue;
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const BUF_SIZE: usize = 16; // The maximum count of log messages in the channel.

//Disable large_enum_variant as using a Box will inevitably cause a small allocation on a critical path,
//allocating in a critical code path will most likely result in degraded performance.
//And yes, logging is a critical path when using bp3d-tracing.
#[allow(clippy::large_enum_variant)]
enum Command {
    Flush,
    Log(LogMsg),
    Terminate,
    EnableLogBuffer,
    DisableLogBuffer,
}

fn log<T: Backend>(
    backend: Option<&mut T>,
    target: &str,
    msg: &str,
    level: Level,
) -> Result<(), T::Error> {
    if let Some(back) = backend {
        back.write(target, msg, level)
    } else {
        Ok(())
    }
}

struct Thread {
    logger: Builder,
    recv_ch: Receiver<Command>,
    enable_log_buffer: bool,
    log_buffer: Arc<ArrayQueue<LogMsg>>,
}

impl Thread {
    pub fn new(
        logger: Builder,
        recv_ch: Receiver<Command>,
        log_buffer: Arc<ArrayQueue<LogMsg>>,
    ) -> Thread {
        Thread {
            logger,
            recv_ch,
            enable_log_buffer: false,
            log_buffer,
        }
    }

    fn exec_commad(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::Terminate => true,
            Command::Flush => {
                if let Some(file) = &mut self.logger.file {
                    if let Err(e) = file.flush() {
                        let _ = log(
                            self.logger.std.as_mut(),
                            "bp3d-logger",
                            &format!("Could not flush file backend: {}", e),
                            Level::Error,
                        );
                    }
                }
                false
            }
            Command::Log(buffer) => {
                let target = buffer.target();
                let msg = buffer.msg();
                let level = buffer.level();
                if let Err(e) = log(self.logger.file.as_mut(), target, msg, level) {
                    let _ = log(
                        self.logger.std.as_mut(),
                        "bp3d-logger",
                        &format!("Could not write to file backend: {}", e),
                        Level::Error,
                    );
                }
                let _ = log(self.logger.std.as_mut(), target, msg, level);
                if self.enable_log_buffer {
                    self.log_buffer.force_push(buffer);
                }
                false
            }
            Command::EnableLogBuffer => {
                self.enable_log_buffer = true;
                false
            }
            Command::DisableLogBuffer => {
                self.enable_log_buffer = false;
                false
            }
        }
    }

    pub fn run(mut self) {
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
    enabled: AtomicBool,
    enable_stdout: Option<Arc<AtomicBool>>,
    log_buffer: Arc<ArrayQueue<LogMsg>>,
    thread: ManuallyDrop<std::thread::JoinHandle<()>>,
}

//TODO: Implement log Level checking for Logger2.checked_log and support setting/getting current log level.
//TODO: Implement support for multiple custom log backends.

impl Logger {
    pub(crate) fn new(builder: Builder) -> Logger {
        let buf_size = builder.buf_size.unwrap_or(BUF_SIZE);
        let (send_ch, recv_ch) = bounded(buf_size);
        let log_buffer = Arc::new(ArrayQueue::new(buf_size));
        let recv_ch1 = recv_ch.clone();
        let log_buffer1 = log_buffer.clone();
        let enable_stdout = builder.std.as_ref().map(|v| v.get_enable());
        let thread = std::thread::spawn(move || {
            let thread = Thread::new(builder, recv_ch1, log_buffer1);
            thread.run();
        });
        Logger {
            thread: ManuallyDrop::new(thread),
            send_ch,
            log_buffer,
            enabled: AtomicBool::new(true),
            enable_stdout
        }
    }

    /// Enables the stdout/stderr logger.
    ///
    /// # Arguments
    ///
    /// * `flag`: true to enable stdout, false to disable stdout.
    pub fn enable_stdout(&self, flag: bool) {
        self.enable_stdout.as_ref().map(|v| v.store(flag, Ordering::Release));
    }

    /// Enables this logger.
    pub fn enable(&self, flag: bool) {
        self.enabled.store(flag, Ordering::Release);
    }

    /// Enables the log redirect pump.
    pub fn enable_log_buffer(&self, flag: bool) {
        unsafe {
            if flag {
                self.send_ch
                    .send(Command::EnableLogBuffer)
                    .unwrap_unchecked();
            } else {
                self.send_ch
                    .send(Command::DisableLogBuffer)
                    .unwrap_unchecked();
                self.clear_log_buffer();
            }
        }
    }

    /// Clears the log buffer.
    #[inline]
    pub fn clear_log_buffer(&self) {
        while self.log_buffer.pop().is_some() {} //Clear the entire log buffer.
    }

    /// Attempts to extract one log message from the buffer.
    #[inline]
    pub fn read_log(&self) -> Option<LogMsg> {
        self.log_buffer.pop()
    }

    /// Low-level log function. This injects log messages directly into the logging thread channel.
    ///
    /// This function applies basic formatting depending on the backend:
    /// - For stdout/stderr backend the format is \<target\> \[level\] msg.
    /// - For file backend the format is \[level\] msg and the message is recorded in the file
    /// corresponding to the log target.
    ///
    /// WARNING: For optimization reasons, this function does not check and thus does not honor
    /// the enabled flag. For a checked log function, use [checked_log](Self::checked_log).
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

    /// Checked log function. This injects log messages into the logging thread channel only if
    /// this logger is enabled.
    ///
    /// This function calls the [raw_log](Self::raw_log) function only when this logger is enabled.
    #[inline]
    pub fn checked_log(&self, msg: &LogMsg) {
        if self.is_enabled() {
            self.raw_log(msg);
        }
    }

    /// Returns true if the logger is currently enabled and is capturing log messages.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
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
        self.enable(false);

        // Disable the log buffer (this automatically clears it).
        self.enable_log_buffer(false);

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
