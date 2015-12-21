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

use config::Config;
use config::CacheLevel;
use config::NodeOrder;
use config::TailMode;
use trie::cache::Cache;
use trie::tail::Tail;
use vector::bit_vec::BitVec;
use vector::flat_vec::FlatVec;

use base::INVALID_LINK_ID;

pub const INVALID_EXTRA: u32 = std::u32::MAX >> 8;

/// Recursive LOUDS trie
pub struct LoudsTrie {
    louds_: BitVec,
    terminal_flags_: BitVec,
    link_flags_: BitVec,
    bases_: Vec<u8>,
    extras_: FlatVec,
    tail_: Tail,
    next_trie_: Option<Box<LoudsTrie> >,
    cache_: Vec<Cache>,
    cache_mask_: usize,
    num_l1_nodes_: usize,
    config_: Config,
//    mapper_: Mapper,
}

impl LoudsTrie {
    // We shouldn't expose this. Clients can just use build, map, and read.
    fn new() -> LoudsTrie {
        LoudsTrie { 
            louds_: BitVec::new(),
            terminal_flags_: BitVec::new(),
            link_flags_: BitVec::new(),
            bases_: Vec::new(),
            extras_: FlatVec::new(),
            tail_: Tail::new(),
            next_trie_: None,
            cache_: Vec::new(),
            cache_mask_: 0,
            num_l1_nodes_: 0,
            config_: Config::new(),
            // mapper: Mapper::new(),
        }
    }

    pub fn build(keys: &Vec<(&[u8], f32)>, flags: u32) -> LoudsTrie {
        let mut config = Config::parse(flags);
        let mut out = LoudsTrie::new();

        let mut terminals: Vec<u32> = Vec::new();
        out.build_trie(keys, &mut terminals, &mut config, 1);

        let mut pairs: Vec<(u32, u32)> = Vec::new();
        pairs.resize(terminals.len(), (0, 0));
        for (i, pair) in (&mut pairs).iter_mut().enumerate() {
            pair.0 = terminals[i];
            pair.1 = i as u32;
        }
        terminals.clear();
        pairs.sort();

        // FIXME: Clean up this usize/u32 situation.

        let mut node_id: usize = 0;
        for pair in &pairs {
            while node_id < pair.0 as usize {
                out.terminal_flags_.push(false);
                node_id += 1;
            }
            if node_id == pair.0 as usize {
                out.terminal_flags_.push(true);
                node_id += 1;
            }
        }
        while node_id < out.bases_.len() {
            out.terminal_flags_.push(false);
            node_id += 1;
        }
        out.terminal_flags_.push(false);
        out.terminal_flags_.build(false, true);

        //for pair in &pairs {
        //    keyset[pair.1].set_id(terminal_flags_.rank1(pair.0));
        //}
        out
    }

