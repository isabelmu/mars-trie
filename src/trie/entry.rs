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

use std::cmp::Ordering;
use iter_util::common_count_eq;

pub struct Entry<'a> {
    slice_: &'a [u8],
    id_: u32,
}

impl<'a> Entry<'a> {
    pub fn new(slice: &'a [u8], id: u32) -> Entry<'a> {
        Entry { slice_: slice, id_: id }
    }
    pub fn common_count<'b>(&'a self, rhs: &Entry<'b>) -> usize {
        common_count_eq(self.slice_.iter(), rhs.slice_.iter())
    }
    pub fn len(&self) -> usize {
        self.slice_.len()
    }
    pub fn get_slice(&self) -> &'a [u8] {
        self.slice_
    }
    pub fn set_slice(&mut self, slice: &'a [u8]) {
        self.slice_ = slice;
    }
    pub fn get_id(&self) -> u32 {
        self.id_
    }
    pub fn set_id(&mut self, id: u32) {
        self.id_ = id;
    }
}

pub fn cmp_slice(l: &Entry, r: &Entry) -> Ordering {
    l.slice_.cmp(&r.slice_)
}

pub fn cmp_id(l: &Entry, r: &Entry) -> Ordering {
    l.id_.cmp(&r.id_)
}

