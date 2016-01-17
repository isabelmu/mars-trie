use std;
use std::mem;
use base::WORD_SIZE;
use super::intrinsic::Ctz;
use super::rank_index::RankIndex;
use super::util::vec_resize;

#[derive(Clone, Debug)]
pub struct BitVec {
    units_: Vec<usize>,
    size_: usize,
    num_1s_: usize,
    ranks_: Vec<RankIndex>,
    select0s_: Vec<u32>,
    select1s_: Vec<u32>,
}

impl PartialEq for BitVec {
    fn eq(&self, other: &BitVec) -> bool {
        self.size_ == other.size_
        && self.num_1s_ == other.num_1s_
        && self.units_ == other.units_
    }
}
impl Eq for BitVec {}

impl BitVec {
    pub fn new() -> BitVec {
        BitVec { 
            units_: Default::default(),
            size_: 0,
            num_1s_: 0,
            ranks_: Default::default(),
            select0s_: Default::default(),
            select1s_: Default::default()
        }
    }
    pub fn from_words<'a, T>(x: T, bits: usize) -> BitVec
      where T: IntoIterator<Item=&'a usize> {
        let mut bv = BitVec::new();
        bv.units_ = x.into_iter().map(|x| *x).collect();
        if (bits + WORD_SIZE - 1) / WORD_SIZE != bv.units_.len() { panic!() }
        bv.size_ = bits;
        let part = bv.size_ % WORD_SIZE;
        if 0 != part {
            let mask = (1 << part) - 1;
            let last = bv.units_[bv.size_ / WORD_SIZE];
            if last & mask != last { panic!() }
        }
        for word in &bv.units_ {
            bv.num_1s_ += word.count_ones() as usize;
        }
        bv
    }

    pub fn is_empty(&self) -> bool {
        self.size_ == 0
    }
    pub fn len(&self) -> usize {
        self.size_
    }

    pub fn num_0s(&self) -> usize {
        self.size_ - self.num_1s_
    }
    pub fn num_1s(&self) -> usize {
        self.num_1s_
    }

    pub fn at(&self, i: usize) -> bool {
        assert!(i < self.size_, "MARISA_BOUND_ERROR");
        (self.units_[i / WORD_SIZE] & (1usize << (i % WORD_SIZE))) != 0
    }

    pub fn is_select0_enabled(&self) -> bool {
        !self.select0s_.is_empty()
    }

    pub fn is_select1_enabled(&self) -> bool {
        !self.select1s_.is_empty()
    }

    pub fn is_rank_enabled(&self) -> bool {
        !self.ranks_.is_empty()
    }

    pub fn build(&mut self, enables_select0: bool, enables_select1: bool) {

        let mut old = BitVec::new();
        mem::swap(self, &mut old);
        let old = old;

        let ranks_size = (old.len() / 512)
                       + (if old.len() % 512 != 0 { 1 } else { 0 })
                       + 1;

        vec_resize(&mut self.ranks_, ranks_size);

        let mut num_0s: usize = 0;
        let mut num_1s: usize = 0;
  
        assert!(old.len() <= std::u32::MAX as usize);

        for i in 0..old.len() {
            if i % 64 == 0 {
                let rank_id: usize = i / 512;
                let nu = num_1s as u32 - self.ranks_[rank_id].abs();
                match (i / 64) % 8 {
                    0 => { self.ranks_[rank_id].set_abs(num_1s as u32); },
                    1 => { self.ranks_[rank_id].set_rel1(nu); },
                    2 => { self.ranks_[rank_id].set_rel2(nu); },
                    3 => { self.ranks_[rank_id].set_rel3(nu); },
                    4 => { self.ranks_[rank_id].set_rel4(nu); },
                    5 => { self.ranks_[rank_id].set_rel5(nu); },
                    6 => { self.ranks_[rank_id].set_rel6(nu); },
                    7 => { self.ranks_[rank_id].set_rel7(nu); },
                    _ => { panic!(); }
                }
            }
  
            if old.at(i) {
                if enables_select1 && (num_1s % 512 == 0) {
                    self.select1s_.push(i as u32);
                }
                num_1s += 1;
            } else {
                if enables_select0 && (num_0s % 512 == 0) {
                    self.select0s_.push(i as u32);
                }
                num_0s += 1;
            }
        }
  
        if old.len() % 512 != 0 {
            let rank_id = (old.len() - 1) / 512;
            let nu = num_1s as u32 - self.ranks_[rank_id].abs();
            match_fallthrough!(
                ((old.len() - 1) / 64) % 8,
            {
                0 => { self.ranks_[rank_id].set_rel1(nu); },
                1 => { self.ranks_[rank_id].set_rel2(nu); },
                2 => { self.ranks_[rank_id].set_rel3(nu); },
                3 => { self.ranks_[rank_id].set_rel4(nu); },
                4 => { self.ranks_[rank_id].set_rel5(nu); },
                5 => { self.ranks_[rank_id].set_rel6(nu); },
                6 => { self.ranks_[rank_id].set_rel7(nu);
                       break;
                     },
                _ => { panic!(); }
            });
        }

        self.size_ = old.len();
        self.num_1s_ = old.num_1s();

        self.ranks_.last_mut().unwrap().set_abs(num_1s as u32);
        if enables_select0 {
            self.select0s_.push(old.len() as u32);
            self.select0s_.shrink_to_fit();
        }
        if enables_select1 {
            self.select1s_.push(old.len() as u32);
            self.select1s_.shrink_to_fit();
        }

        let mut old = old;
        mem::swap(&mut self.units_, &mut old.units_);
        self.units_.shrink_to_fit();
    }

    pub fn disable_select0(&mut self) {
        self.select0s_.clear();
    }
    pub fn disable_select1(&mut self) {
        self.select1s_.clear();
    }

    pub fn push(&mut self, bit: bool) {
        assert!(self.size_ < std::u32::MAX as usize);

        if self.size_ == WORD_SIZE * self.units_.len() {
            let newSize = self.units_.len() + (64 / WORD_SIZE);
            vec_resize(&mut self.units_, newSize);
        }
        if bit {
            self.units_[self.size_ / WORD_SIZE] |=
                1usize << (self.size_ % WORD_SIZE);
            self.num_1s_ += 1;
        }
        self.size_ += 1;
    }

    pub fn clear(&mut self) {
        *self = BitVec::new();
    }

    pub fn rank0(&self, i: usize) -> usize {
        assert!(self.is_rank_enabled(),
                "rank0 was called, but ranks are not enabled");
        assert!(i <= self.size_, "MARISA_BOUND_ERROR");
        return i - self.rank1(i);
    }

    pub fn rank1(&self, i: usize) -> usize {
        assert!(self.is_rank_enabled(),
                "rank1 was called, but ranks are not enabled");
        assert!(i <= self.size_, "MARISA_BOUND_ERROR");

        // FIXME: looks like Index is returning a value instead of an address..
        //        what am I doing wrong?
        assert!(i / 512 < self.ranks_.len());
        let rank = self.ranks_[i / 512];
        let mut offset: usize = rank.abs() as usize;
        match (i / 64) % 8 {
            0 => {}
            1 => { offset += rank.rel1() as usize; }
            2 => { offset += rank.rel2() as usize; }
            3 => { offset += rank.rel3() as usize; }
            4 => { offset += rank.rel4() as usize; }
            5 => { offset += rank.rel5() as usize; }
            6 => { offset += rank.rel6() as usize; }
            7 => { offset += rank.rel7() as usize; }
            _ => { panic!() }
        }
        offset += self.rank1_offset_rest(i);
        offset
    }

    #[cfg(target_pointer_width = "64")]
    fn rank1_offset_rest(&self, i: usize) -> usize {
        assert!(i / 64 < self.units_.len());
        (self.units_[i / 64] & (((1u64.wrapping_shl((i % 64) as u32) as u64) - 1)) as usize)
            .count_ones() as usize
    }
    #[cfg(target_pointer_width = "32")]
    fn rank1_offset_rest(&self, i: usize) -> usize {
        let mut rest: usize = 0;
        if ((i / 32) & 1) == 1 {
          rest += (self.units_[(i / 32) - 1]).count_ones() as usize;
        }
        rest += (self.units_[i / 32] & ((1 << (i % 32)) - 1)).count_ones()
                as usize;
        rest
    }

    #[cfg(target_pointer_width = "64")]
    pub fn select0(&self, mut i: usize) -> usize {
        assert!(self.is_select0_enabled(),
                "select0 was called, but select0 is not enabled");
        assert!(i < self.num_0s(), "MARISA_BOUND_ERROR");

        let select_id: usize = i / 512;
        assert!((select_id + 1) < self.select0s_.len(), "MARISA_BOUND_ERROR");
        if i % 512 == 0 {
            return self.select0s_[select_id] as usize;
        }
        let mut begin = (self.select0s_[select_id] as usize) / 512;
        let mut end = ((self.select0s_[select_id + 1] as usize) + 511) / 512;
        if begin + 10 >= end {
            while i >= (begin + 1) * 512
                       - (self.ranks_[begin + 1].abs() as usize)
            {
                begin += 1;
            }
        } else {
            while begin + 1 < end {
                let middle = (begin + end) / 2;
                if i < (middle * 512) - (self.ranks_[middle].abs() as usize) {
                    end = middle;
                } else {
                    begin = middle;
                }
            }
        }
        let rank_id: usize = begin;
        i -= (rank_id * 512) - (self.ranks_[rank_id].abs() as usize);
    
        let rank = self.ranks_[rank_id];
        let mut unit_id = rank_id * 8;
        if i < (256 - (rank.rel4() as usize)) {
            if i < (128 - (rank.rel2() as usize)) {
                if i >= (64 - (rank.rel1() as usize)) {
                    unit_id += 1;
                    i -= 64 - (rank.rel1() as usize);
                }
            } else if i < (192 - (rank.rel3() as usize)) {
                unit_id += 2;
                i -= 128 - (rank.rel2() as usize);
            } else {
                unit_id += 3;
                i -= 192 - (rank.rel3() as usize);
            }
        } else if i < (384 - (rank.rel6() as usize)) {
            if i < (320 - (rank.rel5() as usize)) {
                unit_id += 4;
                i -= 256 - (rank.rel4() as usize);
            } else {
                unit_id += 5;
                i -= 320 - (rank.rel5() as usize);
            }
        } else if i < (448 - (rank.rel7() as usize)) {
            unit_id += 6;
            i -= 384 - (rank.rel6() as usize);
        } else {
            unit_id += 7;
            i -= 448 - (rank.rel7() as usize);
        }
        return self.select_bit(i, unit_id.wrapping_mul(64),
                               !self.units_[unit_id]);
    }

    #[cfg(target_pointer_width = "32")]
    pub fn select0(&self, mut i: usize) -> usize {
        assert!(self.is_select0_enabled(),
                "select0 was called, but select0 is not enabled");
        assert!(i < self.num_0s(), "MARISA_BOUND_ERROR");
    
        let select_id: usize = i / 512;
        assert!((select_id + 1) < self.select0s_.len(), "MARISA_BOUND_ERROR");
        if (i % 512) == 0 {
            return self.select0s_[select_id];
        }
        let mut begin: usize = self.select0s_[select_id] / 512;
        let mut end: usize = (self.select0s_[select_id + 1] + 511) / 512;
        if begin + 10 >= end {
            while i >= ((begin + 1) * 512) - self.ranks_[begin + 1].abs() {
                begin += 1;
            }
        } else {
            while begin + 1 < end {
                let middle: usize = (begin + end) / 2;
                if i < (middle * 512) - self.ranks_[middle].abs() {
                    end = middle;
                } else {
                    begin = middle;
                }
            }
        }
        let rank_id: usize = begin;
        i -= (rank_id * 512) - self.ranks_[rank_id].abs();
    
        let rank: &RankIndex = self.ranks_[rank_id];
        let mut unit_id: usize = rank_id * 16;
        if i < (256 - rank.rel4()) {
            if i < (128 - rank.rel2()) {
                if i >= (64 - rank.rel1()) {
                  unit_id += 2;
                  i -= 64 - rank.rel1();
                }
            } else if i < (192 - rank.rel3()) {
                unit_id += 4;
                i -= 128 - rank.rel2();
            } else {
                unit_id += 6;
                i -= 192 - rank.rel3();
            }
        } else if i < (384 - rank.rel6()) {
            if i < (320 - rank.rel5()) {
                unit_id += 8;
                i -= 256 - rank.rel4();
            } else {
                unit_id += 10;
                i -= 320 - rank.rel5();
            }
        } else if i < (448 - rank.rel7()) {
            unit_id += 12;
            i -= 384 - rank.rel6();
        } else {
            unit_id += 14;
            i -= 448 - rank.rel7();
        }

//    #ifdef MARISA_USE_SSE2
//        return select_bit(
//            i, unit_id * 32, ~units_[unit_id], ~units_[unit_id + 1]);
//    #else  // MARISA_USE_SSE2
        let mut unit: u32 = !self.units_[unit_id];
        let count = PopCount::new(unit);
        if i >= count.lo32() {
            unit_id += 1;
            i -= count.lo32();
            unit = !self.units_[unit_id];
            count = PopCount::new(unit);
        }

        let mut bit_id: usize = unit_id * 32;
        if i < count.lo16() {
            if i >= count.lo8() {
                bit_id += 8;
                unit >>= 8;
                i -= count.lo8();
            }
        } else if i < count.lo24() {
            bit_id += 16;
            unit >>= 16;
            i -= count.lo16();
        } else {
            bit_id += 24;
            unit >>= 24;
            i -= count.lo24();
        }
        return bit_id + SELECT_TABLE[i][unit & 0xFF];
//#endif  // MARISA_USE_SSE2
    }

    #[cfg(target_pointer_width = "64")]
    pub fn select1(&self, mut i: usize) -> usize {
        assert!(self.is_select1_enabled(),
                "select1 was called, but select1 is not enabled");
        assert!(i < self.num_1s(), "MARISA_BOUND_ERROR");

        let select_id: usize = i / 512;
        assert!((select_id + 1) < self.select1s_.len(), "MARISA_BOUND_ERROR");
        if (i % 512) == 0 {
            return self.select1s_[select_id] as usize;
        }
        let mut begin: usize = (self.select1s_[select_id] as usize) / 512;
        let mut end: usize = ((self.select1s_[select_id + 1] as usize)+ 511)
                             / 512;
        if begin + 10 >= end {
            while i >= self.ranks_[begin + 1].abs() as usize {
                begin += 1;
            }
        } else {
            while begin + 1 < end {
                let middle: usize = (begin + end) / 2;
                if i < self.ranks_[middle].abs() as usize {
                    end = middle;
                } else {
                    begin = middle;
                }
            }
        }
        let rank_id: usize = begin;
        i -= self.ranks_[rank_id].abs() as usize;

        //const RankIndex &rank = 
        let rank = self.ranks_[rank_id];
        let mut unit_id: usize = rank_id * 8;
        if i < rank.rel4() as usize {
            if i < rank.rel2() as usize {
                if i >= rank.rel1() as usize {
                    unit_id += 1;
                    i -= rank.rel1() as usize;
                }
            } else if i < rank.rel3() as usize {
                unit_id += 2;
                i -= rank.rel2() as usize;
            } else {
                unit_id += 3;
                i -= rank.rel3() as usize;
            }
        } else if i < rank.rel6() as usize {
            if i < rank.rel5() as usize {
                unit_id += 4;
                i -= rank.rel4() as usize;
            } else {
                unit_id += 5;
                i -= rank.rel5() as usize;
            }
        } else if i < rank.rel7() as usize {
            unit_id += 6;
            i -= rank.rel6() as usize;
        } else {
            unit_id += 7;
            i -= rank.rel7() as usize;
        }
        return self.select_bit(i, unit_id * 64, self.units_[unit_id]);
    }

    #[cfg(target_pointer_width = "32")]
    pub fn select1(&self, mut i: usize) -> usize {
        assert!(self.is_select1_enabled(),
                "select1 was called, but select1 is not enabled");
        assert!(i < num_1s(), "MARISA_BOUND_ERROR");
    
        let select_id: usize = i / 512;
        assert!((select_id + 1) < self.select1s_.len(), "MARISA_BOUND_ERROR");
        if (i % 512) == 0 {
            return self.select1s_[select_id];
        }
        let mut begin: usize = self.select1s_[select_id] / 512;
        let mut end: usize = (self.select1s_[select_id + 1] + 511) / 512;
        if begin + 10 >= end {
            while i >= ranks_[begin + 1].abs() {
                begin += 1;
            }
        } else {
            while (begin + 1 < end) {
                let middle: usize = (begin + end) / 2;
                if i < ranks_[middle].abs() {
                    end = middle;
                } else {
                    begin = middle;
                }
            }
        }
        let rank_id: usize = begin;
        i -= self.ranks_[rank_id].abs();

        //const RankIndex &rank = ranks_[rank_id];
        let rank = self.ranks_[rank_id];
        let unit_id: usize = rank_id * 16;
        if i < rank.rel4() {
            if i < rank.rel2() {
                if i >= rank.rel1() {
                    unit_id += 2;
                    i -= rank.rel1();
                }
            } else if i < rank.rel3() {
                unit_id += 4;
                i -= rank.rel2();
            } else {
                unit_id += 6;
                i -= rank.rel3();
            }
        } else if i < rank.rel6() {
            if i < rank.rel5() {
                unit_id += 8;
                i -= rank.rel4();
            } else {
                unit_id += 10;
                i -= rank.rel5();
            }
        } else if i < rank.rel7() {
            unit_id += 12;
            i -= rank.rel6();
        } else {
            unit_id += 14;
            i -= rank.rel7();
        }

//#ifdef MARISA_USE_SSE2
//        return select_bit(i, unit_id * 32, units_[unit_id], units_[unit_id + 1]);
//#else  // MARISA_USE_SSE2
        let mut unit: usize = units_[unit_id];
        let count = PopCount::new(unit);
        if i >= count.lo32() {
            unit_id += 1;
            i -= count.lo32();
            unit = units_[unit_id];
            count = PopCount(unit);
        }

        let bit_id: usize = unit_id * 32;
        if i < count.lo16() {
            if i >= count.lo8() {
                bit_id += 8;
                unit >>= 8;
                i -= count.lo8();
            }
        } else if i < count.lo24() {
            bit_id += 16;
            unit >>= 16;
            i -= count.lo16();
        } else {
            bit_id += 24;
            unit >>= 24;
            i -= count.lo24();
        }
        return bit_id + SELECT_TABLE[i][unit & 0xFF];
//#endif // MARISA_USE_SSE2
    }

    #[cfg(target_pointer_width = "64")]
    fn select_bit(&self, mut i: usize, mut bit_id: usize, mut unit: usize)
                  -> usize {
        let MASK_55: usize = 0x5555555555555555usize;
        let MASK_33: usize = 0x3333333333333333usize;
        let MASK_0F: usize = 0x0F0F0F0F0F0F0F0Fusize;
        let MASK_01: usize = 0x0101010101010101usize;
        let MASK_80: usize = 0x8080808080808080usize;

        let mut counts: usize;
        {
//#if defined(MARISA_X64) && defined(MARISA_USE_SSSE3)
//        __m128i lower_nibbles = _mm_cvtsi64_si128(unit & 0x0F0F0F0F0F0F0F0FULL);
//        __m128i upper_nibbles = _mm_cvtsi64_si128(unit & 0xF0F0F0F0F0F0F0F0ULL);
//        upper_nibbles = _mm_srli_epi32(upper_nibbles, 4);
//    
//        __m128i lower_counts =
//            _mm_set_epi8(4, 3, 3, 2, 3, 2, 2, 1, 3, 2, 2, 1, 2, 1, 1, 0);
//        lower_counts = _mm_shuffle_epi8(lower_counts, lower_nibbles);
//        __m128i upper_counts =
//            _mm_set_epi8(4, 3, 3, 2, 3, 2, 2, 1, 3, 2, 2, 1, 2, 1, 1, 0);
//        upper_counts = _mm_shuffle_epi8(upper_counts, upper_nibbles);
//    
//        counts = _mm_cvtsi128_si64(_mm_add_epi8(lower_counts, upper_counts));
//#else  // defined(MARISA_X64) && defined(MARISA_USE_SSSE3)
            counts = unit - (unit.wrapping_shr(1) & MASK_55);
            counts = (counts & MASK_33) + (counts.wrapping_shr(2) & MASK_33);
            counts = (counts + (counts.wrapping_shr(4))) & MASK_0F;
//#endif  // defined(MARISA_X64) && defined(MARISA_USE_SSSE3)
            counts = counts.wrapping_mul(MASK_01);
        }

//#if defined(MARISA_X64) && defined(MARISA_USE_POPCNT)
//        UInt8 skip;
//        {
//            __m128i x = _mm_cvtsi64_si128((i + 1) * MASK_01);
//            __m128i y = _mm_cvtsi64_si128(counts);
//            x = _mm_cmpgt_epi8(x, y);
//            skip = (UInt8)PopCount::count(_mm_cvtsi128_si64(x));
//        }
//#else  // defined(MARISA_X64) && defined(MARISA_USE_POPCNT)
        let x: usize = (counts | MASK_80) - (i + 1).wrapping_mul(MASK_01);
//#  ifdef _MSC_VER
//      unsigned long skip;
//      ::_BitScanForward64(&skip, (x & MASK_80) >> 7);
//      --skip;
//#  else  // _MSC_VER
        // ctz: count trailing zeros (aka tzcnt, bsf)
        let skip: u32 =  //::__builtin_ctzll((x & MASK_80) >> 7);
            (x & MASK_80).wrapping_shr(7).ctz();
//#  endif  // _MSC_VER
//#endif  // defined(MARISA_X64) && defined(MARISA_USE_POPCNT)
    
        bit_id += skip as usize;
        unit = unit.wrapping_shr(skip);
        i = i.wrapping_sub(counts.wrapping_shl(8).wrapping_shr(skip) & 0xFF);

        return bit_id + (SELECT_TABLE[i][unit & 0xFF] as usize);
    }

