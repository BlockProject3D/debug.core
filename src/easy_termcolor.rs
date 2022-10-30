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

use log::Level;
use std::fmt::Display;
use termcolor::{Color, ColorSpec};

pub struct EasyTermColor<T: termcolor::WriteColor>(pub T);

impl<T: termcolor::WriteColor> EasyTermColor<T> {
    pub fn write(mut self, elem: impl Display) -> Self {
        let _ = write!(&mut self.0, "{}", elem);
        self
    }

    pub fn color(mut self, elem: ColorSpec) -> Self {
        let _ = self.0.set_color(&elem);
        self
    }

    pub fn reset(mut self) -> Self {
        let _ = self.0.reset();
        self
    }

    pub fn lf(mut self) -> Self {
        let _ = writeln!(&mut self.0);
        self
    }
}

pub fn color(level: Level) -> ColorSpec {
    match level {
        Level::Error => ColorSpec::new()
            .set_fg(Some(Color::Red))
            .set_bold(true)
            .clone(),
        Level::Warn => ColorSpec::new()
            .set_fg(Some(Color::Yellow))
            .set_bold(true)
            .clone(),
        Level::Info => ColorSpec::new()
            .set_fg(Some(Color::Green))
            .set_bold(true)
            .clone(),
        Level::Debug => ColorSpec::new()
            .set_fg(Some(Color::Blue))
            .set_bold(true)
            .clone(),
        Level::Trace => ColorSpec::new()
            .set_fg(Some(Color::Cyan))
            .set_bold(true)
            .clone(),
    }
}
