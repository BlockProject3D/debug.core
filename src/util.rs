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

use std::fmt::Write;
use bp3d_os::time::LocalOffsetDateTime;
use bp3d_util::format::IoToFmt;
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

/// Write time information into the given [Write](Write).
///
/// # Arguments
///
/// * `msg`: the [Write](Write) to write time information to.
/// * `time`: the time to write.
///
/// returns: ()
pub fn write_time(msg: &mut impl Write, time: OffsetDateTime) {
    let _ = msg.write_str("(");
    let format = format_description!("[weekday repr:short] [month repr:short] [day] [hour repr:12]:[minute]:[second] [period case:upper]");
    let mut wrapper = IoToFmt::new(msg);
    let _ = time.format_into(&mut wrapper, format);
    let msg = wrapper.into_inner();
    let _ = msg.write_str(")");
}

#[cfg(test)]
mod tests  {
    use bp3d_os::time::LocalOffsetDateTime;
    use time::OffsetDateTime;
    use crate::{Level, Location, LogMsg};
    use crate::util::write_time;

    #[test]
    fn fhsdiub() {
        let time = OffsetDateTime::now_local().unwrap_or_else(OffsetDateTime::now_utc);
        let mut msg = LogMsg::new(Location::new("test", "test.c", 1), Level::Info);
        write_time(&mut msg, time);
        assert!(msg.msg().len() > 0);
    }
}
