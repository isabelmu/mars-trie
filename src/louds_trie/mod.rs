use std;
use std::collections::VecDeque;

use base::INVALID_LINK_ID;
use cache::Cache;
use config::Config;
use config::CacheLevel;
use config::NodeOrder;
use config::TailMode;
use entry::Entry;
use range::Range;
use range::WeightedRange;
use key::IKey;
use key::Key;
use key::ReverseKey;
use louds_trie::tail::Tail;
use vector::bit_vec::BitVec;
use vector::flat_vec::FlatVec;

pub mod nav;
mod tail;

pub const INVALID_EXTRA: u32 = std::u32::MAX >> 8;

struct LoudsPos(u32);
struct NodeID(u32);

/// Recursive LOUDS trie
///
/// The LOUDS (level-order unary degree sequence) representation of a tree
/// structure is as follows. A node's children are represented as one '1'
/// bit per child, followed by a '0'. So three children is '1110', and no
/// children is just '0'. These bit strings are packed together in level
/// order (breadth-first order).
///
/// The tree structure starts with a 'super-root' that is always present,
/// but otherwise ignored. The super-root is always '10', indicating that
/// there is a root node below.
///
/// Example (from Jacobson):
///
///  Tree:              With degrees:
///
///                           10  <-- super-root
///                           |
///                           |
///        o                 1110
///       /|\                /|\  
///      / | \              / | \
///     o  o  o          110  0  10
///    / \     \          / \     \       
///   /   \     \        /   \     \     
///  o     o     o      10   10     0
///  |     |            |     |      
///  |     |            |     |     
///  o     o            0     0
///
///  Degree bit sequences, concatenated in level order:
///
///  10 1110 110 0 10 10 10 0 0 0
///
/// Nodes are represented by the index of their corresponding '1' bit.
/// Traversal operations are as follows:
///
/// first_child(m) == select0(rank1(m)) + 1
/// next_sibling(m) == m + 1
/// parent(m) == select1(rank0(m))
///
/// In marisa-trie terminology, "node_id" is the index of the node in level
/// order. This is equal to rank1(m). So the traversal operations become:
///
/// first_child_m(node_id) == select0(node_id) + 1
/// first_child_node_id(node_id) == first_child_m(node_id) - (node_id + 1)
///     Because node_id + 1 is the # of 0s ahead of m, and we're looking for
///     the # of 1s.
///     Note that we first need to check whether the bit at
///     'first_child_m(node_id)' is set.
/// next_sibling_node_id(node_id) == node_id + 1
///     (but we need to check whether m + 1
///
/// 'louds_pos' variables refer to bit indexes in 'louds_'
///
#[derive(Debug)]
pub struct LoudsTrie {
    /// The tree structure
    louds_: BitVec,

    /// Bit vector of terminal-ness per node id. Indexed by node. Can be used to
    /// retrieve NodeID from user-facing word ID:
    ///
    ///     let node_id = self.terminal_flags_.select1(id);
    ///
    terminal_flags_: BitVec,

    /// Per node, does this node have a link to another trie? Indexed by node.
    link_flags_: BitVec,

    /// Base characters, limited to one per node. Indexed by node.
    bases_: Vec<u8>,
    extras_: FlatVec,

    /// Tail strings, accessed by link ID.
    tail_: Tail,
    next_trie_: Option<Box<LoudsTrie> >,
    cache_: Vec<Cache>,
    cache_mask_: usize,
    num_l1_nodes_: usize,
    config_: Config,
//    mapper_: Mapper,
}

trait CallBuildNextTrie {
    fn build_next_trie(&mut self, louds_trie: &mut LoudsTrie,
                       terminals: &mut Vec<u32>, config: &mut Config,
                       trie_id: usize);
}

impl<'a> CallBuildNextTrie for Vec<Key<'a>> {
    fn build_next_trie(&mut self, louds_trie: &mut LoudsTrie,
                       terminals: &mut Vec<u32>, config: &mut Config,
                       trie_id: usize) {
        louds_trie.build_next_trie_fwd(self, terminals, config, trie_id);
    }
}

impl<'a> CallBuildNextTrie for Vec<ReverseKey<'a>> {
    fn build_next_trie(&mut self, louds_trie: &mut LoudsTrie,
                       terminals: &mut Vec<u32>, config: &mut Config,
                       trie_id: usize) {
        louds_trie.build_next_trie_rev(self, terminals, config, trie_id);
    }
}

