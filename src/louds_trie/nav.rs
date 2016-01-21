use std;
use base::*;
use super::{LoudsTrie, NodeID, LoudsPos, LinkID, INVALID_LINK_ID};

#[derive(Copy, Clone)]
struct State<'a> {
    trie_: &'a LoudsTrie,
    node_id_: NodeID,
    louds_pos_: LoudsPos,
    link_id_: LinkID,
    key_pos_: u32,
    //key_id_: u32,
}

impl<'a> std::fmt::Debug for State<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("State")
            .field("node_id_", &self.node_id_)
            .field("louds_pos_", &self.louds_pos_)
            .field("link_id_", &self.link_id_)
            .field("key_pos_", &self.key_pos_)
            .finish()
    }
}

impl<'a> State<'a> {
    fn new(trie: &'a LoudsTrie, node_id: NodeID, louds_pos: LoudsPos,
           link_id: LinkID, key_pos: u32) -> State<'a> {
        State { trie_: trie, node_id_: node_id, louds_pos_: louds_pos,
                link_id_: link_id, key_pos_: key_pos }
    }
}

#[derive(Debug)]
pub struct Nav<'a> {
    trie_: &'a LoudsTrie,
    history_: Vec<State<'a> >,
    key_buf_: Vec<u8>,
}

// For lookups, marisa does caching based on the input character.
// We can't do that here. May want to remove or rethink the cache
// implementation in light of this.

impl<'a> Nav<'a> {
    fn new(trie: &'a LoudsTrie) -> Nav<'a> {
        let mut out = Nav { trie_: trie, history_: Vec::new(),
                            key_buf_: Vec::new() };
        out.history_.push(State::new(trie, NodeID(0), LoudsPos(0),
                          INVALID_LINK_ID, 0));
        out
    }

    fn get_link_id(&self) -> LinkID {
        self.history_.last().unwrap().link_id_
    }

    fn push_str<'b>(&mut self, key: &'b[u8], trie: &'a LoudsTrie, node_id: NodeID,
                    louds_pos: LoudsPos, link_id: LinkID) {

        debug!("push_str(key: {:?})", key);

        let old_len = self.key_buf_.len();
        self.key_buf_.extend(key);
        assert!(old_len <= std::u32::MAX as usize);
        self.history_.push(State::new(trie, node_id, louds_pos, link_id,
                                      old_len as u32));
    }

