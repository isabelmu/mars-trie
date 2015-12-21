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

fn unwrap_ref<T>(opt: &Option<T>) -> &T {
    opt.as_ref().unwrap()
}

fn unwrap_mut<T>(opt: &mut Option<T>) -> &mut T {
    opt.as_mut().unwrap()
}

enum Query<'a> {
    Slice(&'a [u8]),
    ID(usize),
}
 
pub struct Agent<'a> {
    key_: Key<'a>,
    opt_query_: Option<Query<'a> >,
    opt_state_: Option<State>,
}

impl<'a> Agent<'a> {
    pub fn new() -> Agent<'a> {
        Agent { key_: Key::new(), opt_query_: None, opt_state_: None }
    }

    pub fn get_query(&self) -> &Query {
        unwrap_ref(&self.opt_query_)
    }
    pub fn get_key(&self) -> &Key<'a> {
        &self.key_
    }

    pub fn get_state(&self) -> &State {
        unwrap_ref(&self.opt_state_)
    }
    pub fn get_state_mut(&mut self) -> &mut State {
        unwrap_mut(&mut self.opt_state_)
    }

    fn reset_state(&mut self) {
        match &mut self.opt_state_ {
            &mut Some(ref mut x) => x.reset(),
            &mut None => (),
        }
    }

    pub fn set_query_by_slice(&mut self, slice: &'a [u8]) {
        self.reset_state();
        self.opt_query_ = Some(Query::Slice(slice));
    }

    pub fn set_query_by_id(&mut self, key_id: usize) {
        self.reset_state();
        self.opt_query_ = Some(Query::ID(key_id));
    }

    pub fn set_key_by_slice(&mut self, slice: &'a [u8]) {
        self.key_.set_slice(slice);
    }
    pub fn set_key_by_id(&mut self, id: usize) {
        self.key_.set_id(id);
    }
  
    pub fn init_state(&mut self) {
        if self.has_state() { panic!("MARISA_STATE_ERROR"); }
        self.opt_state_ = Some(State::new());
    }

    pub fn has_state(&self) -> bool {
        self.opt_state_.is_some()
    }
 
    pub fn clear(&mut self) {
        *self = Agent::new();
    }
}

