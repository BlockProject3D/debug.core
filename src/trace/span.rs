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
use crate::util::Location;
use std::num::{NonZeroU32, NonZeroU64};
use std::sync::OnceLock;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Id(NonZeroU64);

impl Id {
    pub fn new(callsite: NonZeroU32, instance: NonZeroU32) -> Self {
        Self(unsafe {
            NonZeroU64::new_unchecked((callsite.get() as u64) << 32 | instance.get() as u64)
        })
    }

    pub fn from_raw(id: NonZeroU64) -> Self {
        Self(id)
    }

    pub fn into_raw(self) -> NonZeroU64 {
        self.0
    }

    pub fn get_callsite(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked((self.0.get() >> 32) as u32) }
    }

    pub fn get_instance(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(self.0.get() as u32) }
    }
}

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
    id: Id,
}

impl Drop for Entered {
    fn drop(&mut self) {
        crate::engine::get().span_exit(self.id);
    }
}

pub struct Span {
    id: Id,
}

impl Span {
    pub fn with_fields(callsite: &'static Callsite, fields: &[Field]) -> Self {
        let callsite = *callsite.get_id();
        let instance = crate::engine::get().span_create(callsite, fields);
        Self {
            id: Id::new(callsite, instance),
        }
    }

    pub fn new(callsite: &'static Callsite) -> Self {
        let callsite = *callsite.get_id();
        let instance = crate::engine::get().span_create(callsite, &[]);
        Self {
            id: Id::new(callsite, instance),
        }
    }

    pub fn record(&self, fields: &[Field]) {
        crate::engine::get().span_record(self.id, fields);
    }

    pub fn enter(&self) -> Entered {
        Entered { id: self.id }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        crate::engine::get().span_destroy(self.id);
    }
}

#[cfg(test)]
mod tests {
    use crate::profiler::section::Level;
    use crate::{fields, span};

    #[test]
    fn api_test() {
        let value = 32;
        let str = "this is a test";
        let lvl = Level::Event;
        let _span = span!(API_TEST);
        let span = span!(API_TEST2, {value} {str} {?lvl} {test=value});
        span.record(fields!({ test2 = str }).as_ref());
        let _entered = span.enter();
    }
}