trait CallCache {
    fn cache(&self, louds_trie: &mut LoudsTrie, parent: usize, child: usize,
             weight: f32, label: u8);
}

impl<'a> CallCache for Vec<Key<'a>> {
    fn cache(&self, louds_trie: &mut LoudsTrie, parent: usize, child: usize,
             weight: f32, label: u8) {
        louds_trie.cache_fwd(parent, child, weight, label);
    }
}

impl<'a> CallCache for Vec<ReverseKey<'a>> {
    fn cache(&self, louds_trie: &mut LoudsTrie, parent: usize, child: usize,
             weight: f32, _: u8) {
        louds_trie.cache_rev(parent, child, weight);
    }
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

    pub fn clear(&mut self) {
        *self = LoudsTrie::new();
    }

    pub fn has_child(&self, node_id: NodeID) -> bool {
        self.child_pos(node_id).is_some()
    }
    pub fn child_pos(&self, node_id: NodeID) -> Option<(NodeID, LoudsPos)> {
        let child_louds_pos = self.trie_.louds_.select0(node_id) + 1;
        if self.trie_.louds_.at(louds_pos) {
            let child_node_id = child_louds_pos - node_id - 1;
            Some((NodeID(child_node_id), LoudsPos(child_louds_pos)))
        } else {
            None
        }
    }

