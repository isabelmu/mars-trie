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