    fn build_trie(&mut self, keys: &Vec<(&[u8], f32)>, terminals: &mut Vec<u32>,
                  config: &mut Config, trie_id: usize)
    {
        //build_current_trie(keys, terminals, config, trie_id);

        let next_terminals: Vec<u32> = Vec::new();
        if !keys.is_empty() {
            //build_next_trie(keys, &next_terminals, config, trie_id);
        }

        match &self.next_trie_ {
            &Some(ref x) => {
                let new_cfg =
                    (x.num_tries() + 1) as usize
                    | x.tail_mode() as usize
                    | x.node_order() as usize;
                assert!(new_cfg <= std::u32::MAX as usize);
                *config = Config::parse(new_cfg as u32);
            },
            &None => {
                let new_cfg =
                    1
                    | self.tail_.mode() as usize // FIXME: tail_.mode??
                    | config.node_order() as usize
                    | config.cache_level() as usize;
                *config = Config::parse(new_cfg as u32);
            }
        }
/*
        link_flags_.build(false, false);
        usize node_id = 0;
        for (usize i = 0; i < next_terminals.size(); ++i) {
            while !link_flags_[node_id] {
                ++node_id;
            }
            bases_[node_id] = (u8)(next_terminals[i] % 256);
            next_terminals[i] /= 256;
            ++node_id;
        }
        extras_.build(next_terminals);
        fill_cache();
*/
    }

/*
    fn build_current_trie<T>(keys: &mut Vec<T>,
                             terminals: *mut Vec<u32>,
                             config: &Config, trie_id: usize) {
        for (usize i = 0; i < keys.size(); ++i) {
          keys[i].set_id(i);
        }
        const usize num_keys = Algorithm().sort(keys.begin(), keys.end());
        reserve_cache(config, trie_id, num_keys);

        louds_.push(true);
        louds_.push(false);
        bases_.push('\0');
        link_flags_.push(false);

        Vec<T> next_keys;
        std::queue<Range> queue;
        Vec<WeightedRange> w_ranges;

        queue.push(make_range(0, keys.size(), 0));
        while (!queue.empty()) {
          const usize node_id = link_flags_.size() - queue.size();

          Range range = queue.front();
          queue.pop();

          while ((range.begin() < range.end()) &&
              (keys[range.begin()].length() == range.key_pos())) {
            keys[range.begin()].set_terminal(node_id);
            range.set_begin(range.begin() + 1);
          }

          if (range.begin() == range.end()) {
            louds_.push(false);
            continue;
          }

          w_ranges.clear();
          double weight = keys[range.begin()].weight();
          for (usize i = range.begin() + 1; i < range.end(); ++i) {
            if (keys[i - 1][range.key_pos()] != keys[i][range.key_pos()]) {
              w_ranges.push(make_weighted_range(
                  range.begin(), i, range.key_pos(), (float)weight));
              range.set_begin(i);
              weight = 0.0;
            }
            weight += keys[i].weight();
          }
          w_ranges.push(make_weighted_range(
              range.begin(), range.end(), range.key_pos(), (float)weight));
          if (config.node_order() == MARISA_WEIGHT_ORDER) {
            std::stable_sort(w_ranges.begin(), w_ranges.end(),
                std::greater<WeightedRange>());
          }

          if (node_id == 0) {
            num_l1_nodes_ = w_ranges.size();
          }

          for (usize i = 0; i < w_ranges.size(); ++i) {
            WeightedRange &w_range = w_ranges[i];
            usize key_pos = w_range.key_pos() + 1;
            while (key_pos < keys[w_range.begin()].length()) {
              usize j;
              for (j = w_range.begin() + 1; j < w_range.end(); ++j) {
                if (keys[j - 1][key_pos] != keys[j][key_pos]) {
                  break;
                }
              }
              if (j < w_range.end()) {
                break;
              }
              ++key_pos;
            }
            cache<T>(node_id, bases_.size(), w_range.weight(),
                keys[w_range.begin()][w_range.key_pos()]);

            if (key_pos == w_range.key_pos() + 1) {
              bases_.push(keys[w_range.begin()][w_range.key_pos()]);
              link_flags_.push(false);
            } else {
              bases_.push('\0');
              link_flags_.push(true);
              T next_key;
              next_key.set_str(keys[w_range.begin()].ptr(),
                  keys[w_range.begin()].length());
              next_key.substr(w_range.key_pos(), key_pos - w_range.key_pos());
              next_key.set_weight(w_range.weight());
              next_keys.push(next_key);
            }
            w_range.set_key_pos(key_pos);
            queue.push(w_range.range());
            louds_.push(true);
          }
          louds_.push(false);
        }

        louds_.push(false);
        louds_.build(trie_id == 1, true);
        bases_.shrink();

        build_terminals(keys, terminals);
        keys.swap(next_keys);
    }

    fn build_next_trie(&mut self, keys: &mut Vec<Key>,
                       terminals: *mut Vec<u32>,
                       config: &Config, trie_id: usize) {
        if trie_id == config.num_tries() {
            Vec<Entry> entries;
            entries.resize(keys.size());
            for (usize i = 0; i < keys.size(); ++i) {
                entries[i].set_str(keys[i].ptr(), keys[i].length());
            }
            tail_.build(entries, terminals, config.tail_mode());
            return;
        }
        Vec<ReverseKey> reverse_keys;
        reverse_keys.resize(keys.size());
        for (usize i = 0; i < keys.size(); ++i) {
            reverse_keys[i].set_str(keys[i].ptr(), keys[i].length());
            reverse_keys[i].set_weight(keys[i].weight());
        }
        keys.clear();
        next_trie_.reset(new (std::nothrow) LoudsTrie);
        if next_trie_.get() == NULL {
            panic!();
        }
        next_trie_->build_trie(reverse_keys, terminals, config, trie_id + 1);
    }
    fn build_next_trie(&mut self, keys: &mut Vec<ReverseKey>,
                       terminals: *mut Vec<u32>,
                       config: &Config, trie_id: usize) {
        if trie_id == config.num_tries() {
            Vec<Entry> entries;
            entries.resize(keys.size());
            for (usize i = 0; i < keys.size(); ++i) {
                entries[i].set_str(keys[i].ptr(), keys[i].length());
            }
            tail_.build(entries, terminals, config.tail_mode());
            return;
        }
        next_trie_.reset(new (std::nothrow) LoudsTrie);
        if next_trie_.get() == NULL {
            panic!();
        }
        next_trie_->build_trie(keys, terminals, config, trie_id + 1);
    }

    fn build_terminals<T>(&self, keys: &Vec<T>,
                          terminals: *mut Vec<u32>) {
        Vec<u32> temp;
        temp.resize(keys.size());
        for (usize i = 0; i < keys.size(); ++i) {
            temp[keys[i].id()] = (u32)keys[i].terminal();
        }
        terminals->swap(temp);
    }
*/

