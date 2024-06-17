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
use bp3d_util::format::{FixedBufStr, IoToFmt};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use time::format_description::well_known::Iso8601;

/// A file handler which writes log messages into different files each named by the target name.
pub struct FileHandler {
    targets: HashMap<String, BufWriter<File>>,
    path: PathBuf,
}

impl FileHandler {
    /// Creates a new [FileHandler](FileHandler).
    ///
    /// # Arguments
    ///
    /// * `path`: the path to the base folder which should contain logs.
    ///
    /// returns: FileHandler
    pub fn new(path: PathBuf) -> FileHandler {
        FileHandler {
            targets: HashMap::new(),
            path,
        }
    }

    fn get_create_open_file(
        &mut self,
        target: &str,
    ) -> Result<&mut BufWriter<File>, std::io::Error> {
        if self.targets.get(target).is_none() {
            let f = OpenOptions::new()
                .append(true)
                .create(true)
                .open(self.path.join(format!("{}.log", target)))?;
            self.targets.insert(target.into(), BufWriter::new(f));
        }
        unsafe {
            // This cannot never fail because None is captured and initialized by the if block.
            Ok(self.targets.get_mut(target).unwrap_unchecked())
        }
    }
}

impl Handler for FileHandler {
    fn install(&mut self, _: &Flag) {}

    fn write(&mut self, msg: &LogMsg) {
        let (target, module) = msg.location().get_target_module();
        let mut wrapper = IoToFmt::new(FixedBufStr::<128>::new());
        let _ = msg.time().format_into(&mut wrapper, &Iso8601::DEFAULT);
        let time_str = wrapper.into_inner();
        if let Ok(file) = self.get_create_open_file(target) {
            let _ = writeln!(
                file,
                "[{}] ({}) {}: {}",
                msg.level(),
                time_str.str(),
                module,
                msg.msg()
            );
        }
    }

    fn flush(&mut self) {
        for v in self.targets.values_mut() {
            let _ = v.flush();
        }
    }
}