    pub fn build<'a>(keys: &mut Vec<Key<'a> >, config: &Config) -> LoudsTrie {
        let mut config = *config;
        let mut out = LoudsTrie::new();

        let mut keys_cpy = keys.clone();
        let mut terminals: Vec<u32> = Vec::new();
        out.build_trie(&mut keys_cpy, &mut terminals, &mut config, 1);

        let mut pairs: Vec<(u32, u32)> = Vec::new();
        let mut pairs: Vec<(u32, u32)> = terminals.iter().enumerate()
                                         .map(|(i, &x)| (x, i as u32))
                                         .collect();
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

        assert!(pairs.len() == keys.len());
        for pair in &pairs {
            keys[pair.1 as usize].set_id(
                out.terminal_flags_.rank1(pair.0 as usize));
        }
        out
    }

    fn build_trie<'a, T>(
        &mut self, keys: &mut Vec<T>, terminals: &mut Vec<u32>,
        config: &mut Config, trie_id: usize)
        where T: IKey<'a> + Ord + From<&'a[u8]>,
              Vec<T>: CallCache + CallBuildNextTrie
    {
        self.build_current_trie(keys, terminals, config, trie_id);

        let mut next_terminals: Vec<u32> = Vec::new();
        if !keys.is_empty() {
            keys.build_next_trie(self, &mut next_terminals, config, trie_id);
        }

        match &self.next_trie_ {
            &Some(ref x) => {
                let new_cfg = (x.num_tries() + 1) as usize
                            | x.tail_mode() as usize
                            | x.node_order() as usize;
                assert!(new_cfg <= std::u32::MAX as usize);
                *config = Config::parse(new_cfg as u32);
            },
            &None => {
                let new_cfg = 1
                            | self.tail_.mode() as usize
                            | config.node_order() as usize
                            | config.cache_level() as usize;
                *config = Config::parse(new_cfg as u32);
            }
        }
        self.link_flags_.build(false, false);
        let mut node_id: usize = 0;
        for nt in next_terminals.iter_mut() {
            while !self.link_flags_.at(node_id) {
                node_id += 1;
            }
            self.bases_[node_id] = (*nt % 256) as u8;
            *nt /= 256;
            node_id += 1;
        }
        self.extras_.build(next_terminals.iter());
        self.fill_cache();
    }

    fn build_current_trie<'a, T>(
        &mut self, keys: &mut Vec<T>, terminals: &mut Vec<u32>, config: &Config,
        trie_id: usize)
        where T: IKey<'a> + Ord + From<&'a[u8]>, Vec<T>: CallCache
    {
        for (i, key) in keys.iter_mut().enumerate() {
            key.set_id(i);
        }
        // FIXME: sort fn
        keys.sort();
        let num_keys = keys.len();

        self.reserve_cache(config, trie_id, num_keys);
        self.louds_.push(true);
        self.louds_.push(false);
        self.bases_.push(0);
        self.link_flags_.push(false);

        let mut next_keys: Vec<T> = Vec::new();
        let mut queue: VecDeque<Range> = VecDeque::new();
        let mut w_ranges: Vec<WeightedRange> = Vec::new();

        queue.push_back(Range::new(0, keys.len(), 0));

        while let Some(mut range) = queue.pop_front() {
            let node_id: usize = self.link_flags_.len() - queue.len() - 1;

            while (range.begin() < range.end()) &&
                  (keys[range.begin()].len() == range.key_pos()) {
                keys[range.begin()].set_terminal(node_id);
                let new_begin = range.begin() + 1;
                range.set_begin(new_begin);
            }

            if range.begin() == range.end() {
                self.louds_.push(false);
                continue;
            }

            w_ranges.clear();
            let mut weight: f64 = keys[range.begin()].get_weight() as f64;
            for i in (range.begin() + 1)..range.end() {
                if keys[i - 1].at(range.key_pos())
                != keys[i].at(range.key_pos()) {
                    w_ranges.push(WeightedRange::new(
                        range.begin(), i, range.key_pos(), weight as f32));
                    range.set_begin(i);
                    weight = 0.0;
                }
                weight += keys[i].get_weight() as f64;
            }
            w_ranges.push(WeightedRange::new(
                range.begin(), range.end(), range.key_pos(), weight as f32));
            if config.node_order() == NodeOrder::Weight {
                // FIXME: This should be a std::stable_sort replacement. Not
                //        sure if really needed... but I would guess it's not
                //        here for no reason.
                w_ranges.sort_by(|a, b| b.partial_cmp(a).unwrap()); // reverse
            }

            if node_id == 0 {
                self.num_l1_nodes_ = w_ranges.len();
            }

            for w_range in &mut w_ranges {
                let mut key_pos: usize = w_range.key_pos() + 1;
                'l2: while key_pos < keys[w_range.begin()].len() {
                    for j in (w_range.begin() + 1)..w_range.end() {
                        if keys[j - 1].at(key_pos) != keys[j].at(key_pos) {
                            break 'l2;
                        }
                    }
                    key_pos += 1;
                }
                let bases_len = self.bases_.len();
                keys.cache(self, node_id, bases_len, w_range.weight(),
                           keys[w_range.begin()].at(w_range.key_pos()));

                if key_pos == w_range.key_pos() + 1 {
                    self.bases_.push(keys[w_range.begin()]
                                     .at(w_range.key_pos()));
                    self.link_flags_.push(false);
                } else {
                    self.bases_.push(0);
                    self.link_flags_.push(true);
                    let mut next_key =
                        T::from(keys[w_range.begin()].get_slice());
                    next_key.subslice(w_range.key_pos(),
                                      key_pos - w_range.key_pos());
                    next_key.set_weight(w_range.weight());
                    next_keys.push(next_key);
                }
                w_range.set_key_pos(key_pos);
                queue.push_back(*w_range.range());
                self.louds_.push(true);
            }
            self.louds_.push(false);
        }

        self.louds_.push(false);
        self.louds_.build(trie_id == 1, true);
        self.bases_.shrink_to_fit();

        self.build_terminals(keys, terminals);
        *keys = next_keys;
    }

    fn cache_fwd(&mut self, parent: usize, child: usize, weight: f32, label: u8)
    {
        assert!(parent < child, "MARISA_RANGE_ERROR");
        let cache_id = self.get_cache_id_with_label(parent, label);
        if weight > self.cache_[cache_id].weight() {
            assert!(parent <= std::u32::MAX as usize);
            assert!(child <= std::u32::MAX as usize);
            self.cache_[cache_id].set_parent(parent as u32);
            self.cache_[cache_id].set_child(child as u32);
            self.cache_[cache_id].set_weight(weight);
        }
    }

    fn cache_rev(&mut self, parent: usize, child: usize, weight: f32) {
        assert!(parent < child, "MARISA_RANGE_ERROR");
        let cache_id = self.get_cache_id(child);
        if weight > self.cache_[cache_id].weight() {
            assert!(parent <= std::u32::MAX as usize);
            assert!(child <= std::u32::MAX as usize);
            self.cache_[cache_id].set_parent(parent as u32);
            self.cache_[cache_id].set_child(child as u32);
            self.cache_[cache_id].set_weight(weight);
        }
    }
 
    fn reserve_cache(&mut self, config: &Config, trie_id: usize,
                     num_keys: usize) {
        let mut cache_size: usize = if trie_id == 1 { 256 } else { 1 };
        while cache_size < (num_keys / config.cache_level() as usize) {
            cache_size *= 2;
        }
        self.cache_.resize(cache_size, Cache::new());
        self.cache_mask_ = cache_size - 1;
    }

    fn build_tail<'a, T: Ord + IKey<'a>>(&mut self, keys: &Vec<T>,
                                         terminals: &mut Vec<u32>,
                                         config: &mut Config) {
        let mut entries: Vec<Entry<'a>> = Vec::new();
        entries.reserve(keys.len());
        for key in keys {
            entries.push(Entry::new(key.get_slice(), 0));
        }
        self.tail_ = Tail::build(&mut entries, terminals, config.tail_mode());
    }

    fn build_next_trie_fwd<'a>(&mut self, keys: &mut Vec<Key<'a>>,
                               terminals: &mut Vec<u32>,
                               config: &mut Config, trie_id: usize) {
        if trie_id == config.num_tries().get() as usize {
            self.build_tail(keys, terminals, config);
        } else {
            let mut reverse_keys: Vec<ReverseKey> = Vec::new();
            reverse_keys.reserve(keys.len());
            for key in keys.iter_mut() {
                reverse_keys.push(ReverseKey::from_key(key));
            }
            keys.clear();
            self.next_trie_ = Some(Box::new(LoudsTrie::new()));
            let mut next_trie = self.next_trie_.as_mut().unwrap();
            next_trie.build_trie(&mut reverse_keys, terminals, config,
                                 trie_id + 1);
        }
    }

    fn build_next_trie_rev<'a>(&mut self, keys: &mut Vec<ReverseKey<'a>>,
                               terminals: &mut Vec<u32>,
                               config: &mut Config, trie_id: usize) {
        if trie_id == config.num_tries().get() as usize {
            self.build_tail(keys, terminals, config);
        } else {
            self.next_trie_ = Some(Box::new(LoudsTrie::new()));
            let mut next_trie = self.next_trie_.as_mut().unwrap();
            next_trie.build_trie(keys, terminals, config, trie_id + 1);
        }
    }

    fn build_terminals<'a, T>(&mut self, keys: &Vec<T>,
                              terminals: &mut Vec<u32>)
      where T: IKey<'a> + Ord + From<&'a[u8]> {
        let mut temp: Vec<u32> = Vec::new();
        temp.resize(keys.len(), 0);
        for key in keys {
            temp[key.get_id()] = key.get_terminal() as u32;
        }
        *terminals = temp;
    }

    fn fill_cache(&mut self) {
        for item in (&mut self.cache_).iter_mut() {
            let node_id = item.child() as usize;
            if node_id != 0 {
                item.set_base(self.bases_[node_id]);
                let new_extra = if self.link_flags_.at(node_id)
                    { self.extras_.at(self.link_flags_.rank1(node_id)) }
                    else { INVALID_EXTRA };
                item.set_extra(new_extra);
            } else {
                item.set_parent(std::u32::MAX);
                item.set_child(std::u32::MAX);
            }
        }
    }

    pub fn id_lookup(&self, id: usize) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        self.id_lookup_into_vec(id, &mut v);
        v
    }

    pub fn id_lookup_into_vec(&self, id: usize, key_out: &mut Vec<u8>) {
        assert!(id < self.len());
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
        self.config_.num_tries().get() as usize
    }
    fn num_keys(&self) -> usize {
        self.len()
    }
    fn num_nodes(&self) -> usize {
        (self.louds_.len() / 2) - 1
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
        self.len() == 0
    }
    pub fn len(&self) -> usize {
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

    fn update_link_id(&self, link_id: usize, node_id: usize) -> usize {
        if link_id == INVALID_LINK_ID as usize
        {
            self.link_flags_.rank1(node_id)
        } else {
            link_id + 1
        }
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

#[cfg(test)]
mod test {
    use env_logger;
    use config::{Config, MAX_NUM_TRIES, MIN_NUM_TRIES, NumTries};
    use key::Key;
    use key::IKey;
    use quickcheck as qc;
    use std;
    use std::default::Default;
    use super::LoudsTrie;

    impl qc::Arbitrary for NumTries {
        fn arbitrary<G: qc::Gen>(g: &mut G) -> NumTries {
            // This is slow when using the full range...
            //NumTries::new(g.gen_range(MIN_NUM_TRIES, MAX_NUM_TRIES + 1))
            NumTries::new(g.gen_range(MIN_NUM_TRIES, 17))
        }
        fn shrink(&self) -> Box<Iterator<Item=Self>> {
            let fewer = self.get() / 2;
            let fewer = NumTries::new(fewer);
            if fewer.get() > 0 { qc::single_shrinker(fewer) }
                else { qc::empty_shrinker() }
        }
    }

    fn build_prop(v: Vec<String>, num_tries: NumTries) -> qc::TestResult {
        if v.iter().any(|x| x.is_empty()) {
            return qc::TestResult::discard();
        }
        let mut keys: Vec<Key> = v.iter().map(|s| Key::new(s.as_bytes()))
                                 .collect();
        let config = Config::new().with_num_tries(num_tries);
        let trie = LoudsTrie::build(&mut keys, &config);
        let mut ids_seen = Vec::new();
        for key in keys {
            ids_seen.push(key.get_id());
            let s = trie.id_lookup(key.get_id());
            if !s.iter().eq(key.get_slice().iter()) {
                return qc::TestResult::failed();
            }
        }
        ids_seen.sort();
        for (&a, b) in ids_seen.iter().zip(0..ids_seen.len()) {
            if a != b { return qc::TestResult::failed(); }
        }
        qc::TestResult::passed()
    }

    #[test]
    fn louds_trie_build_qc() {
        let _ = env_logger::init();
        qc::quickcheck(build_prop as fn(Vec<String>, NumTries)
                       -> qc::TestResult);
    }

    #[test]
    fn louds_trie_build_manual() {
        let _ = env_logger::init();
        let n = NumTries::default();
        assert!(!build_prop(vec!["\u{80}".to_string()], n).is_failure());
        assert!(!build_prop(vec!["\u{4b8ca}".to_string(),
                                 "\u{d2c4a}".to_string()], n).is_failure());
    }

/*
void TestTextTail() {
  TEST_START();

  marisa::grimoire::trie::Tail tail;
  marisa::grimoire::Vector<marisa::grimoire::trie::Entry> entries;
  marisa::grimoire::Vector<marisa::UInt32> offsets;
  tail.build(entries, &offsets, MARISA_TEXT_TAIL);

  ASSERT(tail.mode() == MARISA_TEXT_TAIL);
  ASSERT(tail.size() == 0);
  ASSERT(tail.empty());
  ASSERT(tail.total_size() == tail.size());
  ASSERT(tail.io_size() == (sizeof(marisa::UInt64) * 6));

  ASSERT(offsets.empty());

  marisa::grimoire::trie::Entry entry;
  entry.set_str("X", 1);
  entries.push_back(entry);

  tail.build(entries, &offsets, MARISA_TEXT_TAIL);

  ASSERT(tail.mode() == MARISA_TEXT_TAIL);
  ASSERT(tail.size() == 2);
  ASSERT(!tail.empty());
  ASSERT(tail.total_size() == tail.size());
  ASSERT(tail.io_size() == (sizeof(marisa::UInt64) * 7));

  ASSERT(offsets.size() == entries.size());
  ASSERT(offsets[0] == 0);
  ASSERT(tail[offsets[0]] == 'X');
  ASSERT(tail[offsets[0] + 1] == '\0');

  entries.clear();
  entry.set_str("abc", 3);
  entries.push_back(entry);
  entry.set_str("bc", 2);
  entries.push_back(entry);
  entry.set_str("abc", 3);
  entries.push_back(entry);
  entry.set_str("c", 1);
  entries.push_back(entry);
  entry.set_str("ABC", 3);
  entries.push_back(entry);
  entry.set_str("AB", 2);
  entries.push_back(entry);

  tail.build(entries, &offsets, MARISA_TEXT_TAIL);
  std::sort(entries.begin(), entries.end(),
      marisa::grimoire::trie::Entry::IDComparer());

  ASSERT(tail.size() == 11);
  ASSERT(offsets.size() == entries.size());
  for (std::size_t i = 0; i < entries.size(); ++i) {
    const char * const ptr = &tail[offsets[i]];
    ASSERT(std::strlen(ptr) == entries[i].length());
    ASSERT(std::strcmp(ptr, entries[i].ptr()) == 0);
  }

  {
    marisa::grimoire::Writer writer;
    writer.open("trie-test.dat");
    tail.write(writer);
  }

  tail.clear();

  ASSERT(tail.size() == 0);
  ASSERT(tail.total_size() == tail.size());

  {
    marisa::grimoire::Mapper mapper;
    mapper.open("trie-test.dat");
    tail.map(mapper);

    ASSERT(tail.mode() == MARISA_TEXT_TAIL);
    ASSERT(tail.size() == 11);
    for (std::size_t i = 0; i < entries.size(); ++i) {
      const char * const ptr = &tail[offsets[i]];
    ASSERT(std::strlen(ptr) == entries[i].length());
    ASSERT(std::strcmp(ptr, entries[i].ptr()) == 0);
    }
    tail.clear();
  }

  {
    marisa::grimoire::Reader reader;
    reader.open("trie-test.dat");
    tail.read(reader);
  }

  ASSERT(tail.size() == 11);
  ASSERT(offsets.size() == entries.size());
  for (std::size_t i = 0; i < entries.size(); ++i) {
    const char * const ptr = &tail[offsets[i]];
    ASSERT(std::strlen(ptr) == entries[i].length());
    ASSERT(std::strcmp(ptr, entries[i].ptr()) == 0);
  }

  {
    std::stringstream stream;
    marisa::grimoire::Writer writer;
    writer.open(stream);
    tail.write(writer);
    tail.clear();
    marisa::grimoire::Reader reader;
    reader.open(stream);
    tail.read(reader);
  }

  ASSERT(tail.size() == 11);
  ASSERT(offsets.size() == entries.size());
  for (std::size_t i = 0; i < entries.size(); ++i) {
    const char * const ptr = &tail[offsets[i]];
    ASSERT(std::strlen(ptr) == entries[i].length());
    ASSERT(std::strcmp(ptr, entries[i].ptr()) == 0);
  }

  TEST_END();
}

void TestBinaryTail() {
  TEST_START();

  marisa::grimoire::trie::Tail tail;
  marisa::grimoire::Vector<marisa::grimoire::trie::Entry> entries;
  marisa::grimoire::Vector<marisa::UInt32> offsets;
  tail.build(entries, &offsets, MARISA_BINARY_TAIL);

  ASSERT(tail.mode() == MARISA_TEXT_TAIL);
  ASSERT(tail.size() == 0);
  ASSERT(tail.empty());
  ASSERT(tail.total_size() == tail.size());
  ASSERT(tail.io_size() == (sizeof(marisa::UInt64) * 6));

  ASSERT(offsets.empty());

  marisa::grimoire::trie::Entry entry;
  entry.set_str("X", 1);
  entries.push_back(entry);

  tail.build(entries, &offsets, MARISA_BINARY_TAIL);

  ASSERT(tail.mode() == MARISA_BINARY_TAIL);
  ASSERT(tail.size() == 1);
  ASSERT(!tail.empty());
  ASSERT(tail.total_size() == (tail.size() + sizeof(marisa::UInt64)));
  ASSERT(tail.io_size() == (sizeof(marisa::UInt64) * 8));

  ASSERT(offsets.size() == entries.size());
  ASSERT(offsets[0] == 0);

  const char binary_entry[] = { 'N', 'P', '\0', 'T', 'r', 'i', 'e' };
  entries[0].set_str(binary_entry, sizeof(binary_entry));

  tail.build(entries, &offsets, MARISA_TEXT_TAIL);

  ASSERT(tail.mode() == MARISA_BINARY_TAIL);
  ASSERT(tail.size() == entries[0].length());

  ASSERT(offsets.size() == entries.size());
  ASSERT(offsets[0] == 0);

  entries.clear();
  entry.set_str("abc", 3);
  entries.push_back(entry);
  entry.set_str("bc", 2);
  entries.push_back(entry);
  entry.set_str("abc", 3);
  entries.push_back(entry);
  entry.set_str("c", 1);
  entries.push_back(entry);
  entry.set_str("ABC", 3);
  entries.push_back(entry);
  entry.set_str("AB", 2);
  entries.push_back(entry);

  tail.build(entries, &offsets, MARISA_BINARY_TAIL);
  std::sort(entries.begin(), entries.end(),
      marisa::grimoire::trie::Entry::IDComparer());

  ASSERT(tail.mode() == MARISA_BINARY_TAIL);
  ASSERT(tail.size() == 8);
  ASSERT(offsets.size() == entries.size());
  for (std::size_t i = 0; i < entries.size(); ++i) {
    const char * const ptr = &tail[offsets[i]];
    ASSERT(std::memcmp(ptr, entries[i].ptr(), entries[i].length()) == 0);
  }

  TEST_END();
}
*/
}

