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

use crate::field::Field;
use crate::logger::Level;
use crate::util::Location;
use std::fmt::Arguments;

pub struct Callsite {
    location: Location,
    level: Level,
}

impl Callsite {
    pub const fn new(location: Location, level: Level) -> Self {
        Self { location, level }
    }

    pub fn location(&self) -> &Location {
        &self.location
    }

    pub fn level(&self) -> Level {
        self.level
    }
}

pub trait Logger {
    fn log(&self, callsite: &'static Callsite, msg: Arguments, fields: &[Field]);
}

#[cfg(test)]
mod tests {
    use crate::logger::Level;
    use crate::{log, trace};

    #[test]
    fn api_test() {
        let tuple = (41, 42);
        let i = 42;
        let b = true;
        log!(Level::Info, { i }, "test: {i}: {}", i);
        log!(Level::Error, "test: {}", i);
        trace!({i} {?i} {id=i}, "test: {}", i);
        trace!("test: {}, {}", i, i);
        trace!("test41_42: {}, {}", tuple.0, tuple.1);
        trace!({b}, "a boolean");
    }
}
