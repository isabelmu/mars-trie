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

use base::WORD_SIZE;
use std;
use std::default::Default;
use super::util::vec_resize;

/// Static packed vector of u32 values. Bit size of each element is determined
/// by the size of the largest element.
struct PackedVec {
    units_: Vec<usize>,
    value_size_: usize,
    mask_: u32,
    size_: usize,
}

impl PackedVec {
    fn new() -> PackedVec {
        PackedVec {
            units_: Default::default(),
            value_size_: 0,
            mask_: 0,
            size_: 0,
        }
    }
 
    fn build(values: &Vec<u32>) -> PackedVec {
        let mut out = PackedVec::new();

        let mut max_value = match values.iter().max() {
            Some(x) => *x,
            None => { return out; }
        };

        let mut value_size: usize = 0;
        while max_value != 0 {
            value_size += 1;
            max_value >>= 1;
        }

        let num_units = match value_size {
            0 => if values.is_empty() { 0 } else { 64 / WORD_SIZE },
            vs => {
                let tmp = value_size as u64 * values.len() as u64;
                // # of words, rounded up
                let tmp = ((tmp + (WORD_SIZE - 1) as u64)
                           / WORD_SIZE as u64) as usize;
                tmp + tmp % (64 / WORD_SIZE)
            }
        };

        vec_resize(&mut out.units_, num_units);
        if num_units > 0 {
            *out.units_.last_mut().unwrap() = 0;
        }
  
        out.value_size_ = value_size;
        if value_size != 0 {
            out.mask_ = std::u32::MAX >> (32 - value_size);
        }
        out.size_ = values.len();
  
        for i in 0..values.len() {
            out.set(i, values[i]);
        }
        out
    }

// Need to implement Mapper, Reader, Writer first. I believe we can use
// existing standard traits for Reader and Writer at least.
/*    void map(Mapper &mapper) {
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
            size_ = (usize)temp_size;
        }
    }

    void read(Reader &reader) {
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
            size_ = (usize)temp_size;
        }
    }
 
    void write(Writer &writer) const {
        units_.write(writer);
        writer.write((u32)value_size_);
        writer.write((u32)mask_);
        writer.write((u64)size_);
    }
*/

    // I'm not sure whether it's possible to create a nice iterator over a class
    // like this (that would require a proxy object) just yet. Should try it
    // though.

    fn at(&self, i: usize) -> u32 {
      assert!(i < self.size_, "MARISA_BOUND_ERROR");
  
      let pos: usize = i * self.value_size_;
      let unit_id: usize = pos / WORD_SIZE;
      let unit_offset: usize = pos % WORD_SIZE;
  
      (if unit_offset + self.value_size_ <= WORD_SIZE {
          self.units_[unit_id] >> unit_offset
      } else {
          (self.units_[unit_id] >> unit_offset)
          | (self.units_[unit_id + 1] << (WORD_SIZE - unit_offset))
      })
      as u32 & self.mask_
    }
  
    fn value_size(&self) -> usize {
        self.value_size_
    }
    fn mask(&self) -> u32 {
        self.mask_
    }
  
    fn is_empty(&self) -> bool {
        self.size_ == 0
    }
    fn size(&self) -> usize {
        self.size_
    }
    fn total_size(&self) -> usize {
        self.units_.len() * std::mem::size_of::<usize>()
    }
//    fn io_size(&self) -> usize {
//        units_.io_size()
//        + (std::mem::size_of::<u32>() * 2)
//        + std::mem::size_of::<u64>()
//    }

    fn clear(&mut self) {
        *self = PackedVec::new();
    }
 
//   private:
    fn set(&mut self, i: usize, value: u32) {
 
        // FIXME: is this assertion correct...?
        assert!(i < self.size_, "MARISA_BOUND_ERROR");
        assert!(value <= self.mask_, "MARISA_RANGE_ERROR");
  
        let pos: usize = i * self.value_size_;
        let unit_id: usize = pos / WORD_SIZE;
        let unit_offset: usize = pos % WORD_SIZE;
  
        self.units_[unit_id] &= !((self.mask_ as usize) << unit_offset);
        self.units_[unit_id] |= ((value & self.mask_) as usize) << unit_offset;
        if (unit_offset + self.value_size_) > WORD_SIZE {
            self.units_[unit_id + 1] &=
                !((self.mask_ as usize) >> (WORD_SIZE - unit_offset));
            self.units_[unit_id + 1] |=
                ((value & self.mask_) as usize)
                >> (WORD_SIZE - unit_offset);
        }
    }
}

