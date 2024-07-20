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

use crate::field::FieldSet;
use crate::util::Location;
use std::num::NonZeroU32;
use std::sync::OnceLock;
use std::time::Instant;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Level {
    /// A section located in a critically hot path.
    Critical = 0,

    /// A periodic section.
    Periodic = 1,

    // An event based section.
    Event = 2,
}

thread_local! {
    static CUR_TIME: Instant = Instant::now();
}

pub struct Entered<'a, const N: usize> {
    id: NonZeroU32,
    start: u64,
    fields: FieldSet<'a, N>,
}

impl<'a, const N: usize> Drop for Entered<'a, N> {
    fn drop(&mut self) {
        let end = CUR_TIME.with(|v| v.elapsed().as_nanos() as _);
        crate::engine::get().section_record(self.id, self.start, end, self.fields.as_ref());
    }
}

pub struct Section {
    name: &'static str,
    location: Location,
    level: Level,
    parent: Option<&'static Section>,
    id: OnceLock<NonZeroU32>,
}

impl Section {
    pub const fn new(name: &'static str, location: Location, level: Level) -> Self {
        Self {
            name,
            location,
            level,
            parent: None,
            id: OnceLock::new(),
        }
    }

    pub const fn set_parent(mut self, parent: &'static Section) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn location(&self) -> &Location {
        &self.location
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn parent(&self) -> Option<&'static Section> {
        self.parent
    }

    pub fn get_id(&'static self) -> &NonZeroU32 {
        self.id
            .get_or_init(|| crate::engine::get().section_register(self))
    }

    pub fn enter<'a, const N: usize>(&'static self, fields: FieldSet<'a, N>) -> Entered<'a, N> {
        let id = self.get_id();
        Entered {
            id: *id,
            start: CUR_TIME.with(|v| v.elapsed().as_nanos() as _),
            fields,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::field::FieldSet;
    use crate::profiler::section::{Level, Section};
    use crate::{fields, location, profiler_section_start};

    #[test]
    fn basic() {
        static _SECTION: Section = Section::new("api_test", location!(), Level::Event);
    }

    #[test]
    fn api_test() {
        static SECTION: Section = Section::new("api_test", location!(), Level::Event);
        static _SECTION2: Section =
            Section::new("api_test2", location!(), Level::Event).set_parent(&SECTION);
        SECTION.enter(FieldSet::new(fields!()));
        SECTION.enter(FieldSet::new(fields!({ test = 42 })));
        SECTION.enter(FieldSet::new(fields!({ test = "test 123" })));
        SECTION.enter(FieldSet::new(fields!({ test = 42.42 })));
        SECTION.enter(FieldSet::new(fields!({test=?Level::Event})));
        SECTION.enter(FieldSet::new(fields!({test=?Level::Event} {test2=42})));
        let value = 32;
        let str = "this is a test";
        let lvl = Level::Event;
        SECTION.enter(FieldSet::new(fields!({value} {str} {?lvl} {test = value})));
    }

    #[test]
    fn api_test2() {
        let value = 32;
        let str = "this is a test";
        let lvl = Level::Event;
        profiler_section_start!(API_TEST, Level::Event);
        profiler_section_start!(API2_TEST: API_TEST, Level::Event);
        profiler_section_start!(API3_TEST_WITH_PARAMS: API2_TEST, Level::Event, {value} {str} {?lvl} {test=value});
    }
}