/*
 #ifdef MARISA_USE_SSE2
const UInt8 POPCNT_TABLE[256] = {
   0,  8,  8, 16,  8, 16, 16, 24,  8, 16, 16, 24, 16, 24, 24, 32,
   8, 16, 16, 24, 16, 24, 24, 32, 16, 24, 24, 32, 24, 32, 32, 40,
   8, 16, 16, 24, 16, 24, 24, 32, 16, 24, 24, 32, 24, 32, 32, 40,
  16, 24, 24, 32, 24, 32, 32, 40, 24, 32, 32, 40, 32, 40, 40, 48,
   8, 16, 16, 24, 16, 24, 24, 32, 16, 24, 24, 32, 24, 32, 32, 40,
  16, 24, 24, 32, 24, 32, 32, 40, 24, 32, 32, 40, 32, 40, 40, 48,
  16, 24, 24, 32, 24, 32, 32, 40, 24, 32, 32, 40, 32, 40, 40, 48,
  24, 32, 32, 40, 32, 40, 40, 48, 32, 40, 40, 48, 40, 48, 48, 56,
   8, 16, 16, 24, 16, 24, 24, 32, 16, 24, 24, 32, 24, 32, 32, 40,
  16, 24, 24, 32, 24, 32, 32, 40, 24, 32, 32, 40, 32, 40, 40, 48,
  16, 24, 24, 32, 24, 32, 32, 40, 24, 32, 32, 40, 32, 40, 40, 48,
  24, 32, 32, 40, 32, 40, 40, 48, 32, 40, 40, 48, 40, 48, 48, 56,
  16, 24, 24, 32, 24, 32, 32, 40, 24, 32, 32, 40, 32, 40, 40, 48,
  24, 32, 32, 40, 32, 40, 40, 48, 32, 40, 40, 48, 40, 48, 48, 56,
  24, 32, 32, 40, 32, 40, 40, 48, 32, 40, 40, 48, 40, 48, 48, 56,
  32, 40, 40, 48, 40, 48, 48, 56, 40, 48, 48, 56, 48, 56, 56, 64
};

std::size_t select_bit(std::size_t i, std::size_t bit_id,
    UInt32 unit_lo, UInt32 unit_hi) {
  __m128i unit;
  {
    __m128i lower_dword = _mm_cvtsi32_si128(unit_lo);
    __m128i upper_dword = _mm_cvtsi32_si128(unit_hi);
    upper_dword = _mm_slli_si128(upper_dword, 4);
    unit = _mm_or_si128(lower_dword, upper_dword);
  }

  __m128i counts;
  {
  #ifdef MARISA_USE_SSSE3
    __m128i lower_nibbles = _mm_set1_epi8(0x0F);
    lower_nibbles = _mm_and_si128(lower_nibbles, unit);
    __m128i upper_nibbles = _mm_set1_epi8((UInt8)0xF0);
    upper_nibbles = _mm_and_si128(upper_nibbles, unit);
    upper_nibbles = _mm_srli_epi32(upper_nibbles, 4);

    __m128i lower_counts =
        _mm_set_epi8(4, 3, 3, 2, 3, 2, 2, 1, 3, 2, 2, 1, 2, 1, 1, 0);
    lower_counts = _mm_shuffle_epi8(lower_counts, lower_nibbles);
    __m128i upper_counts =
        _mm_set_epi8(4, 3, 3, 2, 3, 2, 2, 1, 3, 2, 2, 1, 2, 1, 1, 0);
    upper_counts = _mm_shuffle_epi8(upper_counts, upper_nibbles);

    counts = _mm_add_epi8(lower_counts, upper_counts);
  #else  // MARISA_USE_SSSE3
    __m128i x = _mm_srli_epi32(unit, 1);
    x = _mm_and_si128(x, _mm_set1_epi8(0x55));
    x = _mm_sub_epi8(unit, x);

    __m128i y = _mm_srli_epi32(x, 2);
    y = _mm_and_si128(y, _mm_set1_epi8(0x33));
    x = _mm_and_si128(x, _mm_set1_epi8(0x33));
    x = _mm_add_epi8(x, y);

    y = _mm_srli_epi32(x, 4);
    x = _mm_add_epi8(x, y);
    counts = _mm_and_si128(x, _mm_set1_epi8(0x0F));
  #endif  // MARISA_USE_SSSE3
  }

  __m128i accumulated_counts;
  {
    __m128i x = counts;
    x = _mm_slli_si128(x, 1);
    __m128i y = counts;
    y = _mm_add_epi32(y, x);

    x = y;
    y = _mm_slli_si128(y, 2);
    x = _mm_add_epi32(x, y);

    y = x;
    x = _mm_slli_si128(x, 4);
    y = _mm_add_epi32(y, x);

    accumulated_counts = _mm_set_epi32(0x7F7F7F7FU, 0x7F7F7F7FU, 0, 0);
    accumulated_counts = _mm_or_si128(accumulated_counts, y);
  }

  UInt8 skip;
  {
    __m128i x = _mm_set1_epi8((UInt8)(i + 1));
    x = _mm_cmpgt_epi8(x, accumulated_counts);
    skip = POPCNT_TABLE[_mm_movemask_epi8(x)];
  }

  UInt8 byte;
  {
  #ifdef _MSC_VER
    __declspec(align(16)) UInt8 unit_bytes[16];
    __declspec(align(16)) UInt8 accumulated_counts_bytes[16];
  #else  // _MSC_VER
    UInt8 unit_bytes[16] __attribute__ ((aligned (16)));
    UInt8 accumulated_counts_bytes[16] __attribute__ ((aligned (16)));
  #endif  // _MSC_VER
    accumulated_counts = _mm_slli_si128(accumulated_counts, 1);
    _mm_store_si128(reinterpret_cast<__m128i *>(unit_bytes), unit);
    _mm_store_si128(reinterpret_cast<__m128i *>(accumulated_counts_bytes),
        accumulated_counts);

    bit_id += skip;
    byte = unit_bytes[skip / 8];
    i -= accumulated_counts_bytes[skip / 8];
  }

  return bit_id + SELECT_TABLE[i][byte];
}
 #endif  // MARISA_USE_SSE2
*/




