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

#[derive(Copy, Clone, Debug)]
struct Union {
    /// link: u32 or weight: f32
    bits_: u32,
}

impl Union {
    fn new() -> Union {
        Union { bits_: 0 }
    }
    fn get_weight(&self) -> f32 {
        unsafe { std::mem::transmute(self.bits_) }
    }
    fn get_link(&self) -> u32 {
        self.bits_
    }
    fn set_weight(&mut self, weight: f32) {
        self.bits_ = unsafe { std::mem::transmute(weight) };
    }
    fn set_link(&mut self, link: u32) {
        self.bits_ = link
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Cache {
    parent_: u32,
    child_: u32,
    union_: Union,
}

impl Cache {
    pub fn new() -> Cache {
        let mut out = Cache { parent_: 0, child_: 0, union_: Union::new() };
        out.set_weight(std::f32::MIN);
        out
    }

    pub fn set_parent(&mut self, parent: u32) {
        self.parent_ = parent;
    }
    pub fn set_child(&mut self, child: u32) {
        self.child_ = child;
    }
    pub fn set_base(&mut self, base: u8) {
        let new_link = (self.union_.get_link() & !0xFFu32) | (base as u32);
        self.union_.set_link(new_link);
    }
    pub fn set_extra(&mut self, extra: u32) {
        assert!(extra <= 0x00FFFFFF, "MARISA_SIZE_ERROR");
        let new_link = (self.union_.get_link() & 0xFFu32) | (extra << 8);
        self.union_.set_link(new_link);
    }
    pub fn set_weight(&mut self, weight: f32) {
        self.union_.set_weight(weight);
    }

    pub fn parent(&self) -> u32 {
        self.parent_
    }
    pub fn child(&self) -> u32 {
        self.child_
    }
    pub fn base(&self) -> u8 {
        (self.union_.get_link() & 0xFFu32) as u8
    }
    pub fn extra(&self) -> u32 {
        (self.union_.get_link() & 0x00FFFFFFu32) >> 8
    }
    pub fn label(&self) -> u8 {
        self.base()
    }
    pub fn link(&self) -> u32 {
        self.union_.get_link()
    }
    pub fn weight(&self) -> f32 {
        self.union_.get_weight()
    }
}