    pub fn id_lookup(&self, id: usize) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        self.id_lookup_into_vec(id, &mut v);
        v
    }

    pub fn id_lookup_into_vec(&self, id: usize, key_out: &mut Vec<u8>) {
        assert!(id < self.size());
        key_out.clear();

        let mut node_id = self.terminal_flags_.select1(id);
        if node_id == 0 {
            return;
        }
        loop {
            if self.link_flags_.at(node_id) {
                let prev_key_pos = key_out.len();
                self.restore(self.get_link(node_id), key_out);
                key_out[prev_key_pos..].reverse();
            } else {
                key_out.push(self.bases_[node_id]);
            }
            if node_id <= self.num_l1_nodes_ {
                key_out.reverse();
                return;
            }
            node_id = self.louds_.select1(node_id) - node_id - 1;
        }
    }

    fn restore(&self, link: usize, key_out: &mut Vec<u8>) {
        match &self.next_trie_ {
            &Some(ref next) => {
                next.restore_(link, key_out);
            },
            &None => {
                self.tail_.restore(link, key_out);
            }
        }
    }

    fn restore_(&self, node_id: usize, key_out: &mut Vec<u8>) {
        assert!(node_id != 0, "MARISA_RANGE_ERROR");

        let mut node_id = node_id;
        loop {
            let cache_id = self.get_cache_id(node_id);
            if node_id == self.cache_[cache_id].child() as usize {
                if self.cache_[cache_id].extra() != INVALID_EXTRA {
                    self.restore(self.cache_[cache_id].link() as usize,
                                 key_out);
                } else {
                    key_out.push(self.cache_[cache_id].label());
                }
                node_id = self.cache_[cache_id].parent() as usize;
                if node_id == 0 {
                    return;
                }
            } else {
                if self.link_flags_.at(node_id) {
                    self.restore(self.get_link(node_id), key_out);
                } else {
                    key_out.push(self.bases_[node_id]);
                }
                if node_id <= self.num_l1_nodes_ {
                    return;
                }
                node_id = self.louds_.select1(node_id) - node_id - 1;
            }
        }
    }

    fn num_tries(&self) -> usize {
        self.config_.num_tries()
    }
    fn num_keys(&self) -> usize {
        self.size()
    }
    fn num_nodes(&self) -> usize {
        (self.louds_.size() / 2) - 1
    }
    fn cache_level(&self) -> CacheLevel {
        self.config_.cache_level()
    }
    fn tail_mode(&self) -> TailMode {
        self.config_.tail_mode()
    }
    fn node_order(&self) -> NodeOrder {
        self.config_.node_order()
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }
    pub fn size(&self) -> usize {
        self.terminal_flags_.num_1s()
    }

    fn get_cache_id_with_label(&self, node_id: usize, label: u8) -> usize {
        (node_id ^ (node_id << 5) ^ (label as usize)) & self.cache_mask_
    }

    fn get_cache_id(&self, node_id: usize) -> usize {
        node_id & self.cache_mask_
    }

    // FIXME: is this correct terminology for link_id?
    fn get_link_id(&self, node_id: usize) -> usize {
        self.link_flags_.rank1(node_id)
    }

    fn get_link(&self, node_id: usize) -> usize {
        self.get_link_2(node_id, self.get_link_id(node_id))
    }

    fn get_link_2(&self, node_id: usize, link_id: usize) -> usize {
        ((self.bases_[node_id] as u32) | (self.extras_.at(link_id) * 256))
            as usize
    }
}

