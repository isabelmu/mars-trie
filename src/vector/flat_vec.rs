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

use std;
use base::WORD_SIZE;
use super::util::vec_resize;

#[derive(Clone, Debug)]
pub struct FlatVec {
    units_: Vec<usize>,
    value_size_: usize,
    mask_: u32,
    len_: usize,
}

impl FlatVec {
    pub fn new() -> FlatVec {
        FlatVec { units_: Vec::new(), value_size_: 0, mask_: 0, len_: 0, }
    }

    // I imagine there's a better way to do this? I guess implementing
    // FromIterator would be a start

    pub fn from_values<'a, T, Q>(x: T) -> FlatVec
      where T: Clone + IntoIterator<Item=&'a u32, IntoIter=Q>,
            Q: ExactSizeIterator<Item=&'a u32> {
        let mut fv = FlatVec::new();
        fv.build(x);
        fv
    }

    pub fn build<'a, T, Q>(&mut self, values: T)
      where T: Clone + IntoIterator<Item=&'a u32, IntoIter=Q>,
            Q: ExactSizeIterator<Item=&'a u32> {
        self.clear();

        let values_len = values.clone().into_iter().len();

        let mut max_value = *values.clone().into_iter().max().unwrap_or(&0);

        let mut value_size: usize = 0;
        while max_value != 0 {
            value_size += 1;
            max_value >>= 1;
        }

        let mut num_units: usize = if values_len == 0 { 0 }
                                                 else { 64 / WORD_SIZE };
        if value_size != 0 {
            num_units = ((value_size as usize * values_len + WORD_SIZE - 1)
                         / WORD_SIZE) as usize;
            num_units += num_units % (64 / WORD_SIZE);
        }

        vec_resize(&mut self.units_, num_units);
        if num_units > 0 {
            *self.units_.last_mut().unwrap() = 0;
        }

        self.value_size_ = value_size;
        if value_size != 0 {
            self.mask_ = std::u32::MAX.wrapping_shr(32 - value_size as u32);
        }
        self.len_ = values.clone().into_iter().len();

        for (idx, &val) in values.clone().into_iter().enumerate() {
            self.set(idx, val);
        }
    }

/*
  void map(Mapper &mapper) {
    FlatVec temp;
    temp.map_(mapper);
    swap(temp);
  }
  void read(Reader &reader) {
    FlatVec temp;
    temp.read_(reader);
    swap(temp);
  }
  void write(Writer &writer) const {
    write_(writer);
  }
*/

    fn at(&self, i: usize) -> u32 {
        assert!(i < self.len(), "MARISA_BOUND_ERROR");
        let pos = i * self.value_size();
        let unit_id = pos / WORD_SIZE;
        let unit_offset = pos % WORD_SIZE;
        if unit_offset + self.value_size() <= WORD_SIZE {
            self.units_[unit_id].wrapping_shr(unit_offset as u32) as u32
            & self.mask_
        } else {
            (self.units_[unit_id].wrapping_shr(unit_offset as u32) as u32)
            | (self.units_[unit_id + 1]
               .wrapping_shl((WORD_SIZE - unit_offset) as u32)
               as u32)
            & self.mask_
        }
    }
    pub fn value_size(&self) -> usize {
        self.value_size_
    }
    fn mask(&self) -> u32 {
        self.mask_
    }
    pub fn is_empty(&self) -> bool {
        self.units_.len() == 0
    }
    pub fn len(&self) -> usize {
        self.len_
    }
    fn storage_len(&self) -> usize {
        self.units_.len()
    }

    // FIXME: use From/Into
    fn to_vec(&self) -> Vec<u32> {
        let mut vals = Vec::new();
        for i in 0..self.len() { vals.push(self.at(i)); }
        vals
    }

/*
    fn total_size() -> usize {
      return units_.total_size();
    }
*/
/*
    fn io_size() -> usize {
      units_.io_size() + (sizeof(u32) * 2) + sizeof(u64)
    }
*/

    fn clear(&mut self) {
        // FIXME: Should keep allocation around instead, no?
        *self = FlatVec::new();
    }


/*
  void map_(Mapper &mapper) {
    units_.map(mapper);
    {
      u32 temp_value_size;
      mapper.map(&temp_value_size);
      MARISA_THROW_IF(temp_value_size > 32, MARISA_FORMAT_ERROR);
      value_size_ = temp_value_size;
    }
    {
      u32 temp_mask;
      mapper.map(&temp_mask);
      mask_ = temp_mask;
    }
    {
      u64 temp_size;
      mapper.map(&temp_size);
      MARISA_THROW_IF(temp_size > MARISA_SIZE_MAX, MARISA_SIZE_ERROR);
      len_ = (usize)temp_size;
    }
  }

  void read_(Reader &reader) {
    units_.read(reader);
    {
      u32 temp_value_size;
      reader.read(&temp_value_size);
      MARISA_THROW_IF(temp_value_size > 32, MARISA_FORMAT_ERROR);
      value_size_ = temp_value_size;
    }
    {
      u32 temp_mask;
      reader.read(&temp_mask);
      mask_ = temp_mask;
    }
    {
      u64 temp_size;
      reader.read(&temp_size);
      MARISA_THROW_IF(temp_size > MARISA_SIZE_MAX, MARISA_SIZE_ERROR);
      len_ = (usize)temp_size;
    }
  }

  void write_(Writer &writer) const {
    units_.write(writer);
    writer.write((u32)value_size_);
    writer.write((u32)mask_);
    writer.write((u64)len_);
  }
*/

    fn set(&mut self, i: usize, value: u32) {
        assert!(i < self.len(), "MARISA_BOUND_ERROR");
        assert!(value <= self.mask_, "MARISA_RANGE_ERROR");

        let pos = i * self.value_size();
        let unit_id = pos / WORD_SIZE;
        let unit_offset = pos % WORD_SIZE;

        self.units_[unit_id] &=
            !(self.mask_ as usize).wrapping_shl(unit_offset as u32);

        self.units_[unit_id] |=
            ((value & self.mask_) as usize).wrapping_shl(unit_offset as u32);

        if unit_offset + self.value_size() > WORD_SIZE {
            self.units_[unit_id + 1] &=
                !((self.mask_ as usize)
                  .wrapping_shr((WORD_SIZE - unit_offset) as u32));
            self.units_[unit_id + 1] |=
                ((value & self.mask_) as usize)
                .wrapping_shr((WORD_SIZE - unit_offset) as u32);
        }
    }
}