/*
    fn total_size(&self) -> usize {
        self.units_.total_size()
        + self.ranks_.total_size()
        + self.select0s_.total_size()
        + self.select1s_.total_size();
    }
    fn io_size(&self) -> usize {
        self.units_.io_size()
        + (mem::size_of::<u32>() * 2)
        + self.ranks_.io_size()
        + self.select0s_.io_size()
        + self.select1s_.io_size();
    }
*/


/*
  void map_(Mapper &mapper) {
    units_.map(mapper);
    {
      u32 temp_size;
      mapper.map(&temp_size);
      size_ = temp_size;
    }
    {
      u32 temp_num_1s;
      mapper.map(&temp_num_1s);
      MARISA_THROW_IF(temp_num_1s > size_, MARISA_FORMAT_ERROR);
      num_1s_ = temp_num_1s;
    }
    ranks_.map(mapper);
    select0s_.map(mapper);
    select1s_.map(mapper);
  }

  void read_(Reader &reader) {
    units_.read(reader);
    {
      u32 temp_size;
      reader.read(&temp_size);
      size_ = temp_size;
    }
    {
      u32 temp_num_1s;
      reader.read(&temp_num_1s);
      MARISA_THROW_IF(temp_num_1s > size_, MARISA_FORMAT_ERROR);
      num_1s_ = temp_num_1s;
    }
    ranks_.read(reader);
    select0s_.read(reader);
    select1s_.read(reader);
  }

  void write_(Writer &writer) const {
    units_.write(writer);
    writer.write((u32)size_);
    writer.write((u32)num_1s_);
    ranks_.write(writer);
    select0s_.write(writer);
    select1s_.write(writer);
  }
*/

}

