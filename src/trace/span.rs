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
use bp3d_logger::Location;
use std::num::NonZeroU32;
use std::sync::OnceLock;

pub struct Callsite {
    name: &'static str,
    location: Location,
    id: OnceLock<NonZeroU32>,
}

impl Callsite {
    pub const fn new(name: &'static str, location: Location) -> Self {
        Self {
            name,
            location,
            id: OnceLock::new(),
        }
    }

    pub fn location(&self) -> &Location {
        &self.location
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn get_id(&'static self) -> &NonZeroU32 {
        self.id
            .get_or_init(|| crate::engine::get().register_callsite(self))
    }
}

pub struct Entered {
    id: NonZeroU32,
}

impl Drop for Entered {
    fn drop(&mut self) {
        crate::engine::get().span_exit(self.id);
    }
}

pub struct Span {
    id: NonZeroU32,
}

impl Span {
    pub fn with_fields(callsite: &'static Callsite, fields: &[Field]) -> Self {
        let id = crate::engine::get().span_create(*callsite.get_id(), fields);
        Self { id }
    }

    pub fn new(callsite: &'static Callsite) -> Self {
        let id = crate::engine::get().span_create(*callsite.get_id(), &[]);
        Self { id }
    }

    pub fn record(&self, fields: &[Field]) {
        crate::engine::get().span_record(self.id, fields);
    }

    pub fn enter(self) -> Entered {
        Entered { id: self.id }
    }
}

#[cfg(test)]
mod tests {
    use crate::profiler::section::Level;
    use crate::trace::span::Span;
    use crate::{fields, span};

    #[test]
    fn api_test() {
        let value = 32;
        let str = "this is a test";
        let lvl = Level::Event;
        span!(API_TEST);
        span!(API_TEST2);
        let _span = Span::new(&API_TEST);
        let span = Span::with_fields(
            &API_TEST2,
            fields!({value} {str} {?lvl} {test=value}).as_ref(),
        );
        span.record(fields!({ test2 = str }).as_ref());
        let _entered = span.enter();
    }
}