#[cfg(test)]
mod test {
    use std;
//    use base::WORD_SIZE;
    use super::FlatVec;
    use quickcheck as qc;
    use env_logger;

    impl qc::Arbitrary for FlatVec {
        fn arbitrary<G: qc::Gen>(g: &mut G) -> FlatVec {
            let values: Vec<u32> = qc::Arbitrary::arbitrary(g);
            let mut v = FlatVec::new();
            v.build(&values);
            v
        }
        fn shrink(&self) -> Box<Iterator<Item=Self>> {
            if self.len() > 1 {
                let mut v = Vec::new();
                let vals = self.to_vec();
                let split = self.len() / 2;
                v.push(FlatVec::from_values(vals.iter().take(split)));
                v.push(FlatVec::from_values(vals.iter().skip(split)));
                Box::new(v.into_iter())
            } else {
                qc::empty_shrinker()
            }
        }
    }

    #[test]
    fn test_flat_vec_qc() {
        let _ = env_logger::init();
        fn prop(mut fv: FlatVec) -> bool {
            let v = fv.to_vec();
            for (idx, &val) in v.iter().enumerate() {
                if fv.at(idx) != val { return false; }
            }
            fv.build(&v);
            for (idx, &val) in v.iter().enumerate() {
                if fv.at(idx) != val { return false; }
            }
            true
        }
        qc::quickcheck(prop as fn(FlatVec) -> bool);
    }

    // From marisa-trie/tests/vector-test.cc
    #[test]
    fn test_flat_vec_manual() {
        let _ = env_logger::init();

        let mut vec = FlatVec::new();

        assert!(vec.value_size() == 0);
        assert!(vec.mask() == 0);
        assert!(vec.len() == 0);
        assert!(vec.is_empty());
        //assert!(vec.total_size() == 0);
        //assert!(vec.io_size() == (sizeof(u64) * 3));

        let mut values = Vec::<u32>::new();

        vec.build(&values);

        assert!(vec.value_size() == 0);
        assert!(vec.mask() == 0);
        assert!(vec.len() == 0);
        assert!(vec.is_empty());
        //assert!(vec.total_size() == 0);
        //assert!(vec.io_size() == (sizeof(u64) * 3));

        values.push(0);
        vec.build(&values);

        assert!(vec.value_size() == 0);
        assert!(vec.mask() == 0);
        assert!(vec.len() == 1);
        assert!(!vec.is_empty());
        //assert!(vec.total_size() == 8);
        //assert!(vec.io_size() == (sizeof(u64) * 4));
        assert!(vec.at(0) == 0);

        values.push(255);
        vec.build(&values);

        assert!(vec.value_size() == 8);
        assert!(vec.mask() == 0xFF);
        assert!(vec.len() == 2);
        assert!(vec.at(0) == 0);
        assert!(vec.at(1) == 255);

        values.push(65536);
        vec.build(&values);

        assert!(vec.value_size() == 17);
        assert!(vec.mask() == 0x1FFFF);
        assert!(vec.len() == 3);
        assert!(vec.at(0) == 0);
        assert!(vec.at(1) == 255);
        assert!(vec.at(2) == 65536);

/*
        {
          Writer writer;
          writer.open("vector-test.dat");
          vec.write(writer);
        }
    
        vec.clear();
    
        assert!(vec.value_size() == 0);
        assert!(vec.mask() == 0);
        assert!(vec.size() == 0);
    
        {
          Mapper mapper;
          mapper.open("vector-test.dat");
          vec.map(mapper);
    
          assert!(vec.value_size() == 17);
          assert!(vec.mask() == 0x1FFFF);
          assert!(vec.size() == 3);
          assert!(vec[0] == 0);
          assert!(vec[1] == 255);
          assert!(vec[2] == 65536);
    
          vec.clear();
        }
    
        {
          Reader reader;
          reader.open("vector-test.dat");
          vec.read(reader);
        }
    
        assert!(vec.value_size() == 17);
        assert!(vec.mask() == 0x1FFFF);
        assert!(vec.size() == 3);
        assert!(vec[0] == 0);
        assert!(vec[1] == 255);
        assert!(vec[2] == 65536);
    
        values.clear();
        for (usize i = 0; i < 10000; ++i) {
          values.push(std::rand());
        }
        vec.build(values);
    
        assert!(vec.size() == values.size());
        for (usize i = 0; i < vec.size(); ++i) {
          assert!(vec[i] == values[i]);
        }
*/
    }
}