const SELECT_TABLE: [[u8; 256]; 8] =
[ [ 7, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 5, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 6, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 5, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 7, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 5, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 6, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 5, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  , 4, 0, 1, 0, 2, 0, 1, 0, 3, 0, 1, 0, 2, 0, 1, 0
  ]
, [ 7, 7, 7, 1, 7, 2, 2, 1, 7, 3, 3, 1, 3, 2, 2, 1
  , 7, 4, 4, 1, 4, 2, 2, 1, 4, 3, 3, 1, 3, 2, 2, 1
  , 7, 5, 5, 1, 5, 2, 2, 1, 5, 3, 3, 1, 3, 2, 2, 1
  , 5, 4, 4, 1, 4, 2, 2, 1, 4, 3, 3, 1, 3, 2, 2, 1
  , 7, 6, 6, 1, 6, 2, 2, 1, 6, 3, 3, 1, 3, 2, 2, 1
  , 6, 4, 4, 1, 4, 2, 2, 1, 4, 3, 3, 1, 3, 2, 2, 1
  , 6, 5, 5, 1, 5, 2, 2, 1, 5, 3, 3, 1, 3, 2, 2, 1
  , 5, 4, 4, 1, 4, 2, 2, 1, 4, 3, 3, 1, 3, 2, 2, 1
  , 7, 7, 7, 1, 7, 2, 2, 1, 7, 3, 3, 1, 3, 2, 2, 1
  , 7, 4, 4, 1, 4, 2, 2, 1, 4, 3, 3, 1, 3, 2, 2, 1
  , 7, 5, 5, 1, 5, 2, 2, 1, 5, 3, 3, 1, 3, 2, 2, 1
  , 5, 4, 4, 1, 4, 2, 2, 1, 4, 3, 3, 1, 3, 2, 2, 1
  , 7, 6, 6, 1, 6, 2, 2, 1, 6, 3, 3, 1, 3, 2, 2, 1
  , 6, 4, 4, 1, 4, 2, 2, 1, 4, 3, 3, 1, 3, 2, 2, 1
  , 6, 5, 5, 1, 5, 2, 2, 1, 5, 3, 3, 1, 3, 2, 2, 1
  , 5, 4, 4, 1, 4, 2, 2, 1, 4, 3, 3, 1, 3, 2, 2, 1
  ]
, [ 7, 7, 7, 7, 7, 7, 7, 2, 7, 7, 7, 3, 7, 3, 3, 2
  , 7, 7, 7, 4, 7, 4, 4, 2, 7, 4, 4, 3, 4, 3, 3, 2
  , 7, 7, 7, 5, 7, 5, 5, 2, 7, 5, 5, 3, 5, 3, 3, 2
  , 7, 5, 5, 4, 5, 4, 4, 2, 5, 4, 4, 3, 4, 3, 3, 2
  , 7, 7, 7, 6, 7, 6, 6, 2, 7, 6, 6, 3, 6, 3, 3, 2
  , 7, 6, 6, 4, 6, 4, 4, 2, 6, 4, 4, 3, 4, 3, 3, 2
  , 7, 6, 6, 5, 6, 5, 5, 2, 6, 5, 5, 3, 5, 3, 3, 2
  , 6, 5, 5, 4, 5, 4, 4, 2, 5, 4, 4, 3, 4, 3, 3, 2
  , 7, 7, 7, 7, 7, 7, 7, 2, 7, 7, 7, 3, 7, 3, 3, 2
  , 7, 7, 7, 4, 7, 4, 4, 2, 7, 4, 4, 3, 4, 3, 3, 2
  , 7, 7, 7, 5, 7, 5, 5, 2, 7, 5, 5, 3, 5, 3, 3, 2
  , 7, 5, 5, 4, 5, 4, 4, 2, 5, 4, 4, 3, 4, 3, 3, 2
  , 7, 7, 7, 6, 7, 6, 6, 2, 7, 6, 6, 3, 6, 3, 3, 2
  , 7, 6, 6, 4, 6, 4, 4, 2, 6, 4, 4, 3, 4, 3, 3, 2
  , 7, 6, 6, 5, 6, 5, 5, 2, 6, 5, 5, 3, 5, 3, 3, 2
  , 6, 5, 5, 4, 5, 4, 4, 2, 5, 4, 4, 3, 4, 3, 3, 2
  ]
, [ 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 3
  , 7, 7, 7, 7, 7, 7, 7, 4, 7, 7, 7, 4, 7, 4, 4, 3
  , 7, 7, 7, 7, 7, 7, 7, 5, 7, 7, 7, 5, 7, 5, 5, 3
  , 7, 7, 7, 5, 7, 5, 5, 4, 7, 5, 5, 4, 5, 4, 4, 3
  , 7, 7, 7, 7, 7, 7, 7, 6, 7, 7, 7, 6, 7, 6, 6, 3
  , 7, 7, 7, 6, 7, 6, 6, 4, 7, 6, 6, 4, 6, 4, 4, 3
  , 7, 7, 7, 6, 7, 6, 6, 5, 7, 6, 6, 5, 6, 5, 5, 3
  , 7, 6, 6, 5, 6, 5, 5, 4, 6, 5, 5, 4, 5, 4, 4, 3
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 3
  , 7, 7, 7, 7, 7, 7, 7, 4, 7, 7, 7, 4, 7, 4, 4, 3
  , 7, 7, 7, 7, 7, 7, 7, 5, 7, 7, 7, 5, 7, 5, 5, 3
  , 7, 7, 7, 5, 7, 5, 5, 4, 7, 5, 5, 4, 5, 4, 4, 3
  , 7, 7, 7, 7, 7, 7, 7, 6, 7, 7, 7, 6, 7, 6, 6, 3
  , 7, 7, 7, 6, 7, 6, 6, 4, 7, 6, 6, 4, 6, 4, 4, 3
  , 7, 7, 7, 6, 7, 6, 6, 5, 7, 6, 6, 5, 6, 5, 5, 3
  , 7, 6, 6, 5, 6, 5, 5, 4, 6, 5, 5, 4, 5, 4, 4, 3
  ]
, [ 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 4
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 5
  , 7, 7, 7, 7, 7, 7, 7, 5, 7, 7, 7, 5, 7, 5, 5, 4
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6
  , 7, 7, 7, 7, 7, 7, 7, 6, 7, 7, 7, 6, 7, 6, 6, 4
  , 7, 7, 7, 7, 7, 7, 7, 6, 7, 7, 7, 6, 7, 6, 6, 5
  , 7, 7, 7, 6, 7, 6, 6, 5, 7, 6, 6, 5, 6, 5, 5, 4
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 4
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 5
  , 7, 7, 7, 7, 7, 7, 7, 5, 7, 7, 7, 5, 7, 5, 5, 4
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6
  , 7, 7, 7, 7, 7, 7, 7, 6, 7, 7, 7, 6, 7, 6, 6, 4
  , 7, 7, 7, 7, 7, 7, 7, 6, 7, 7, 7, 6, 7, 6, 6, 5
  , 7, 7, 7, 6, 7, 6, 6, 5, 7, 6, 6, 5, 6, 5, 5, 4
  ]
, [ 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 5
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6
  , 7, 7, 7, 7, 7, 7, 7, 6, 7, 7, 7, 6, 7, 6, 6, 5
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 5
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6
  , 7, 7, 7, 7, 7, 7, 7, 6, 7, 7, 7, 6, 7, 6, 6, 5
  ]
, [ 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6
  ]
, [ 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  , 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7
  ]
];

