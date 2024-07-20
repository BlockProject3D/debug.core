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

use crate::engine::ENGINE_INIT_FLAG;
use crate::field::Field;
use crate::trace::span::Callsite;
use std::fmt::Arguments;
use std::num::NonZeroU32;
use std::sync::atomic::Ordering;

pub struct VoidDebugger {}

impl crate::profiler::Profiler for VoidDebugger {
    fn section_register(&self, _: &'static crate::profiler::section::Section) -> NonZeroU32 {
        ENGINE_INIT_FLAG.store(true, Ordering::Relaxed);
        unsafe { NonZeroU32::new_unchecked(1) }
    }

    fn section_record(&self, _: NonZeroU32, _: u64, _: u64, _: &[Field]) {
        ENGINE_INIT_FLAG.store(true, Ordering::Relaxed);
    }
}

impl crate::trace::Tracer for VoidDebugger {
    fn register_callsite(&self, _: &'static Callsite) -> NonZeroU32 {
        ENGINE_INIT_FLAG.store(true, Ordering::Relaxed);
        unsafe { NonZeroU32::new_unchecked(1) }
    }

    fn span_create(&self, _: NonZeroU32, _: &[Field]) -> NonZeroU32 {
        ENGINE_INIT_FLAG.store(true, Ordering::Relaxed);
        unsafe { NonZeroU32::new_unchecked(1) }
    }

    fn span_enter(&self, _: NonZeroU32) {
        ENGINE_INIT_FLAG.store(true, Ordering::Relaxed);
    }

    fn span_record(&self, _: NonZeroU32, _: &[Field]) {
        ENGINE_INIT_FLAG.store(true, Ordering::Relaxed);
    }

    fn span_exit(&self, _: NonZeroU32) {
        ENGINE_INIT_FLAG.store(true, Ordering::Relaxed);
    }
}

impl crate::logger::Logger for VoidDebugger {
    fn log(&self, callsite: &'static crate::logger::Callsite, args: Arguments, _: &[Field]) {
        println!(
            "[{}] {}: {}",
            callsite.level(),
            callsite.location().module_path(),
            args
        );
        ENGINE_INIT_FLAG.store(true, Ordering::Relaxed);
    }
}
