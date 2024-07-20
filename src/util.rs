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

/// The context of a log message.
#[derive(Clone, Copy)]
pub struct Location {
    module_path: &'static str,
    file: &'static str,
    line: u32,
}

impl Location {
    /// Creates a new instance of a log message location.
    ///
    /// This function is const to let the caller store location structures in statics.
    ///
    /// # Arguments
    ///
    /// * `module_path`: the module path obtained from the [module_path](module_path) macro.
    /// * `file`: the source file obtained from the [file](file) macro.
    /// * `line`: the line number in the source file obtained from the [line](line) macro.
    ///
    /// returns: Metadata
    pub const fn new(module_path: &'static str, file: &'static str, line: u32) -> Self {
        Self {
            module_path,
            file,
            line,
        }
    }

    /// The module path which issued this log message.
    pub fn module_path(&self) -> &'static str {
        self.module_path
    }

    /// The source file which issued this log message.
    pub fn file(&self) -> &'static str {
        self.file
    }

    /// The line in the source file which issued this log message.
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Extracts the target name and the module name from the module path.
    pub fn get_target_module(&self) -> (&'static str, &'static str) {
        extract_target_module(self.module_path)
    }
}

/// Generate a [Location](crate::Location) structure.
#[macro_export]
macro_rules! location {
    () => {
        $crate::util::Location::new(module_path!(), file!(), line!())
    };
}