#[cfg(test)]
mod test {
    use base::WORD_SIZE;
    use env_logger;
    use quickcheck as qc;
    use std;
    use super::BitVec;

    fn build_same(old: &BitVec, new: &mut BitVec) {
        if old.is_rank_enabled() {
            new.build(old.is_select0_enabled(), old.is_select1_enabled());
        }
    }

    impl qc::Arbitrary for BitVec {
        fn arbitrary<G: qc::Gen>(g: &mut G) -> BitVec {
            let mut v = BitVec::new();
            let vec_size = { let s = g.size(); g.gen_range(0, s) };
            for _ in 0..vec_size {
                v.push(g.gen());
            }
            match g.gen_range(0, 5) {
                0 => {},
                1 => { v.build(false, false); }
                2 => { v.build(true, false); }
                3 => { v.build(false, true); }
                4 => { v.build(true, true); }
                _ => panic!()
            }
            v
        }

        fn shrink(&self) -> Box<Iterator<Item=Self>> {
            let iter = self.units_.iter();
            let mut v = Vec::new();
            match (self.size_ / WORD_SIZE, self.size_ % WORD_SIZE) {
                (0, 0) | (0, 1) => {},
                (c, p) if self.size_ <= WORD_SIZE => {
                    // split partials
                    let split = (self.size_ + 1) / 2;
                    let val = self.units_[0];
                    let mask = (1 << split) - 1;
                    let l = val & mask;
                    let r = val >> split;
                    let mut lv = BitVec::from_words(std::iter::once(&l), split);
                    let mut rv = BitVec::from_words(std::iter::once(&r),
                                                    self.size_ - split);
                    build_same(self, &mut lv);
                    build_same(self, &mut rv);
                    v.push(lv);
                    v.push(rv);
                },
                (c, p) => {
                    // separate at word-middle
                    let len = self.units_.len();
                    let split = (len + 1) / 2;
                    let iter2 = iter.clone();
                    let mut lv = BitVec::from_words(
                        iter.take(split), split * WORD_SIZE);
                    let mut rv = BitVec::from_words(
                        iter2.skip(split), (c - split) * WORD_SIZE + p);
                    build_same(self, &mut lv);
                    build_same(self, &mut rv);
                    v.push(lv);
                    v.push(rv);
                },
            }

            if self.size_ < 10 {
                let mut units = self.units_.clone();
                let mut found_non_zero = false;
                for x in units.iter_mut() {
                    if *x != 0 { found_non_zero = true; }
                    *x = *x / 2;
                }
                if found_non_zero {
                    let mut new = BitVec::from_words(units.iter(), self.size_);
                    build_same(self, &mut new);
                    v.push(new);
                }
            }

            // Shrink the built-ness as well
            match (self.is_rank_enabled(), self.is_select0_enabled(),
                   self.is_select1_enabled()) {
                (false, s0, s1) if s0 || s1 => { panic!() },
                (false, _, _) => {},
                (true, s0, s1) if s0 || s1 => {
                    if s0 {
                        let mut cpy = self.clone();
                        cpy.build(false, s1);
                        v.push(cpy);
                    }
                    if s1 {
                        let mut cpy = self.clone();
                        cpy.build(s0, false);
                        v.push(cpy);
                    }
                },
                (true, false, false) => {
                    v.push(self.clone());
                },
                // FIXME: rustc says the above patterns don't cover (t,t,_) but
                //        the (true, s0, s1) pattern should. Probably caused by
                //        the 'if' guard not being used for analysis by the
                //        compiler.
                _ => { panic!() }
            }
            Box::new(v.into_iter())
        }
    }