    fn push(&mut self, mut node_id: NodeID, louds_pos: LoudsPos) {
        debug!("push (node_id: {:?}, louds_pos: {:?})", node_id, louds_pos);
        let mut trie = self.history_.last().unwrap().trie_;
        loop {
            if trie.link_flags_.at(node_id.0 as usize) {
                let (linked_node_id, link_id) = trie.get_linked_ids(node_id.0
                                                                    as usize);
                debug!("linked_node_id: {:?}, link_id: {:?}",
                       linked_node_id, link_id);
                // Proceed either to next trie or tail
                match &trie.next_trie_ {
                    &Some(ref next_trie) => {
                        debug!("Link TRUE--next trie");
                        trie = &**next_trie;
                        node_id = linked_node_id; // not sure about this
                        continue;
                    },
                    &None => {
                        debug!("Link TRUE--tail");
                        // FIXME: Shouldn't need this temporary vector.
                        //        'restore' should return an iterator, and
                        //        state.push should consume it.
                        let mut v = Vec::new();
                        trie.tail_.restore(linked_node_id.0 as usize, &mut v);

                        // Not sure if these values are correct/useful.
                        // If some stuff is only needed for some nodes...
                        // should reflect that in the types we use
                        self.push_str(&v, trie, node_id, louds_pos, link_id);
                    }
                }
            } else {
                //debug!("Link flags FALSE");
                let node_char = [ trie.bases_[node_id.0 as usize] ];
                self.push_str(&node_char, trie, node_id, louds_pos,
                              INVALID_LINK_ID);
            }
            break;
        }
        debug!("done with push");
    }
    pub fn has_child(&self) -> bool {
        self.history_.last().map(|s| s.trie_.has_child(s.node_id_))
            .unwrap_or(false)
    }
    pub fn go_to_child(&mut self) -> bool {
        debug!("go_to_child");
        if let Some((node_id, louds_pos)) =
            self.history_.last()
            .and_then(|s| { self.trie_.child_pos(s.node_id_) })
        {
            debug!("  (node_id: {:?} louds_pos: {:?})", node_id.0, louds_pos.0);
            self.push(node_id, louds_pos);
            debug!("  true");
            true
        }
        else {
            debug!("  no child");
            false
        }
    }
    pub fn has_prev_sibling(&self) -> bool {
        // FIXME: Is this all...?
        self.history_.last().map(|h| {
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
        self.history_.last().map(|h| {
            h.trie_.louds_.at(h.louds_pos_.0 as usize + 1)
        }).unwrap_or(false)
    }
    pub fn go_to_sibling(&mut self) -> bool {
        debug!("go_to_sibling");
        if let Some(&s) = self.history_.last() {
            let cur_len = self.key_buf_.len();
            assert!((s.key_pos_ as usize) <= cur_len);
            self.key_buf_.truncate(s.key_pos_ as usize);
            if s.trie_.louds_.at(s.louds_pos_.0 as usize + 1) {
                debug!("  (node_id: {:?} louds_pos: {:?})",
                       s.node_id_.0 + 1, s.louds_pos_.0 + 1);
                self.history_.pop();
                // FIXME: What about LinkID?
                self.push(NodeID(s.node_id_.0 + 1),
                          LoudsPos(s.louds_pos_.0 + 1));
                true
            } else {
                debug!("  no sibling");
                false
            }
        } else {
            false
        }
    }
    pub fn has_parent(&self) -> bool {
        panic!("not implemented")
    }
    pub fn go_to_parent(&mut self) -> bool {
        debug!("go_to_parent");
        // Could use LOUDS-trie select1(rank0(m)) to navigate upward (within a
        // single trie), but it's probably more efficient just to keep a stack
        // and pop to go up
        if let Some(s) = self.history_.pop() {
            let cur_len = self.key_buf_.len();
            assert!((s.key_pos_ as usize) <= cur_len);
            self.key_buf_.truncate(s.key_pos_ as usize);
            if let Some(s) = self.history_.last() {
                let node_id = s.node_id_;
                let louds_pos = s.louds_pos_;
                debug!("  (node_id: {:?} louds_pos: {:?})", node_id, louds_pos);
            }
            true
        } else {
            false
        }
    }
    pub fn is_leaf(&self) -> bool {
        self.history_.last().map(|s| {
            // Use root trie
            self.trie_.terminal_flags_.at(s.node_id_.0 as usize)
        }).unwrap_or(false)
    }
    //pub fn get_string(&self) -> &str {
    //    panic!("not implemented")
    //}
    pub fn get_u8(&self) -> &[u8] {
        &self.key_buf_[..]
    }
    pub fn is_end(&self) -> bool {
        panic!("not implemented")
    }
}

#[derive(Copy, Clone, Debug)]
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
    fn next_terminal<'a, 'b>(&mut self, nav: &'b mut Nav<'a>)
      -> Option<&'b[u8]> {
        loop {
            match *self {
                DFT::End => { return None; },
                _ => (),
            }
            if self.depth_first_traversal_step(nav) && nav.is_leaf() {
                return Some(nav.get_u8());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use env_logger;
    use std::cmp::Ordering;
    use quickcheck as qc;
    use config::{Config, NumTries};
    use key::Key;
    use super::{DFT, Nav};
    use super::super::LoudsTrie;

    fn debug_print_louds_bv(mut trie: &LoudsTrie) {
        let mut x = 0;
        loop {
            debug!("{}, {:#?}", x, trie.louds_);
            if let &Some(ref next_trie) = &trie.next_trie_ {
                trie = &**next_trie;
            } else {
                break;
            }
            x += 1;
        }
    }

    fn navr_prop(v: Vec<String>, num_tries: NumTries)
      -> qc::TestResult {
        debug!("in: {:?}", v);
        let mut vu: Vec<Vec<u8>> = Vec::new();
        for s in v.iter() {
            vu.push(From::from(s.as_bytes()));
        }
        debug!("u8: {:?}", vu);

        if v.iter().any(|x| x.is_empty()) {
            //debug!("");
            return qc::TestResult::discard();
        }
        let mut keys: Vec<Key> = v.iter().map(|s| Key::new(s.as_bytes()))
                                 .collect();
        let config = Config::new().with_num_tries(num_tries);
        let trie = LoudsTrie::build(&mut keys, &config);
        //debug!("trie: {:#?}", trie);
        debug_print_louds_bv(&trie);

        let mut nav = Nav::new(&trie);
        let mut dft = DFT::new();

        let mut vv1: Vec<Vec<u8>> = v.iter().map(|s| From::from(s.as_bytes()))
                                    .collect();

        let mut vv2: Vec<Vec<u8>> = Vec::new();
        while let Some(s) = dft.next_terminal(&mut nav) {
            vv2.push(From::from(s));
        }

        vv1.sort();
        vv2.sort();
        //debug!("vv1: {:?}", vv1);
        //debug!("vv2: {:?}", vv2);

        let b = vv1.cmp(&vv2) == Ordering::Equal;
        //debug!("equal: {:?}", b);
        //debug!("");
        qc::TestResult::from_bool(b)
    }

    fn navr_prop_str(v: Vec<&str>, num_tries: NumTries)
      -> qc::TestResult {
        let v_owned: Vec<String> = v.iter().map(|&s| s.to_owned()).collect();
        navr_prop(v_owned, num_tries)
    }

    fn navr_prop_1(v: Vec<String>) -> qc::TestResult {
        navr_prop(v, NumTries::new(1))
    }

    fn navr_prop_str_1(v: Vec<&str>) -> qc::TestResult {
        navr_prop_str(v, NumTries::new(1))
    }

    fn navr_prop_str_2(v: Vec<&str>) -> qc::TestResult {
        navr_prop_str(v, NumTries::new(2))
    }

    #[test]
    fn navr_qc() {
        let _ = env_logger::init();
        qc::quickcheck(navr_prop as fn(Vec<String>, NumTries)
                       -> qc::TestResult);
    }

    #[test]
    fn navr_qc_1() {
        let _ = env_logger::init();
        qc::quickcheck(navr_prop_1 as fn(Vec<String>)
                       -> qc::TestResult);
    }

    fn assert_p(tr: qc::TestResult) {
        assert!(!tr.is_failure());
    }

    #[test]
    fn navr_manual() {
        let _ = env_logger::init();
        //assert_p(navr_prop_str_1(vec!["a"]));
        //assert_p(navr_prop_str_1(vec!["ab"]));
        //assert_p(navr_prop_str_1(vec!["ab"]));
        //assert_p(navr_prop_str_1(vec!["\u{194}\u{128}"]));
        //assert_p(navr_prop_str_1(vec!["Testing"]));
        //assert_p(navr_prop_str_1(vec!["\u{80}"]));
        //assert_p(navr_prop_str_1(vec!["\u{7f}"]));
        //assert_p(navr_prop_str_1(vec!["~"]));
        //assert_p(navr_prop_str_1(vec!["\u{0}"]));
        //assert_p(navr_prop_str_1(vec!["Testing", "T"]));
        //assert_p(navr_prop_str_1(vec!["Testing", "Test"]));
        //assert_p(navr_prop_str_1(vec!["trouble", "Threep"]));
        //assert_p(navr_prop_str_1(vec!["Threep", "Test"]));
        //assert_p(navr_prop_str_1(vec!["trouble", "Threep", "Test"]));
        //assert_p(navr_prop_str_1(
        //    vec!["Testing", "trouble", "Trouble", "Threep", "Test"]));

        //assert_p(navr_prop_str_2(vec!["a"]));
        //assert_p(navr_prop_str_2(vec!["ab"]));
        //assert_p(navr_prop_str_2(vec!["\u{194}\u{128}"]));
        //assert_p(navr_prop_str_2(vec!["Testing"]));
        //assert_p(navr_prop_str_2(vec!["\u{80}"]));
        //assert_p(navr_prop_str_2(vec!["\u{7f}"]));
        //assert_p(navr_prop_str_2(vec!["~"]));
        //assert_p(navr_prop_str_2(vec!["\u{0}"]));
        assert_p(navr_prop_str_2(vec!["Testing", "T"]));
        //assert_p(navr_prop_str_2(vec!["Testing", "Test"]));
        //assert_p(navr_prop_str_2(vec!["trouble", "Threep"]));
        //assert_p(navr_prop_str_2(vec!["Threep", "Test"]));
        //assert_p(navr_prop_str_2(vec!["trouble", "Threep", "Test"]));
        //assert_p(navr_prop_str_2(
        //    vec!["Testing", "trouble", "Trouble", "Threep", "Test"]));
    }
}

