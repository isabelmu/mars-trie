// Copyright (c) 2015, Johannes Muenzel
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// - Redistributions of source code must retain the above copyright notice, this
//   list of conditions and the following disclaimer.
// - Redistributions in binary form must reproduce the above copyright notice,
//   this list of conditions and the following disclaimer in the documentation
//   and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.


// FIXME: For now, these are portable versions of intrinsic functions we need.
// They should be replaced by the fastest available intrinsics on each supported
// platform, and the fastest known portable version on other platforms.

/// Count trailing zeros
pub trait Ctz {
    fn ctz(self) -> u32;
}

impl Ctz for u8 {
    fn ctz(self) -> u32 {
        (self as u32).ctz()
    }
}

impl Ctz for u16 {
    fn ctz(self) -> u32 {
        (self as u32).ctz()
    }
}

impl Ctz for u32 {
    fn ctz(self) -> u32 {
        let v = self;
        let mut c: u32 = 32;
        let v = v & ((-(v as i32)) as u32);
        if 0 != v { c -= 1; }
        if 0 != (v & 0x0000FFFF) { c -= 16; }
        if 0 != (v & 0x00FF00FF) { c -= 8; }
        if 0 != (v & 0x0F0F0F0F) { c -= 4; }
        if 0 != (v & 0x33333333) { c -= 2; }
        if 0 != (v & 0x55555555) { c -= 1; }
        c
    }
}

impl Ctz for u64 {
    fn ctz(self) -> u32 {
        let v = self;
        let mut c: u32 = 64;
        let v = v & ((-(v as i64)) as u64);
        if 0 != v { c -= 1; }
        if 0 != (v & 0x00000000FFFFFFFF) { c -= 32; }
        if 0 != (v & 0x0000FFFF0000FFFF) { c -= 16; }
        if 0 != (v & 0x00FF00FF00FF00FF) { c -= 8; }
        if 0 != (v & 0x0F0F0F0F0F0F0F0F) { c -= 4; }
        if 0 != (v & 0x3333333333333333) { c -= 2; }
        if 0 != (v & 0x5555555555555555) { c -= 1; }
        c
    }
}

#[cfg(target_pointer_width = "32")]
impl Ctz for usize {
    fn ctz(self) -> u32 {
        (self as u32).ctz()
    }
}

#[cfg(target_pointer_width = "64")]
impl Ctz for usize {
    fn ctz(self) -> u32 {
        (self as u64).ctz()
    }
}


#[cfg(test)]
mod test {
    use std;
    use quickcheck as qc;
    use env_logger;
    use super::Ctz;

    #[test]
    fn test_ctz_usize() {
        let _ = env_logger::init();
        fn prop(i: usize) -> bool {
            let z = i.ctz() as usize;
            let is_full = z == std::mem::size_of::<usize>() * 8;
            let lo_mask = if is_full { !0 } else { ((1 << z) - 1) };
            let hi_mask = !lo_mask;
            let next_bit_ok = if is_full { true } else { ((1 << z) & i) != 0 };
            next_bit_ok
            && (lo_mask & i == 0)
            && (hi_mask == 0 || (hi_mask & i != 0))
        }
        qc::quickcheck(prop as fn(usize) -> bool);
    }
}