    #[test]
    fn can_build() {
        let _ = env_logger::init();
        fn prop(mut bv: BitVec) -> bool {
            bv.build(true, true);
            true
        }
        qc::quickcheck(prop as fn(BitVec) -> bool);
    }

    fn naive_rank0(bv: &BitVec, i: usize) -> usize {
        i - naive_rank1(bv, i)
    }
    fn naive_rank1(bv: &BitVec, i: usize) -> usize {
        let mut accum_1s: usize = 0;
        assert!(i <= bv.size_);
        let full = i / WORD_SIZE;
        for word in bv.units_.iter().take(full) {
            accum_1s += word.count_ones() as usize;
        }
        let part = i % WORD_SIZE;
        if part != 0 {
            let mask = (1 << part) - 1;
            let masked = bv.units_[full] & mask;
            accum_1s += masked.count_ones() as usize;
        }
        accum_1s
    }
    fn naive_num_0s(bv: &BitVec) -> usize {
        naive_rank0(bv, bv.size_)
    }
    fn naive_num_1s(bv: &BitVec) -> usize {
        naive_rank1(bv, bv.size_)
    }
    fn naive_select0(bv: &BitVec, i: usize) -> usize {
        naive_select(bv, false, i)
    }
    fn naive_select1(bv: &BitVec, i: usize) -> usize {
        naive_select(bv, true, i)        
    }
    fn naive_select(bv: &BitVec, bit: bool, i: usize) -> usize {
        let partial_idx = bv.size_ / WORD_SIZE;
        let mut seen = 0;
        for (idx, &word) in bv.units_.iter().enumerate() {
            let c =
              if idx == partial_idx { bv.size_ % WORD_SIZE } else { WORD_SIZE };
            assert!(c != 0);
            let ones = word.count_ones() as usize;
            let zeroes = c - ones;
            let seen_prev = seen;
            let seen_here = if bit { ones } else { zeroes };
            seen += seen_here;
            if seen > i {
                let find = if bit { 1 } else { 0 };
                let mut word = word;
                let mut countdown = i - seen_prev;
                for x in 0..c {
                    if word & 1 == find {
                        if 0 == countdown {
                            return x + idx * WORD_SIZE;
                        }
                        countdown -= 1;
                    }
                    word >>= 1;
                }
                unreachable!();
            }
        }
        bv.size_
    }

