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

use trie::state::State;

enum Query {
    StrQ(&str),
    IdQ(usize),
}
 
struct Agent {
    key_: Key,
    query_: Option<Query>,
    state_: Option<State>,
}

impl Agent {
    fn new() -> Agent {
        Agent { key_(), query_: None, state_: None }
    }

    const Query &query() const {
      return query_;
    }
    const Key &key() const {
      return key_;
    }

    const grimoire::trie::State &state() const {
      return *state_;
    }
    grimoire::trie::State &state() {
      return *state_;
    }
 
    void Agent::set_query(const char *str) {
      MARISA_THROW_IF(str == NULL, MARISA_NULL_ERROR);
      if (state_.get() != NULL) {
        state_->reset();
      }
      query_.set_str(str);
    }

    void Agent::set_query(const char *ptr, std::size_t length) {
      MARISA_THROW_IF((ptr == NULL) && (length != 0), MARISA_NULL_ERROR);
      if (state_.get() != NULL) {
        state_->reset();
      }
      query_.set_str(ptr, length);
    }

    void Agent::set_query(std::size_t key_id) {
      if (state_.get() != NULL) {
        state_->reset();
      }
      query_.set_id(key_id);
    }

    void set_key(const char *str) {
      MARISA_DEBUG_IF(str == NULL, MARISA_NULL_ERROR);
      key_.set_str(str);
    }
    void set_key(const char *ptr, std::size_t length) {
      MARISA_DEBUG_IF((ptr == NULL) && (length != 0), MARISA_NULL_ERROR);
      MARISA_DEBUG_IF(length > MARISA_UINT32_MAX, MARISA_SIZE_ERROR);
      key_.set_str(ptr, length);
    }
    void set_key(std::size_t id) {
      MARISA_DEBUG_IF(id > MARISA_UINT32_MAX, MARISA_SIZE_ERROR);
      key_.set_id(id);
    }
  
    fn init_state(&mut self) {
        if self.has_state() panic!("MARISA_STATE_ERROR");
        self.state_ = State::new();
    }

    fn has_state(&self) -> bool {
        self.state_.is_some()
    }
 
    fn Agent::clear(&mut self) {
        *self = Agent::new();
    }
}

