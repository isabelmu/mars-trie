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

use trie::key::Key;
use trie::state::State;

/*
fn unwrap_ref<T>(opt: &Option<T>) -> &T {
    opt.as_ref().unwrap()
}

fn unwrap_mut<T>(opt: &mut Option<T>) -> &mut T {
    opt.as_mut().unwrap()
}
*/

pub struct Query<'a> {
    opt_slice_: &'a [u8],
    opt_state_: State,
}

impl<'a> Query<'a> {
    pub fn new(slice: &'a [u8]) -> Query<'a> {
        Query { opt_slice_: slice, opt_state_: None }
    }

    pub fn get_slice(&self) -> &'a [u8] {
        &self.opt_slice_
    }

    pub fn get_state(&self) -> &State {
        unwrap_ref(&self.opt_state_)
    }

    pub fn get_state_mut(&mut self) -> &mut State {
        unwrap_mut(&mut self.opt_state_)
    }

    pub fn set_slice(&mut self, slice: &'a [u8]) {
        self.opt_query_ = Some(slice);
        self.reset_state();
    }

    pub fn reset_state(&mut self) {
        self.opt_state_ = Some(State::new());
    }

    pub fn has_state(&self) -> bool {
        self.opt_state_.is_some()
    }
}