    #[test]
    fn test_num_0s_1s() {
        let _ = env_logger::init();
        fn prop(bv: BitVec) -> bool {
            let nn0 = naive_num_0s(&bv);
            let n0 = bv.num_0s();
            let nn1 = naive_num_1s(&bv);
            let n1 = bv.num_1s();
            nn0 == n0 && nn1 == n1
        }
        qc::quickcheck(prop as fn(BitVec) -> bool);
    }

    #[test]
    fn clone_is_eq() {
        let _ = env_logger::init();
        fn prop(bv: BitVec) -> bool {
            let bv2 = bv.clone();
            bv == bv2
        }
        qc::quickcheck(prop as fn(BitVec) -> bool);
    }

    #[test]
    fn from_words() {
        let _ = env_logger::init();
        fn prop(bv: BitVec) -> bool {
            let bv2 = BitVec::from_words(&bv.units_, bv.size_);
            bv == bv2
        }
        qc::quickcheck(prop as fn(BitVec) -> bool);
    }

    fn rank_prop(bv: BitVec, i: usize) -> qc::TestResult {
        if i >= bv.size_ { return qc::TestResult::discard(); }
        if !bv.is_rank_enabled() { return qc::TestResult::discard(); }
        let nr0 = naive_rank0(&bv, i);
        let r0 = bv.rank0(i);
        let nr1 = naive_rank1(&bv, i);
        let r1 = bv.rank1(i);
        qc::TestResult::from_bool(nr0 == r0 && nr1 == r1)
    }

    #[test]
    fn test_rank_qc() {
        let _ = env_logger::init();
        qc::quickcheck(rank_prop as fn(BitVec, usize) -> qc::TestResult);
    }

