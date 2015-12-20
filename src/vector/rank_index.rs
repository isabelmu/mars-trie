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

#[derive(Clone, Copy, Debug)]
pub struct RankIndex {
    abs_: u32,
    rel_lo_: u32,
    rel_hi_: u32,
}

impl Default for RankIndex {
    fn default() -> RankIndex { RankIndex::new() }
}

impl RankIndex {
    pub fn new() -> RankIndex {
        RankIndex { abs_: 0, rel_lo_: 0, rel_hi_: 0 }
    }

    pub fn set_abs(&mut self, value: u32) {
        self.abs_ = value;
    }
    pub fn set_rel1(&mut self, value: u32) {
        assert!(value <= 64, "MARISA_RANGE_ERROR");
        self.rel_lo_ = (self.rel_lo_ & !0x7F) | (value & 0x7F);
    }
    pub fn set_rel2(&mut self, value: u32) {
        assert!(value <= 128, "MARISA_RANGE_ERROR");
        self.rel_lo_ = (self.rel_lo_ & !(0xFF << 7))
                     | ((value & 0xFF).wrapping_shl(7));
    }
    pub fn set_rel3(&mut self, value: u32) {
        assert!(value <= 192, "MARISA_RANGE_ERROR");
        self.rel_lo_ = (self.rel_lo_ & !(0xFF << 15))
                     | ((value & 0xFF).wrapping_shl(15));
    }
    pub fn set_rel4(&mut self, value: u32) {
        assert!(value <= 256, "MARISA_RANGE_ERROR");
        self.rel_lo_ = (self.rel_lo_ & !(0x1FF << 23))
                     | ((value & 0x1FF).wrapping_shl(23));
    }
    pub fn set_rel5(&mut self, value: u32) {
        assert!(value <= 320, "MARISA_RANGE_ERROR");
        self.rel_hi_ = (self.rel_hi_ & !0x1FF)
                     | (value & 0x1FF);
    }
    pub fn set_rel6(&mut self, value: u32) {
        assert!(value <= 384, "MARISA_RANGE_ERROR");
        self.rel_hi_ = (self.rel_hi_ & !(0x1FF << 9))
                     | ((value & 0x1FF).wrapping_shl(9));
    }
    pub fn set_rel7(&mut self, value: u32) {
        assert!(value <= 448, "MARISA_RANGE_ERROR");
        self.rel_hi_ = (self.rel_hi_ & !(0x1FF << 18))
                     | ((value & 0x1FF).wrapping_shl(18));
    }

    pub fn abs(&self) -> u32 {
        self.abs_
    }
    pub fn rel1(&self) -> u32 {
        self.rel_lo_ & 0x7F
    }
    pub fn rel2(&self) -> u32 {
        (self.rel_lo_ >> 7) & 0xFF
    }
    pub fn rel3(&self) -> u32 {
        (self.rel_lo_ >> 15) & 0xFF
    }
    pub fn rel4(&self) -> u32 {
        (self.rel_lo_ >> 23) & 0x1FF
    }
    pub fn rel5(&self) -> u32 {
        self.rel_hi_ & 0x1FF
    }
    pub fn rel6(&self) -> u32 {
        (self.rel_hi_ >> 9) & 0x1FF
    }
    pub fn rel7(&self) -> u32 {
        (self.rel_hi_ >> 18) & 0x1FF
    }
}

mod test {
    use super::RankIndex;

    #[test]
    fn test_rank_index() {
        let mut rank: RankIndex = Default::default();

        assert!(rank.abs() == 0);
        assert!(rank.rel1() == 0);
        assert!(rank.rel2() == 0);
        assert!(rank.rel3() == 0);
        assert!(rank.rel4() == 0);
        assert!(rank.rel5() == 0);
        assert!(rank.rel6() == 0);
        assert!(rank.rel7() == 0);

        rank.set_abs(10000);
        rank.set_rel1(64);
        rank.set_rel2(128);
        rank.set_rel3(192);
        rank.set_rel4(256);
        rank.set_rel5(320);
        rank.set_rel6(384);
        rank.set_rel7(448);

        assert!(rank.abs() == 10000);
        assert!(rank.rel1() == 64);
        assert!(rank.rel2() == 128);
        assert!(rank.rel3() == 192);
        assert!(rank.rel4() == 256);
        assert!(rank.rel5() == 320);
        assert!(rank.rel6() == 384);
        assert!(rank.rel7() == 448);
    }
}

