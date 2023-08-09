// Copyright (c) 2023, BlockProject 3D
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
use crate::{LogMsg, Logger};
use bp3d_os::time::LocalOffsetDateTime;
use crossbeam_channel::{bounded, Receiver, Sender};
use crossbeam_queue::ArrayQueue;
use log::{Level, Log, Metadata, Record};
use time::OffsetDateTime;
use time::macros::format_description;
use std::fmt::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const BUF_SIZE: usize = 16; // The maximum count of log messages in the channel.

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
    logger: Logger,
    recv_ch: Receiver<Command>,
    enable_log_buffer: bool,
    log_buffer: Arc<ArrayQueue<LogMsg>>,
}

impl Thread {
    pub fn new(
        logger: Logger,
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

pub struct LoggerImpl {
    send_ch: Sender<Command>,
    enabled: AtomicBool,
    recv_ch: Receiver<Command>,
    log_buffer: Arc<ArrayQueue<LogMsg>>,
    thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl LoggerImpl {
    pub fn new() -> LoggerImpl {
        let (send_ch, recv_ch) = bounded(BUF_SIZE);
        LoggerImpl {
            thread: Mutex::new(None),
            send_ch,
            recv_ch,
            log_buffer: Arc::new(ArrayQueue::new(BUF_SIZE)),
            enabled: AtomicBool::new(false),
        }
    }

    pub fn enable(&self, flag: bool) {
        self.enabled.store(flag, Ordering::Release);
    }

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
            }
        }
    }

    #[inline]
    pub fn clear_log_buffer(&self) {
        while self.log_buffer.pop().is_some() {} //Clear the entire log buffer.
    }

    #[inline]
    pub fn read_log(&self) -> Option<LogMsg> {
        self.log_buffer.pop()
    }

    pub fn terminate(&self) {
        // This should never panic as there's no way another call would have panicked!
        let mut thread = self.thread.lock().unwrap();
        if let Some(handle) = thread.take() {
            // This cannot panic as send_ch is owned by LoggerImpl which is intended
            // to be statically allocated.
            unsafe {
                self.send_ch.send(Command::Flush).unwrap_unchecked();
                self.send_ch.send(Command::Terminate).unwrap_unchecked();
            }
            // Join the logging thread; this will lock until the thread is completely terminated.
            handle.join().unwrap();
        }
    }

    pub fn start_new_thread(&self, logger: Logger) {
        let mut flag = false;
        {
            // This should never panic as there's no way another call would have panicked!
            let mut thread = self.thread.lock().unwrap();
            if let Some(handle) = thread.take() {
                // This cannot panic as send_ch is owned by LoggerImpl which is intended
                // to be statically allocated.
                unsafe {
                    self.send_ch.send(Command::Terminate).unwrap_unchecked();
                }
                if handle.join().is_err() {
                    flag = true;
                }
            }
            let recv_ch = self.recv_ch.clone();
            let log_buffer = self.log_buffer.clone();
            *thread = Some(std::thread::spawn(move || {
                let thread = Thread::new(logger, recv_ch, log_buffer);
                thread.run();
            }));
        }
        if flag {
            // Somehow the previous thread has panicked; log that panic...
            unsafe {
                // This cannot panic as send_ch is owned by LoggerImpl which is intended
                // to be statically allocated.
                self.send_ch
                    .send(Command::Log(LogMsg::from_msg(
                        "bp3d-logger",
                        Level::Error,
                        "The logging thread has panicked!",
                    )))
                    .unwrap_unchecked();
            }
        }
    }

    pub fn low_level_log(&self, msg: &LogMsg) {
        unsafe {
            // This cannot panic as send_ch is owned by LoggerImpl which is intended
            // to be statically allocated.
            self.send_ch
                .send(Command::Log(msg.clone()))
                .unwrap_unchecked();
        }
    }

    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }
}

fn extract_target_module<'a>(record: &'a Record) -> (&'a str, Option<&'a str>) {
    let base_string = record.module_path().unwrap_or_else(|| record.target());
    let target = base_string
        .find("::")
        .map(|v| &base_string[..v])
        .unwrap_or(base_string);
    let module = base_string.find("::").map(|v| &base_string[(v + 2)..]);
    (target, module)
}

impl Log for LoggerImpl {
    fn enabled(&self, _: &Metadata) -> bool {
        self.is_enabled()
    }

    fn log(&self, record: &Record) {
        // Apparently the log crate is defective: the enabled function is ignored...
        if !self.enabled(record.metadata()) {
            return;
        }
        let (target, module) = extract_target_module(record);
        let time = Some(OffsetDateTime::now_utc());
        let format = format_description!("[weekday repr:short] [month repr:short] [day] [hour repr:12]:[minute]:[second] [period case:upper]");
        let formatted = time.unwrap_or_else(OffsetDateTime::now_utc).format(format).unwrap_or_default();
        let mut msg = LogMsg::new(target, record.level());
        let _ = write!(
            msg,
            "({}) {}: {}",
            formatted,
            module.unwrap_or("main"),
            record.args()
        );
        self.low_level_log(&msg);
    }

    fn flush(&self) {
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