/*
    fn total_size() usize {
        louds_.total_size()
        + terminal_flags_.total_size()
        + link_flags_.total_size()
        + bases_.total_size()
        + extras_.total_size()
        + tail_.total_size()
        + if (next_trie_.get() != NULL { next_trie_->total_size() } else { 0 }
        + cache_.total_size()
    }

    fn io_size() -> usize {
        Header().io_size()
        + louds_.io_size()
        + terminal_flags_.io_size()
        + link_flags_.io_size()
        + bases_.io_size()
        + extras_.io_size()
        + tail_.io_size()
//match
        + if next_trie_.get() != NULL { 
             next_trie_->io_size() - Header().io_size() } else { 0 }
        + cache_.io_size()
        + (sizeof(u32) * 2)
    }

    fn clear(&mut self) {
        *self = LoudsTrie::new();
    }

    fn reserve_cache(&mut self, config: &Config, trie_id: usize,
                     num_keys: usize) {
        let cache_size: usize = if trie_id == 1 { 256 } else { 1 };
        while cache_size < (num_keys / config.cache_level()) {
          cache_size *= 2;
        }
        cache_.resize(cache_size);
        cache_mask_ = cache_size - 1;
    }

// FIXME: Use trait for this overloading business
template <>
void LoudsTrie::cache<Key>(usize parent, usize child,
    float weight, char label) {
  MARISA_DEBUG_IF(parent >= child, MARISA_RANGE_ERROR);

  const usize cache_id = get_cache_id(parent, label);
  if (weight > cache_[cache_id].weight()) {
    cache_[cache_id].set_parent(parent);
    cache_[cache_id].set_child(child);
    cache_[cache_id].set_weight(weight);
  }
}

template <>
void LoudsTrie::cache<ReverseKey>(usize parent, usize child,
    float weight, char) {
  MARISA_DEBUG_IF(parent >= child, MARISA_RANGE_ERROR);

  const usize cache_id = get_cache_id(child);
  if (weight > cache_[cache_id].weight()) {
    cache_[cache_id].set_parent(parent);
    cache_[cache_id].set_child(child);
    cache_[cache_id].set_weight(weight);
  }
}

void LoudsTrie::fill_cache() {
  for (usize i = 0; i < cache_.size(); ++i) {
    const usize node_id = cache_[i].child();
    if (node_id != 0) {
      cache_[i].set_base(bases_[node_id]);
      cache_[i].set_extra(!link_flags_[node_id] ?
          MARISA_INVALID_EXTRA : extras_[link_flags_.rank1(node_id)]);
    } else {
      cache_[i].set_parent(MARISA_UINT32_MAX);
      cache_[i].set_child(MARISA_UINT32_MAX);
    }
  }
}



  
 
    inline bool find_child(Agent &agent) const;

bool LoudsTrie::find_child(Agent &agent) const {
  MARISA_DEBUG_IF(agent.state().query_pos() >= agent.query().length(),
      MARISA_BOUND_ERROR);

  State &state = agent.state();
  const usize cache_id = get_cache_id(state.node_id(),
      agent.query()[state.query_pos()]);
  if (state.node_id() == cache_[cache_id].parent()) {
    if (cache_[cache_id].extra() != MARISA_INVALID_EXTRA) {
      if (!match(agent, cache_[cache_id].link())) {
        return false;
      }
    } else {
      state.set_query_pos(state.query_pos() + 1);
    }
    state.set_node_id(cache_[cache_id].child());
    return true;
  }

  usize louds_pos = louds_.select0(state.node_id()) + 1;
  if (!louds_[louds_pos]) {
    return false;
  }
  state.set_node_id(louds_pos - state.node_id() - 1);
  usize link_id = MARISA_INVALID_LINK_ID;
  do {
    if (link_flags_[state.node_id()]) {
      link_id = update_link_id(link_id, state.node_id());
      const usize prev_query_pos = state.query_pos();
      if (match(agent, get_link(state.node_id(), link_id))) {
        return true;
      } else if (state.query_pos() != prev_query_pos) {
        return false;
      }
    } else if (bases_[state.node_id()] ==
        (u8)agent.query()[state.query_pos()]) {
      state.set_query_pos(state.query_pos() + 1);
      return true;
    }
    state.set_node_id(state.node_id() + 1);
    ++louds_pos;
  } while (louds_[louds_pos]);
  return false;
}

    inline usize update_link_id(usize link_id,
        usize node_id) const;

usize LoudsTrie::update_link_id(usize link_id,
    usize node_id) const {
  return (link_id == MARISA_INVALID_LINK_ID) ?
      link_flags_.rank1(node_id) : (link_id + 1);
}

}

*/

