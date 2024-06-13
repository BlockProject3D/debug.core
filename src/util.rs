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

//! Logging utilities.

use bp3d_os::time::LocalOffsetDateTime;
use time::macros::format_description;
use time::OffsetDateTime;
use crate::LogMsg;

/// Extracts the target name and the module path (without the target name) from a full module path string.
///
/// # Arguments
///
/// * `base_string`: a full module path string (ex: bp3d_logger::util::extract_target_module).
///
/// returns: (&str, &str)
pub fn extract_target_module(base_string: &str) -> (&str, &str) {
    let target = base_string
        .find("::")
        .map(|v| &base_string[..v])
        .unwrap_or(base_string);
    let module = base_string.find("::").map(|v| &base_string[(v + 2)..]);
    (target, module.unwrap_or("main"))
}

/// Unsafe [Write](std::io::Write) wrapper for [LogMsg].
///
/// This utility is provided for interactions with foreign APIs that only supports writing through
/// the io::Write interface and are GUARANTEED to result in UTF-8 data (ex: time crate).
pub struct IoWrapper<'a>(&'a mut LogMsg);

impl<'a> IoWrapper<'a> {
    /// Creates a new [Write](std::io::Write) wrapper for [LogMsg].
    ///
    /// Safety
    ///
    /// Subsequent calls to [Write](std::io::Write) must result in a valid UTF-8 string
    /// once a call to [msg](LogMsg::msg) is made.
    pub unsafe fn new(msg: &'a mut LogMsg) -> Self {
        Self(msg)
    }

    /// Extracts the underlying [LogMsg] from this wrapper.
    ///
    /// Safety
    ///
    /// The content of the underlying [LogMsg] MUST be a valid UTF-8 string.
    pub unsafe fn into_inner(self) -> &'a mut LogMsg {
        self.0
    }
}

impl<'a> std::io::Write for IoWrapper<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe { Ok(self.0.write(buf)) }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Write time information into the given [LogMsg].
///
/// # Arguments
///
/// * `msg`: the [LogMsg] to write time information to.
/// * `time`: the time to write.
///
/// returns: ()
pub fn write_time(msg: &mut LogMsg, time: OffsetDateTime) {
    unsafe { msg.write(b"(") };
    let format = format_description!("[weekday repr:short] [month repr:short] [day] [hour repr:12]:[minute]:[second] [period case:upper]");
    let mut wrapper = unsafe { IoWrapper::new(msg) };
    let _ = time.format_into(&mut wrapper, format);
    unsafe { msg.write(b") ") };
}

/// Adds the current time to the given [LogMsg].
///
/// # Arguments
///
/// * `msg`: the [LogMsg] to write time information to.
///
/// returns: ()
pub fn add_time(msg: &mut LogMsg) {
    write_time(msg, OffsetDateTime::now_local().unwrap_or_else(OffsetDateTime::now_utc))
}
