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

#[derive(Copy, Clone)]
pub struct Range {
    begin_: u32,
    end_: u32,
    key_pos_: u32,
}

impl Range {
    fn new(begin: usize, end: usize, key_pos: usize) -> Range {
        Range {
            begin_: begin as u32,
            end_: end as u32,
            key_pos_: key_pos as u32
        }
    }

    pub fn set_begin(&mut self, begin: usize) {
        assert!(begin <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.begin_ = begin as u32;
    }
    pub fn set_end(&mut self, end: usize) {
        assert!(end <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.end_ = end as u32;
    }
    pub fn set_key_pos(&mut self, key_pos: usize) {
        assert!(key_pos <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.key_pos_ = key_pos as u32;
    }
  
    pub fn begin(&self) -> usize {
        self.begin_ as usize
    }
    pub fn end(&self) -> usize {
        self.end_ as usize
    }
    pub fn key_pos(&self) -> usize {
        self.key_pos_ as usize
    }
}

#[derive(Copy, Clone)]
pub struct WeightedRange {
    range_: Range,
    weight_: f32,
}

impl WeightedRange {
    pub fn new(begin: usize, end: usize, key_pos: usize, weight: f32)
      -> WeightedRange {
        WeightedRange {
            range_: Range::new(begin, end, key_pos),
            weight_ : weight,
        }
    }
  
    pub fn set_range(&mut self, range: &Range) {
        self.range_ = *range
    }
    pub fn set_begin(&mut self, begin: usize) {
        self.range_.set_begin(begin);
    }
    pub fn set_end(&mut self, end: usize) {
        self.range_.set_end(end);
    }
    pub fn set_key_pos(&mut self, key_pos: usize) {
        self.range_.set_key_pos(key_pos);
    }
    pub fn set_weight(&mut self, weight: f32) {
        self.weight_ = weight
    }
  
    pub fn range(&self) -> &Range {
        &self.range_
    }
    pub fn begin(&self) -> usize {
        self.range_.begin()
    }
    pub fn end(&self) -> usize {
        self.range_.end()
    }
    pub fn key_pos(&self) -> usize {
        self.range_.key_pos()
    }
    pub fn weight(&self) -> f32 {
        self.weight_
    }
}

impl PartialEq for WeightedRange { 
    fn eq(&self, rhs: &WeightedRange) -> bool {
        self.weight() == rhs.weight()
    }
}

impl Eq for WeightedRange {}

impl PartialOrd for WeightedRange { 
    fn partial_cmp(&self, rhs: &WeightedRange) -> Option<std::cmp::Ordering> {
        self.weight().partial_cmp(&rhs.weight())
    }
}

