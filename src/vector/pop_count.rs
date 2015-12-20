// Copyright (c) 2010-2013, Susumu Yata
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

// NOTE: This should only be used when partial counts are needed instead of
//       or in addition to the full pop-count. For full counts alone, Rust's
//       count_ones function can be used. It will call the appropriate LLVM
//       intrinsic.

pub struct PopCount {
    value_: usize
}

#[cfg(target_pointer_width = "64")]
impl PopCount {
    pub fn new(mut x: usize) -> PopCount {
        x = (x & 0x5555555555555555usize)
            + ((x & 0xAAAAAAAAAAAAAAAAusize).wrapping_shr(1));
        x = (x & 0x3333333333333333usize)
            + ((x & 0xCCCCCCCCCCCCCCCCusize).wrapping_shr(2));
        x = (x & 0x0F0F0F0F0F0F0F0Fusize)
            + ((x & 0xF0F0F0F0F0F0F0F0usize).wrapping_shr(4));
        x = x.wrapping_mul(0x0101010101010101usize);
        PopCount { value_: x }
    }

    pub fn lo8(&self) -> usize {
        self.value_ & 0xFF
    }
    pub fn lo16(&self) -> usize {
        self.value_.wrapping_shr(8) & 0xFF
    }
    pub fn lo24(&self) -> usize {
        self.value_.wrapping_shr(16) & 0xFF
    }
    pub fn lo32(&self) -> usize {
        self.value_.wrapping_shr(24) & 0xFF
    }
    pub fn lo40(&self) -> usize {
        self.value_.wrapping_shr(32) & 0xFF
    }
    pub fn lo48(&self) -> usize {
        self.value_.wrapping_shr(40) & 0xFF
    }
    pub fn lo56(&self) -> usize {
        self.value_.wrapping_shr(48) & 0xFF
    }
    pub fn lo64(&self) -> usize {
        self.value_.wrapping_shr(56)
    }
}

#[cfg(target_pointer_width = "32")]
impl PopCount {
    pub fn new(mut x: usize) -> PopCount {
        x = (x & 0x55555555) + (x & 0xAAAAAAAA).wrapping_shr(1);
        x = (x & 0x33333333) + (x & 0xCCCCCCCC).wrapping_shr(2);
        x = (x & 0x0F0F0F0F) + (x & 0xF0F0F0F0).wrapping_shr(4);
        x = x.wrapping_mul(0x01010101);
        PopCount { value_: x }
    }

    pub fn lo8(&self) -> usize {
        self.value_ & 0xFF
    }
    pub fn lo16(&self) -> usize {
        self.value_.wrapping_shr(8) & 0xFF
    }
    pub fn lo24(&self) -> usize {
        self.value_.wrapping_shr(16) & 0xFF
    }
    pub fn lo32(&self) -> usize {
        self.value_.wrapping_shr(24)
    }
}

#[cfg(test)]
mod test {
    use super::PopCount;

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_pop_count_64() {
        let count = PopCount::new(0);
        assert!(count.lo8() == 0);
        assert!(count.lo16() == 0);
        assert!(count.lo24() == 0);
        assert!(count.lo32() == 0);
        assert!(count.lo40() == 0);
        assert!(count.lo48() == 0);
        assert!(count.lo56() == 0);
        assert!(count.lo64() == 0);

        let count = PopCount::new(0xFFFFFFFFFFFFFFFFusize);
        assert!(count.lo8() == 8);
        assert!(count.lo16() == 16);
        assert!(count.lo24() == 24);
        assert!(count.lo32() == 32);
        assert!(count.lo40() == 40);
        assert!(count.lo48() == 48);
        assert!(count.lo56() == 56);
        assert!(count.lo64() == 64);

        let count = PopCount::new(0xFF7F3F1F0F070301usize);
        assert!(count.lo8() == 1);
        assert!(count.lo16() == 3);
        assert!(count.lo24() == 6);
        assert!(count.lo32() == 10);
        assert!(count.lo40() == 15);
        assert!(count.lo48() == 21);
        assert!(count.lo56() == 28);
        assert!(count.lo64() == 36);
    }

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_pop_count_32() {
        let count = PopCount::new(0);
        assert!(count.lo8() == 0);
        assert!(count.lo16() == 0);
        assert!(count.lo24() == 0);
        assert!(count.lo32() == 0);

        let count = PopCount::new(0xFFFFFFFFusize);
        assert!(count.lo8() == 8);
        assert!(count.lo16() == 16);
        assert!(count.lo24() == 24);
        assert!(count.lo32() == 32);

        let count = PopCount::new(0xFF3F0F03usize);
        assert!(count.lo8() == 2);
        assert!(count.lo16() == 6);
        assert!(count.lo24() == 12);
        assert!(count.lo32() == 20);
    }
}

