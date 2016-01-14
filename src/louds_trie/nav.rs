use std;
use base::*;
use super::{LoudsTrie, NodeID, LoudsPos, LinkID, INVALID_LINK_ID};

struct History<'a> {
    trie_: &'a LoudsTrie,
    node_id_: NodeID,
    louds_pos_: LoudsPos,
    link_id_: LinkID,
    key_pos_: u32,
    //key_id_: u32,
}

impl<'a> History<'a> {
    fn new(trie: &'a LoudsTrie, node_id: NodeID, louds_pos: LoudsPos,
           link_id: LinkID, key_pos: u32) -> History<'a> {
        History { trie_: trie, node_id_: node_id, louds_pos_: louds_pos,
                  link_id_: link_id, key_pos_: key_pos }
    }
}

struct State<'a> {
    history_: Vec<History<'a> >,
    key_buf_: Vec<u8>,
}

impl<'a> State<'a> {
    fn new() -> State<'a> {
        State { history_: Vec::new(), key_buf_: Vec::new() }
    }

    fn push<'b>(&mut self, key: &'b[u8], trie: &'a LoudsTrie, node_id: NodeID,
                louds_pos: LoudsPos, link_id: LinkID) {
        self.key_buf_.extend(key);
        assert!(self.key_buf_.len() <= std::u32::MAX as usize);
        self.history_.push(History::new(trie, node_id, louds_pos, link_id,
                                        self.key_buf_.len() as u32));
    }

    fn get_key(&self) -> &[u8] {
        &self.key_buf_[..]
    }

    fn pop(&mut self) -> Option<History<'a> > {
        self.history_.pop()
    }

    fn get_node_id(&self) -> NodeID {
        self.history_.last().unwrap().node_id_
    }

    fn get_link_id(&self) -> LinkID {
        self.history_.last().unwrap().link_id_
    }
}

pub struct Nav<'a> {
    state_: State<'a>,
    trie_: &'a LoudsTrie,
}

// For lookups, marisa does caching based on the input character.
// We can't do that here. May want to remove or rethink the cache
// implementation in light of this.

impl<'a> Nav<'a> {
    pub fn new(trie: &'a LoudsTrie) -> Nav<'a> {
        Nav { state_: State::new(), trie_: trie }
    }

    pub fn has_child(&self) -> bool {
        self.trie_.has_child(self.state_.get_node_id())
    }
    pub fn go_to_child(&mut self) -> bool {
        let init_node_id = self.state_.get_node_id();
        if let Some((node_id, louds_pos)) = self.trie_.child_pos(init_node_id) {
            let mut trie = self.trie_;
            loop {
                if trie.link_flags_.at(node_id.0 as usize) {
                    let (node_id, link_id) =
                        trie.get_linked_ids(node_id.0 as usize);
                    // Proceed either to next trie or tail
                    match &trie.next_trie_ {
                        &Some(ref next_trie) => {
                            trie = &**next_trie;
                            continue;
                        },
                        &None => {
                            // FIXME: Shouldn't need this temporary vector.
                            //        'restore' should return an iterator, and
                            //        state.push should consume it.
                            let mut v = Vec::new();
                            trie.tail_.restore(node_id.0 as usize, &mut v);

                            // Not sure if these values are correct/useful.
                            // If some stuff is only needed for some nodes...
                            // should reflect that in the types we use
                            self.state_.push(&v, trie, node_id, louds_pos,
                                             link_id);
                        }
                    }
                } else {
                    let node_char = [ trie.bases_[node_id.0 as usize] ];
                    self.state_.push(&node_char, trie, node_id, louds_pos,
                                     INVALID_LINK_ID);
                }
                return true;
            }
        }
        false
    }
    pub fn has_prev_sibling(&self) -> bool {
        // FIXME: Is this all...?
        self.state_.history_.last().map(|h| {
            h.trie_.louds_.at(h.louds_pos_.0 as usize - 1)
        }).unwrap_or(false)
    }
    pub fn go_to_prev_sibling(&mut self) -> bool {
        // pop history and string
        // decrease louds_pos and node_id by 1 if we have a sibling
        // get string (factor this out?)
        // push new history & string

        panic!("not implemented")
    }
    pub fn has_sibling(&self) -> bool {
        // FIXME: Is this all...?
        self.state_.history_.last().map(|h| {
            h.trie_.louds_.at(h.louds_pos_.0 as usize + 1)
        }).unwrap_or(false)
    }
    pub fn go_to_sibling(&mut self) -> bool {
        // pop history and string
        // increase louds_pos and node_id by 1 if we have a sibling
        // get string (factor this out?)
        // push new history & string

        panic!("not implemented")
    }
    pub fn has_parent(&self) -> bool {
        panic!("not implemented")
    }
    pub fn go_to_parent(&self) -> bool {
        panic!("not implemented")
    }
    pub fn is_leaf(&self) -> bool {
        panic!("not implemented")
    }
    pub fn get_string(&self) -> &'a str {
        panic!("not implemented")
    }
    pub fn is_end(&self) -> bool {
        panic!("not implemented")
    }
}

#[derive(Copy, Clone)]
enum DFT {
    ToChild,
    ToSibling,
    ToParentSibling,
    End
}

impl DFT {
    fn new() -> DFT {
        DFT::ToChild
    }
    fn depth_first_traversal_step<'a>(&mut self, nav: &mut Nav<'a>) -> bool {
        match *self {
            DFT::ToChild => {
                if nav.go_to_child() {
                    return true;
                } else {
                    *self = DFT::ToSibling;
                    return false;
                }
            },
            DFT::ToSibling => {
                if nav.go_to_sibling() {
                    *self = DFT::ToChild;
                    return true;
                } else {
                    *self = DFT::ToParentSibling;
                    return false;
                }
            },
            DFT::ToParentSibling => {
                if !nav.go_to_parent() {
                    *self = DFT::End;
                    return false;
                }
                if nav.go_to_sibling() {
                    *self = DFT::ToChild;
                    return true;
                }
                return false;
            },
            DFT::End => {
                return false;
            }
        }
    }
    fn next_terminal<'a>(&mut self, nav: &mut Nav<'a>) -> Option<&'a str> {
        loop {
            match *self {
                DFT::End => { return None; },
                _ => (),
            }
            if self.depth_first_traversal_step(nav) {
                return Some(nav.get_string());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use config::NumTries;
    use super::Nav;
    use super::super::LoudsTrie;

    fn restore_with_nav_prop(v: Vec<String>, num_tries: NumTries)
      -> qc::TestResult {
/*
        if v.iter().any(|x| x.is_empty()) {
            return qc::TestResult::discard();
        }
        let mut keys: Vec<Key> = v.iter().map(|s| Key::new(s.as_bytes()))
                                 .collect();
        let config = Config::new().with_num_tries(num_tries);
        let trie = LoudsTrie::build(&mut keys, &config);

        let nav = Nav::new(&trie);

                return qc::TestResult::failed();
*/
        qc::TestResult::passed()
    }

    #[test]
    fn restore_with_nav_qc() {
        panic!()

    }

    #[test]
    fn restore_with_nav_manual() {
        panic!()

    }
}

