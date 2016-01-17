use std;
use base::*;
use super::{LoudsTrie, NodeID, LoudsPos, LinkID, INVALID_LINK_ID};

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
    history_: Vec<State<'a> >,
    key_buf_: Vec<u8>,
}

// For lookups, marisa does caching based on the input character.
// We can't do that here. May want to remove or rethink the cache
// implementation in light of this.

impl<'a> Nav<'a> {
    fn new(trie: &'a LoudsTrie) -> Nav<'a> {
        let mut out = Nav { history_: Vec::new(), key_buf_: Vec::new() };
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

//        debug!("push_str(key: {:#?}, node_id: {:#?}, louds_pos: {:#?}, \
//                link_id: {:#?}", key, node_id, louds_pos, link_id);

        let old_len = self.key_buf_.len();
        self.key_buf_.extend(key);
        assert!(old_len <= std::u32::MAX as usize);
        self.history_.push(State::new(trie, node_id, louds_pos, link_id,
                                      old_len as u32));
    }

    fn push(&mut self, mut node_id: NodeID, louds_pos: LoudsPos) {
//        debug!("push");
        let mut trie = self.history_.last().unwrap().trie_;
        loop {
            if trie.link_flags_.at(node_id.0 as usize) {
                debug!("Link flags TRUE");
                let (linked_node_id, link_id) = trie.get_linked_ids(node_id.0
                                                                    as usize);
                // Proceed either to next trie or tail
                match &trie.next_trie_ {
                    &Some(ref next_trie) => {
                        panic!();
                        trie = &**next_trie;
                        node_id = linked_node_id; // not sure about this
                        continue;
                    },
                    &None => {
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
                debug!("Link flags FALSE");
                let node_char = [ trie.bases_[node_id.0 as usize] ];
                self.push_str(&node_char, trie, node_id, louds_pos,
                              INVALID_LINK_ID);
            }
            break;
        }
    }
    pub fn has_child(&self) -> bool {
        self.history_.last().map(|s| s.trie_.has_child(s.node_id_))
            .unwrap_or(false)
    }
    pub fn go_to_child(&mut self) -> bool {
        debug!("go_to_child");
        if let Some((node_id, louds_pos)) =
            self.history_.last()
            .and_then(|s| { s.trie_.child_pos(s.node_id_) })
        {
//            debug!("(node_id: {:#?} louds_pos: {:#?})", node_id, louds_pos);
            self.push(node_id, louds_pos);
            true
        }
        else {
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
        if self.history_.len() < 2 {
            return false;
        }
        if let Some(s) = self.history_.pop() {
            let cur_len = self.key_buf_.len();
            debug!("s.key_pos_: {:?}, cur_len: {:?}", s.key_pos_, cur_len);
            assert!((s.key_pos_ as usize) < cur_len);
            self.key_buf_.truncate(s.key_pos_ as usize);
            if s.trie_.louds_.at(s.louds_pos_.0 as usize + 1) {
                // FIXME: What about LinkID?
                self.push(NodeID(s.node_id_.0 + 1),
                          LoudsPos(s.louds_pos_.0 + 1));
                true
            } else {
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

        if self.history_.len() == 1 {
            self.history_.pop();
            return false;
        }
        
        if let Some(s) = self.history_.pop() {
            let cur_len = self.key_buf_.len();
            assert!((s.key_pos_ as usize) < cur_len);
            self.key_buf_.truncate(s.key_pos_ as usize);
            true
        } else {
            false
        }
    }
    pub fn is_leaf(&self) -> bool {
        //debug!("is_leaf");
        self.history_.last().map(|s|
            s.trie_.terminal_flags_.at(s.node_id_.0 as usize)
        ).unwrap_or(false)
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
//                    debug!("{:#?}, {:#?}", *self, *nav);
                    //if nav.history_.len() == 3 { panic!() }

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

    fn nav_restore_prop(v: Vec<String>, num_tries: NumTries)
      -> qc::TestResult {
        debug!("in: {:?}", v);
        let mut vu: Vec<Vec<u8>> = Vec::new();
        for s in v.iter() {
            vu.push(From::from(s.as_bytes()));
        }
        debug!("u8: {:?}", vu);

        if v.iter().any(|x| x.is_empty()) {
            debug!("");
            return qc::TestResult::discard();
        }
        let mut keys: Vec<Key> = v.iter().map(|s| Key::new(s.as_bytes()))
                                 .collect();
        let config = Config::new().with_num_tries(num_tries);
        let trie = LoudsTrie::build(&mut keys, &config);

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
        debug!("vv1: {:?}", vv1);
        debug!("vv2: {:?}", vv2);

        let b = vv1.cmp(&vv2) == Ordering::Equal;
        debug!("equal: {:?}", b);
        debug!("");
        qc::TestResult::from_bool(b)
    }

    fn nav_restore_prop_1(v: Vec<String>) -> qc::TestResult {
        nav_restore_prop(v, NumTries::new(1))
    }

    #[test]
    fn nav_restore_qc() {
        let _ = env_logger::init();
        qc::quickcheck(nav_restore_prop as fn(Vec<String>, NumTries)
                       -> qc::TestResult);
    }

    #[test]
    fn nav_restore_qc_1() {
        let _ = env_logger::init();
        qc::quickcheck(nav_restore_prop_1 as fn(Vec<String>)
                       -> qc::TestResult);
    }

    fn assert_passed(tr: qc::TestResult) {
        assert!(!tr.is_failure());
    }

    #[test]
    fn nav_restore_manual() {
        let _ = env_logger::init();
        //assert_passed(nav_restore_prop_1(vec!["a".to_owned()]));
        //assert_passed(nav_restore_prop_1(vec!["ab".to_owned()]));
        //assert_passed(nav_restore_prop_1(vec!["\u{194}\u{128}".to_owned()]));
        //assert_passed(nav_restore_prop_1(vec!["Testing".to_owned()]));
        //assert_passed(nav_restore_prop_1(vec!["\u{80}".to_owned()]));
        //assert_passed(nav_restore_prop_1(vec!["\u{7f}".to_owned()]));
        //assert_passed(nav_restore_prop_1(vec!["~".to_owned()]));
        //assert_passed(nav_restore_prop_1(vec!["\u{0}".to_owned()]));
        //assert_passed(nav_restore_prop_1(vec!["Testing".to_owned(),
        //                                      "T".to_owned()]));
        //assert_passed(nav_restore_prop_1(vec!["Testing".to_owned(),
        //                                      "Test".to_owned()]));
        assert_passed(nav_restore_prop_1(vec![
                                              "Threep".to_owned(),
                                              "Test".to_owned()]));
        //assert_passed(nav_restore_prop_1(vec!["trouble".to_owned(),
        //                                      "Threep".to_owned(),
        //                                      "Test".to_owned()]));
        //assert_passed(nav_restore_prop_1(vec!["Testing".to_owned(),
        //                                      "trouble".to_owned(),
        //                                      "Trouble".to_owned(),
        //                                      "Threep".to_owned(),
        //                                      "Test".to_owned()]));
    }
}

