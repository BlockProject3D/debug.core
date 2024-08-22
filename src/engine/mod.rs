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

use std::sync::atomic::{AtomicBool, Ordering};

mod void;

pub trait Engine:
    crate::logger::Logger + crate::profiler::Profiler + crate::trace::Tracer + Sync
{
}
impl<T: crate::logger::Logger + crate::profiler::Profiler + crate::trace::Tracer + Sync> Engine
    for T
{
}

static ENGINE_INIT_FLAG: AtomicBool = AtomicBool::new(false);

static mut ENGINE: &dyn Engine = &void::VoidDebugger {};

pub fn get() -> &'static dyn Engine {
    unsafe { ENGINE }
}

pub fn set(engine: &'static dyn Engine) -> bool {
    let flag = ENGINE_INIT_FLAG.load(Ordering::Relaxed);
    if flag {
        return false;
    }
    unsafe { ENGINE = engine };
    ENGINE_INIT_FLAG.store(true, Ordering::Relaxed);
    true
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;
    use crate::trace::span::Id;

    #[test]
    fn basic() {
        crate::engine::set(&crate::engine::void::VoidDebugger {});
        assert!(!crate::engine::set(&crate::engine::void::VoidDebugger {}));
    }

    #[test]
    fn after_use() {
        crate::engine::get().span_exit(Id::new(unsafe { NonZeroU32::new_unchecked(1) }, unsafe { NonZeroU32::new_unchecked(1) }));
        assert!(!crate::engine::set(&crate::engine::void::VoidDebugger {}));
    }
}