/*
    fn map(mapper: &mut Mapper) -> LoudsTrie {
        Header().map(mapper);
    
        let temp: LoudsTrie;
        temp.map_(mapper);
        temp.mapper_.swap(mapper);
        temp
    }

    fn read(Reader &reader) -> LoudsTrie {
        Header().read(reader);
        let temp: LoudsTrie;
        temp.read_(reader);
        temp
    }

    fn write(&self, writer: &mut Writer) {
        Header().write(writer);
        write_(writer);
    }

    void map_(Mapper &mapper);
    void read_(Reader &reader);
    void write_(Writer &writer) const;
 
void LoudsTrie::map_(Mapper &mapper) {
  louds_.map(mapper);
  terminal_flags_.map(mapper);
  link_flags_.map(mapper);
  bases_.map(mapper);
  extras_.map(mapper);
  tail_.map(mapper);
  if ((link_flags_.num_1s() != 0) && tail_.empty()) {
    next_trie_.reset(new (std::nothrow) LoudsTrie);
    MARISA_THROW_IF(next_trie_.get() == NULL, MARISA_MEMORY_ERROR);
    next_trie_->map_(mapper);
  }
  cache_.map(mapper);
  cache_mask_ = cache_.size() - 1;
  {
    u32 temp_num_l1_nodes;
    mapper.map(&temp_num_l1_nodes);
    num_l1_nodes_ = temp_num_l1_nodes;
  }
  {
    u32 temp_config_flags;
    mapper.map(&temp_config_flags);
    config_.parse((int)temp_config_flags);
  }
}

void LoudsTrie::read_(Reader &reader) {
  louds_.read(reader);
  terminal_flags_.read(reader);
  link_flags_.read(reader);
  bases_.read(reader);
  extras_.read(reader);
  tail_.read(reader);
  if ((link_flags_.num_1s() != 0) && tail_.empty()) {
    next_trie_.reset(new (std::nothrow) LoudsTrie);
    MARISA_THROW_IF(next_trie_.get() == NULL, MARISA_MEMORY_ERROR);
    next_trie_->read_(reader);
  }
  cache_.read(reader);
  cache_mask_ = cache_.size() - 1;
  {
    u32 temp_num_l1_nodes;
    reader.read(&temp_num_l1_nodes);
    num_l1_nodes_ = temp_num_l1_nodes;
  }
  {
    u32 temp_config_flags;
    reader.read(&temp_config_flags);
    config_.parse((int)temp_config_flags);
  }
}

void LoudsTrie::write_(Writer &writer) const {
  louds_.write(writer);
  terminal_flags_.write(writer);
  link_flags_.write(writer);
  bases_.write(writer);
  extras_.write(writer);
  tail_.write(writer);
  if (next_trie_.get() != NULL) {
    next_trie_->write_(writer);
  }
  cache_.write(writer);
  writer.write((u32)num_l1_nodes_);
  writer.write((u32)config_.flags());
}
*/

