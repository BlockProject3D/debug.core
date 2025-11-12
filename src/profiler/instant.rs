// Copyright (c) 2025, BlockProject 3D
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

// This file is duplicated from bp3d_os::time::Instant.
// This should not be the case but Cargo has decided to do so!!

#[cfg(unix)]
use libc::{clock_gettime, timespec, CLOCK_MONOTONIC_RAW};
#[cfg(unix)]
use std::hash::Hash;
use std::time::Duration;

#[cfg(unix)]
trait DurationNewUnchecked {
    unsafe fn new_unchecked(secs: u64, subsec_nanos: u32) -> Duration;
}

#[cfg(unix)]
impl DurationNewUnchecked for Duration {
    unsafe fn new_unchecked(secs: u64, subsec_nanos: u32) -> Duration {
        const NANOS_PER_SEC: u32 = 1000000000;
        if subsec_nanos >= NANOS_PER_SEC {
            unsafe { std::hint::unreachable_unchecked() }
        }
        Duration::new(secs, subsec_nanos)
    }
}

#[cfg(unix)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Instant(Duration);

#[cfg(windows)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Instant(std::time::Instant);

#[cfg(windows)]
impl Instant {
    #[inline(always)]
    pub fn now() -> Self {
        Self(std::time::Instant::now())
    }

    #[inline(always)]
    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
}

#[cfg(unix)]
impl Instant {
    pub fn now() -> Self {
        let mut t = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe { clock_gettime(CLOCK_MONOTONIC_RAW, &mut t) };
        Self(unsafe { Duration::new_unchecked(t.tv_sec as _, t.tv_nsec as _) })
    }

    pub fn elapsed(&self) -> Duration {
        let mut other = timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe { clock_gettime(CLOCK_MONOTONIC_RAW, &mut other) };
        // Need super slow code cause somehow CLOCK_MONOTONIC_RAW is randomly broken.
        let a = unsafe { Duration::new_unchecked(other.tv_sec as _, other.tv_nsec as _) };
        a - self.0
    }
}
