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

use log::Level;
use std::fmt::{Error, Write};
use std::mem::MaybeUninit;

// Limit the size of the target string to 16 bytes.
const LOG_TARGET_SIZE: usize = 16;
// Size of the control fields of the log message structure:
// sizeof msg_len + 1 byte for target_len + 1 byte for level
const LOG_CONTROL_SIZE: usize = std::mem::size_of::<u32>() + 2;
// Limit the size of the log message string so that the size of the log structure is LOG_BUFFER_SIZE
const LOG_MSG_SIZE: usize = LOG_BUFFER_SIZE - LOG_TARGET_SIZE - LOG_CONTROL_SIZE;
const LOG_BUFFER_SIZE: usize = 1024;

#[inline]
fn log_to_u8(level: Level) -> u8 {
    match level {
        Level::Error => 0,
        Level::Warn => 1,
        Level::Info => 2,
        Level::Debug => 3,
        Level::Trace => 4
    }
}

#[inline]
fn u8_to_log(l: u8) -> Level {
    match l {
        0 => Level::Error,
        1 => Level::Warn,
        3 => Level::Debug,
        4 => Level::Trace,
        _ => Level::Info
    }
}

/// A log message.
///
/// This structure uses a large 1K buffer which stores the entire log message to improve
/// performance.
///
/// The repr(C) is used to force the control fields (msg_len, level and target_len) to be before
/// the message buffer and avoid large movs when setting control fields.
///
/// # Examples
///
/// ```
/// use log::Level;
/// use bp3d_logger::LogMsg;
/// use std::fmt::Write;
/// let mut msg = LogMsg::new("test", Level::Info);
/// let _ = write!(msg, "This is a formatted message {}", 42);
/// assert_eq!(msg.msg(), "This is a formatted message 42");
/// ```
#[derive(Clone)]
#[repr(C)]
pub struct LogMsg {
    msg_len: u32,
    level: u8,
    target_len: u8,
    buffer: [MaybeUninit<u8>; LOG_MSG_SIZE + LOG_TARGET_SIZE]
}

impl LogMsg {
    /// Creates a new instance of log message buffer.
    ///
    /// # Arguments
    ///
    /// * `target`: the target name this log comes from.
    /// * `level`: the [Level](Level) of the log message.
    ///
    /// returns: LogMsg
    ///
    /// # Examples
    ///
    /// ```
    /// use log::Level;
    /// use bp3d_logger::LogMsg;
    /// let msg = LogMsg::new("test", Level::Info);
    /// assert_eq!(msg.target(), "test");
    /// assert_eq!(msg.level(), Level::Info);
    /// ```
    pub fn new(target: &str, level: Level) -> LogMsg {
        let len = std::cmp::min(LOG_TARGET_SIZE, target.as_bytes().len());
        let mut buffer = LogMsg {
            buffer: unsafe { MaybeUninit::uninit().assume_init() },
            target_len: len as _,
            msg_len: len as _,
            level: log_to_u8(level),
        };
        unsafe {
            std::ptr::copy_nonoverlapping(
                target.as_bytes().as_ptr(),
                std::mem::transmute(buffer.buffer.as_mut_ptr()),
                len,
            );
        }
        buffer
    }

    /// Clears the log message but keep the target and the level.
    ///
    /// # Examples
    ///
    /// ```
    /// use log::Level;
    /// use bp3d_logger::LogMsg;
    /// let mut msg = LogMsg::from_msg("test", Level::Info, "this is a test");
    /// msg.clear();
    /// assert_eq!(msg.msg(), "");
    /// assert_eq!(msg.target(), "test");
    /// assert_eq!(msg.level(), Level::Info);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.msg_len = self.target_len as _;
    }

    /// Auto-creates a new log message with a pre-defined string message.
    ///
    /// This function is the same as calling [write](LogMsg::write) after [new](LogMsg::new).
    ///
    /// # Arguments
    ///
    /// * `target`: the target name this log comes from.
    /// * `level`: the [Level](Level) of the log message.
    /// * `msg`: the message string.
    ///
    /// returns: LogMsg
    ///
    /// # Examples
    ///
    /// ```
    /// use log::Level;
    /// use bp3d_logger::LogMsg;
    /// let mut msg = LogMsg::from_msg("test", Level::Info, "this is a test");
    /// assert_eq!(msg.target(), "test");
    /// assert_eq!(msg.level(), Level::Info);
    /// assert_eq!(msg.msg(), "this is a test");
    /// ```
    pub fn from_msg(target: &str, level: Level, msg: &str) -> LogMsg {
        let mut ads = Self::new(target, level);
        unsafe { ads.write(msg.as_bytes()) };
        ads
    }

    /// Appends a raw byte buffer at the end of the message buffer.
    ///
    /// Returns the number of bytes written.
    ///
    /// # Arguments
    ///
    /// * `buf`: the raw byte buffer to append.
    ///
    /// returns: usize
    ///
    /// # Safety
    ///
    /// * [LogMsg](LogMsg) contains only valid UTF-8 strings so buf must contain only valid UTF-8
    /// bytes.
    /// * If buf contains invalid UTF-8 bytes, further operations on the log message buffer may
    /// result in UB.
    pub unsafe fn write(&mut self, buf: &[u8]) -> usize {
        let len = std::cmp::min(buf.len(), LOG_MSG_SIZE - self.msg_len as usize);
        if len > 0 {
            std::ptr::copy_nonoverlapping(
                buf.as_ptr(),
                std::mem::transmute(self.buffer.as_mut_ptr().offset(self.msg_len as _)),
                len,
            );
            self.msg_len += len as u32; //The length is always less than 2^32.
        }
        len
    }

    /// Returns the target name this log comes from.
    #[inline]
    pub fn target(&self) -> &str {
        // SAFEY: This is always safe because BufLogMsg is always UTF-8.
        unsafe {
            std::str::from_utf8_unchecked(std::mem::transmute(&self.buffer[..self.target_len as _]))
        }
    }

    /// Returns the log message as a string.
    #[inline]
    pub fn msg(&self) -> &str {
        // SAFEY: This is always safe because BufLogMsg is always UTF-8.
        unsafe {
            std::str::from_utf8_unchecked(std::mem::transmute(
                &self.buffer[self.target_len as _..self.msg_len as _],
            ))
        }
    }

    /// Returns the level of this log message.
    #[inline]
    pub fn level(&self) -> Level {
        u8_to_log(self.level)
    }
}

impl Write for LogMsg {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        unsafe {
            self.write(s.as_bytes());
        }
        Ok(())
    }
}