    #[test]
    fn test_naive_select() {
        let mut bv = BitVec::new();
        bv.push(true);
        bv.push(false);
        bv.push(false);
        bv.push(true);
        bv.build(true, false);
        assert_eq!(1, naive_select0(&bv, 0));
        assert_eq!(2, naive_select0(&bv, 1));

        let mut bv = BitVec::new();
        bv.push(false);
        bv.push(false);
        bv.build(true, false);
        assert_eq!(0, naive_select0(&bv, 0));
        assert_eq!(1, naive_select0(&bv, 1));

        let arr: [usize; 1] = [ 29151 ];
        let mut bv = BitVec::from_words(arr.iter(), 16);
        bv.build(true, true);
        assert_eq!(0, naive_select1(&bv, 0));
        assert_eq!(1, naive_select1(&bv, 1));
        assert_eq!(2, naive_select1(&bv, 2));
        assert_eq!(3, naive_select1(&bv, 3));
        assert_eq!(4, naive_select1(&bv, 4));
        assert_eq!(5, naive_select0(&bv, 0));
        assert_eq!(6, naive_select1(&bv, 5));
        assert_eq!(7, naive_select1(&bv, 6));
        assert_eq!(8, naive_select1(&bv, 7));
        assert_eq!(9, naive_select0(&bv, 1));
        assert_eq!(10, naive_select0(&bv, 2));
        assert_eq!(11, naive_select0(&bv, 3));
        assert_eq!(12, naive_select1(&bv, 8));
        assert_eq!(13, naive_select1(&bv, 9));
        assert_eq!(14, naive_select1(&bv, 10));
        assert_eq!(15, naive_select0(&bv, 4));
        assert_eq!(16, naive_select0(&bv, 5));
        assert_eq!(16, naive_select1(&bv, 11));

        let mut bv = BitVec::new();
        bv.push(false);
        bv.push(true);
        bv.push(false);
        bv.push(true);
        bv.build(false, true);
        assert_eq!(1, naive_select1(&bv, 0));
        assert_eq!(3, naive_select1(&bv, 1));
    }

    fn test_select0_qc_prop(mut bv: BitVec, i: usize) -> qc::TestResult {
        if i > bv.size_ || i >= bv.num_0s() || !bv.is_select0_enabled() {
            return qc::TestResult::discard();
        }
        let ns = naive_select0(&bv, i);
        let s = bv.select0(i);
        qc::TestResult::from_bool(ns == s)
    }

    #[test]
    fn test_select0_qc() {
        let _ = env_logger::init();
        qc::quickcheck(test_select0_qc_prop
                       as fn(BitVec, usize) -> qc::TestResult);
    }

    #[test]
    fn test_select0_manual() {
        let _ = env_logger::init();
        fn build_from_words<'a, T>(x: T, bits: usize) -> BitVec
          where T: IntoIterator<Item=&'a usize> {
            let mut bv = BitVec::from_words(x, bits);
            bv.build(true, true);
            bv
        }
        let arr = [2195usize];
        let tr = test_select0_qc_prop(build_from_words(arr.iter(), 16), 4);
        assert!(!tr.is_failure());

        let arr = [16162699108551845249usize, 7032usize];
        let tr = test_select0_qc_prop(build_from_words(arr.iter(), 78), 39);
        assert!(!tr.is_failure());
    }

    #[test]
    fn test_select1_qc() {
        let _ = env_logger::init();
        fn prop(mut bv: BitVec, i: usize) -> qc::TestResult {
            if i > bv.size_ || i >= bv.num_1s() || !bv.is_select1_enabled() {
                return qc::TestResult::discard();
            }
            let ns = naive_select1(&bv, i);
            let s = bv.select1(i);
            qc::TestResult::from_bool(ns == s)
        }
        qc::quickcheck(prop as fn(BitVec, usize) -> qc::TestResult);
    }

    fn test_bit_vector_prop(mut bv: BitVec) -> qc::TestResult {
        let _ = env_logger::init();
        if !bv.is_select0_enabled() || !bv.is_select1_enabled() {
            return qc::TestResult::discard();
        }
        
        let mut zeros: Vec<usize> = Vec::new();
        let mut ones: Vec<usize> = Vec::new();
        
        for i in 0..bv.size_ {
            if bv.rank0(i) != zeros.len() {
                return qc::TestResult::failed();
            }
            if bv.rank1(i) != ones.len() {
                return qc::TestResult::failed();
            }
            if bv.at(i) {
                ones.push(i);
            } else {
                zeros.push(i)
            };
        }
        for (sel_idx, &val) in zeros.iter().enumerate() {
            if bv.select0(sel_idx) != val {
                return qc::TestResult::failed();
            }
        }
        for (sel_idx, &val) in ones.iter().enumerate() {
            if bv.select1(sel_idx) != val {
                return qc::TestResult::failed();
            }
        }
        if bv.num_0s() != zeros.len() {
            return qc::TestResult::failed();
        }
        if bv.num_1s() != ones.len() {
            return qc::TestResult::failed();
        }
        qc::TestResult::passed()

        // Add this back in when we can read/write
/*
        std::stringstream stream;
        {
          marisa::grimoire::Writer writer;
          writer.open(stream);
          bv.write(writer);
        }

        bv.clear();

        assert!(bv.size() == 0);
        assert!(bv.empty());
        assert!(bv.total_size() == 0);
        assert!(bv.io_size() == sizeof(marisa::UInt64) * 5);
        
        {
          marisa::grimoire::Reader reader;
          reader.open(stream);
          bv.read(reader);
        }

        assert!(bv.size() == bits.size());

        num_zeros = 0, num_ones = 0;
        for (std::size_t i = 0; i < bits.size(); ++i) {
          assert!(bv[i] == bits[i]);
          assert!(bv.rank0(i) == num_zeros);
          assert!(bv.rank1(i) == num_ones);
          ++(bv[i] ? num_ones : num_zeros);
        }
        for (std::size_t i = 0; i < zeros.size(); ++i) {
          assert!(bv.select0(i) == zeros[i]);
        }
        for (std::size_t i = 0; i < ones.size(); ++i) {
          assert!(bv.select1(i) == ones[i]);
        }
        assert!(bv.num_0s() == num_zeros);
        assert!(bv.num_1s() == num_ones);
*/
    }

    #[test]
    fn test_bit_vector() {
        let _ = env_logger::init();
        qc::quickcheck(test_bit_vector_prop as fn(BitVec) -> qc::TestResult);
    }
}

