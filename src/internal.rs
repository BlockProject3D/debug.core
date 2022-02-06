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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use crossbeam_channel::{Receiver, Sender, bounded};
use log::{Level, Log, Metadata, Record};
use crate::backend::Backend;
use crate::{Logger, LogMsg};

const BUF_SIZE: usize = 128; // The maximum count of log messages in the channel.

enum Command
{
    Flush,
    Log(LogMsg),
    Terminate
}

fn log<T: Backend>(backend: Option<&mut T>, target: &str, msg: &str, level: Level) -> Result<(), T::Error>
{
    if let Some(back) = backend {
        back.write(target, msg, level)
    } else {
        Ok(())
    }
}

fn exec_commad(cmd: Command, logger: &mut Logger) -> bool
{
    match cmd {
        Command::Terminate => true,
        Command::Flush => {
            if let Some(file) = &mut logger.file {
                if let Err(e) = file.flush() {
                    let _ = log(logger.std.as_mut(),
                                "bp3d-logger",
                                &format!("Could not flush file backend: {}", e),
                                Level::Error);
                }
            }
            false
        },
        Command::Log(LogMsg { target, msg, level }) => {
            if let Err(e) = log(logger.file.as_mut(), &target, &msg, level) {
                let _ = log(logger.std.as_mut(),
                            "bp3d-logger",
                            &format!("Could not write to file backend: {}", e),
                            Level::Error);
            }
            let _ = log(logger.std.as_mut(), &target, &msg, level);
            false
        }
    }
}

pub struct LoggerImpl
{
    thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    send_ch: Sender<Command>,
    recv_ch: Receiver<Command>,
    log_buffer_send_ch: Sender<LogMsg>,
    log_buffer_recv_ch: Receiver<LogMsg>,
    enable_log_buffer: AtomicBool
}

impl LoggerImpl {
    pub fn new() -> LoggerImpl {
        let (send_ch, recv_ch) = bounded(BUF_SIZE);
        let (log_buffer_send_ch, log_buffer_recv_ch) = bounded(BUF_SIZE);
        LoggerImpl {
            thread: Mutex::new(None),
            send_ch,
            recv_ch,
            log_buffer_send_ch,
            log_buffer_recv_ch,
            enable_log_buffer: AtomicBool::new(false)
        }
    }

    pub fn enable_log_buffer(&self, flag: bool) {
        self.enable_log_buffer.store(flag, Ordering::Release);
    }

    pub fn clear_log_buffer(&self) {
        while let Ok(_) = self.log_buffer_recv_ch.try_recv() {} //Clear the entire log buffer.
    }

    pub fn get_log_buffer(&self) -> Receiver<LogMsg> {
        self.log_buffer_recv_ch.clone()
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
                if let Err(_) = handle.join() {
                    flag = true;
                }
            }
            let recv_ch = self.recv_ch.clone();
            *thread = Some(std::thread::spawn(move || {
                let mut logger = logger;
                while let Ok(v) = recv_ch.recv() {
                    exec_commad(v, &mut logger);
                }
            }));
        }
        if flag { // Somehow the previous thread has panicked; log that panic...
            unsafe {
                // This cannot panic as send_ch is owned by LoggerImpl which is intended
                // to be statically allocated.
                self.send_ch.send(Command::Log(LogMsg {
                    level: Level::Error,
                    msg: "The logging thread has panicked!".into(),
                    target: "bp3d-logger".into()
                })).unwrap_unchecked();
            }
        }
    }
}

fn extract_target_module<'a>(record: &'a Record) -> (&'a str, Option<&'a str>)
{
    let base_string = record.module_path().unwrap_or(record.target());
    let target = base_string.find("::")
        .map(|v| &base_string[..v])
        .unwrap_or(base_string);
    let module = base_string.find("::")
        .map(|v| &base_string[(v + 2)..]);
    (target, module)
}

impl Log for LoggerImpl
{
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let (target, module) = extract_target_module(record);
        let msg = LogMsg {
            msg: format!("({}) {}: {}", "<time here>", module.unwrap_or("main"), record.args()),
            target: target.into(),
            level: record.level()
        };
        if self.enable_log_buffer.load(Ordering::Acquire) {
            unsafe {
                // This cannot panic as both send_ch and log_buffer_send_ch are owned by LoggerImpl
                // which is intended to be statically allocated.
                self.send_ch.send(Command::Log(msg.clone())).unwrap_unchecked();
                self.log_buffer_send_ch.send(msg).unwrap_unchecked();
            }
        } else {
            unsafe {
                // This cannot panic as send_ch is owned by LoggerImpl which is intended
                // to be statically allocated.
                self.send_ch.send(Command::Log(msg)).unwrap_unchecked();
            }
        }
    }

    fn flush(&self) {
        unsafe {
            // This cannot panic as send_ch is owned by LoggerImpl which is intended
            // to be statically allocated.
            self.send_ch.send(Command::Flush).unwrap_unchecked();
        }
    }
}
