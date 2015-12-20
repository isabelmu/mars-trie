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
    // union {
    //   f32 weight;
    //   u32 terminal;
    // }
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
struct Key<'a> {
    slice_: Option<&'a [u8]>,
    union_: Union,
    id_: u32,
}

#[derive(Copy, Clone)]
struct ReverseKey<'a> {
    slice_: Option<&'a [u8]>,
    union_: Union,
    id_: u32,
}

impl<'a> Key<'a> {
    fn new() -> Key<'a> {
        Key { slice_: None, union_: Union::new(), id_: 0 }
    }

    fn at(&self, i: usize) -> u8 {
        self.slice_.unwrap()[i]
    }

    fn substr(&mut self, pos: usize, length: usize) {
        if let Some(x) = self.slice_ {
            assert!(length <= x.len(), "MARISA_BOUND_ERROR");
            assert!(pos <= x.len() - length, "MARISA_BOUND_ERROR");
            self.slice_ = Some(&x[pos..pos+length]);
        } else {
            panic!();
        }
    }

    fn set_str(&mut self, slice: &'a[u8]) {
        assert!(slice.len() <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.slice_ = Some(slice);
    }
    fn set_weight(&mut self, weight: f32) {
        self.union_.set_weight(weight);
    }
    fn set_terminal(&mut self, terminal: usize) {
        self.union_.set_terminal(terminal);
    }
    fn set_id(&mut self, id: usize) {
        assert!(id <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.id_ = id as u32;
    }

    fn get_weight(&self) -> f32 {
        self.union_.get_weight()
    }
    fn get_terminal(&self) -> usize {
        self.union_.get_terminal()
    }
    fn get_id(&self) -> usize {
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

/*
class ReverseKey {
 public:
  ReverseKey()
      : ptr_(static_cast<const u8 *>(NULL) - 1),
        length_(0), union_(), id_(0) {
    union_.terminal = 0;
  }
  ReverseKey(const ReverseKey &entry)
      : ptr_(entry.ptr_), length_(entry.length_),
        union_(entry.union_), id_(entry.id_) {}

  ReverseKey &operator=(const ReverseKey &entry) {
    ptr_ = entry.ptr_;
    length_ = entry.length_;
    union_ = entry.union_;
    id_ = entry.id_;
    return *this;
  }

  u8 operator[](usize i) const {
    MARISA_DEBUG_IF(i >= length_, MARISA_BOUND_ERROR);
    return *(ptr_ - i);
  }

  void substr(usize pos, usize length) {
    MARISA_DEBUG_IF(pos > length_, MARISA_BOUND_ERROR);
    MARISA_DEBUG_IF(length > length_, MARISA_BOUND_ERROR);
    MARISA_DEBUG_IF(pos > (length_ - length), MARISA_BOUND_ERROR);
    ptr_ -= pos;
    length_ = (u32)length;
  }

  void set_str(const u8 *ptr, usize length) {
    MARISA_DEBUG_IF((ptr == NULL) && (length != 0), MARISA_NULL_ERROR);
    MARISA_DEBUG_IF(length > MARISA_UINT32_MAX, MARISA_SIZE_ERROR);
    ptr_ = ptr + length - 1;
    length_ = (u32)length;
  }
  void set_weight(f32 weight) {
    union_.weight = weight;
  }
  void set_terminal(usize terminal) {
    MARISA_DEBUG_IF(terminal > MARISA_UINT32_MAX, MARISA_SIZE_ERROR);
    union_.terminal = (u32)terminal;
  }
  void set_id(usize id) {
    MARISA_DEBUG_IF(id > MARISA_UINT32_MAX, MARISA_SIZE_ERROR);
    id_ = (u32)id;
  }

  const u8 *ptr() const {
    return ptr_ - length_ + 1;
  }
  usize length() const {
    return length_;
  }
  f32 weight() const {
    return union_.weight;
  }
  usize terminal() const {
    return union_.terminal;
  }
  usize id() const {
    return id_;
  }

 private:
}

bool operator==(const ReverseKey &lhs, const ReverseKey &rhs) {
  if (lhs.length() != rhs.length()) {
    return false;
  }
  for (usize i = 0; i < lhs.length(); ++i) {
    if (lhs[i] != rhs[i]) {
      return false;
    }
  }
  return true;
}

bool operator!=(const ReverseKey &lhs, const ReverseKey &rhs) {
  return !(lhs == rhs);
}

bool operator<(const ReverseKey &lhs, const ReverseKey &rhs) {
  for (usize i = 0; i < lhs.length(); ++i) {
    if (i == rhs.length()) {
      return false;
    }
    if (lhs[i] != rhs[i]) {
      return (UInt8)lhs[i] < (UInt8)rhs[i];
    }
  }
  return lhs.length() < rhs.length();
}

bool operator>(const ReverseKey &lhs, const ReverseKey &rhs) {
  return rhs < lhs;
}

*/
