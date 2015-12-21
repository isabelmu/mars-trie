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
use trie::entry::Entry;

#[derive(Copy, Clone)]
struct Union {
    // weight or terminal
    bits_: u32,
}

impl Union {
    fn new() -> Union {
        Union { bits_: 0 }
    }

    fn get_weight(&self) -> f32 {
        unsafe { std::mem::transmute(self.bits_) }
    }
    fn get_terminal(&self) -> usize {
        self.bits_ as usize
    }

    fn set_weight(&mut self, weight: f32) {
        self.bits_ = unsafe { std::mem::transmute(weight) };
    }
    fn set_terminal(&mut self, terminal: usize) {
        assert!(terminal <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.bits_ = terminal as u32;
    }
}

#[derive(Copy, Clone)]
pub struct Key<'a> {
    slice_: &'a[u8],
    union_: Union,
    id_: u32,
}

impl<'a> Key<'a> {
    pub fn new(slice: &'a[u8]) -> Key<'a> {
        Key { slice_: slice, union_: Union::new(), id_: 0 }
    }

    pub fn at(&self, i: usize) -> u8 {
        self.slice_[i]
    }

    pub fn subslice(&mut self, pos: usize, length: usize) {
        assert!(length <= self.slice_.len(), "MARISA_BOUND_ERROR");
        assert!(pos <= self.slice_.len() - length, "MARISA_BOUND_ERROR");
        self.slice_ = &self.slice_[pos..pos+length];
    }

    pub fn set_slice(&mut self, slice: &'a[u8]) {
        assert!(slice.len() <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.slice_ = slice;
    }
    pub fn set_weight(&mut self, weight: f32) {
        self.union_.set_weight(weight);
    }
    pub fn set_terminal(&mut self, terminal: usize) {
        self.union_.set_terminal(terminal);
    }
    pub fn set_id(&mut self, id: usize) {
        assert!(id <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.id_ = id as u32;
    }

    pub fn get_slice(&self) -> &'a[u8] {
        self.slice_
    }
    pub fn get_weight(&self) -> f32 {
        self.union_.get_weight()
    }
    pub fn get_terminal(&self) -> usize {
        self.union_.get_terminal()
    }
    pub fn get_id(&self) -> usize {
        self.id_ as usize
    }
}

impl<'a> PartialEq for Key<'a> {
    fn eq(&self, rhs: &Key<'a>) -> bool {
        self.slice_ == rhs.slice_
    }
}

impl<'a> Eq for Key<'a> {}

impl<'a> PartialOrd for Key<'a> {
    fn partial_cmp(&self, rhs: &Key) -> Option<std::cmp::Ordering> {
        self.slice_.partial_cmp(&rhs.slice_)
    }
}

impl<'a> Ord for Key<'a> {
    fn cmp(&self, rhs: &Key) -> std::cmp::Ordering {
        self.slice_.cmp(&rhs.slice_)
    }
}

/// Just like Key, except we index and subslice from the end of the slice
#[derive(Copy, Clone)]
pub struct ReverseKey<'a> {
    slice_: &'a[u8],
    union_: Union,
    id_: u32,
}

// FIXME: Reduce amount of identical code between Key and ReverseKey. Only
//        at() and subslice() are different at all!
impl<'a> ReverseKey<'a> {
    pub fn new(slice: &'a[u8]) -> ReverseKey<'a> {
        ReverseKey { slice_: slice, union_: Union::new(), id_: 0 }
    }

    pub fn at(&self, i: usize) -> u8 {
        self.slice_[self.slice_.len() - i - 1]
    }

    pub fn subslice(&mut self, pos: usize, length: usize) {
        assert!(length <= self.slice_.len(), "MARISA_BOUND_ERROR");
        assert!(pos <= self.slice_.len() - length, "MARISA_BOUND_ERROR");
        let new_end = self.slice_.len() - pos;
        let new_begin = new_end - length;
        self.slice_ = &self.slice_[new_begin..new_end];
    }

    pub fn set_slice(&mut self, slice: &'a[u8]) {
        assert!(slice.len() <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.slice_ = slice;
    }
    pub fn set_weight(&mut self, weight: f32) {
        self.union_.set_weight(weight);
    }
    pub fn set_terminal(&mut self, terminal: usize) {
        self.union_.set_terminal(terminal);
    }
    pub fn set_id(&mut self, id: usize) {
        assert!(id <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.id_ = id as u32;
    }

    pub fn get_slice(&self) -> &'a[u8] {
        self.slice_
    }
    pub fn get_weight(&self) -> f32 {
        self.union_.get_weight()
    }
    pub fn get_terminal(&self) -> usize {
        self.union_.get_terminal()
    }
    pub fn get_id(&self) -> usize {
        self.id_ as usize
    }
}

impl<'a> PartialEq for ReverseKey<'a> {
    fn eq(&self, rhs: &ReverseKey<'a>) -> bool {
        self.slice_ == rhs.slice_
    }
}

impl<'a> Eq for ReverseKey<'a> {}

impl<'a> PartialOrd for ReverseKey<'a> {
    fn partial_cmp(&self, rhs: &ReverseKey) -> Option<std::cmp::Ordering> {
        self.slice_.partial_cmp(&rhs.slice_)
    }
}

impl<'a> Ord for ReverseKey<'a> {
    fn cmp(&self, rhs: &ReverseKey) -> std::cmp::Ordering {
        self.slice_.cmp(&rhs.slice_)
    }
}

