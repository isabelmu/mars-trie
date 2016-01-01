use std;
use base::*;

pub struct History {
    node_id_: u32,
    louds_pos_: u32,
    key_pos_: u32,
    link_id_: u32,
    key_id_: u32,
}

impl History {
    fn new() -> History {
        History { node_id_: 0, louds_pos_: 0, key_pos_: 0,
                  link_id_: INVALID_LINK_ID, key_id_: INVALID_KEY_ID }
    }
    fn set_node_id(&mut self, node_id: usize) {
        assert!(node_id <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.node_id_ = node_id as u32;
    }
    fn set_louds_pos(&mut self, louds_pos: usize) {
        assert!(louds_pos <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.louds_pos_ = louds_pos as u32;
    }
    fn set_key_pos(&mut self, key_pos: usize) {
        assert!(key_pos <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.key_pos_ = key_pos as u32;
    }
    fn set_link_id(&mut self, link_id: usize) {
        assert!(link_id <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.link_id_ = link_id as u32;
    }
    fn set_key_id(&mut self, key_id: usize) {
        assert!(key_id <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.key_id_ = key_id as u32;
    }
    fn node_id(&self) -> usize {
        self.node_id_ as usize
    }
    fn louds_pos(&self) -> usize {
        self.louds_pos_ as usize
    }
    fn key_pos(&self) -> usize {
        self.key_pos_ as usize
    }
    fn link_id(&self) -> usize {
        self.link_id_ as usize
    }
    fn key_id(&self) -> usize {
        self.key_id_ as usize
    }
}

pub struct State {
    key_buf_: Vec<u8>,
    history_: Vec<History>,
    node_id_: u32,
    query_pos_: u32,
    history_pos_: u32,
}

impl State {
    pub fn new() -> State {
        State { key_buf_: Vec::new(), history_: Vec::new(), node_id_: 0,
                query_pos_: 0, history_pos_: 0, }
    }

    fn set_node_id(&mut self, node_id: usize) {
        assert!(node_id <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.node_id_ = node_id as u32;
    }
    fn set_query_pos(&mut self, query_pos: usize) {
        assert!(query_pos <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.query_pos_ = query_pos as u32;
    }
    fn set_history_pos(&mut self, history_pos: usize) {
        assert!(history_pos <= std::u32::MAX as usize, "MARISA_SIZE_ERROR");
        self.history_pos_ = history_pos as u32;
    }

    fn get_node_id(&self) -> usize {
        self.node_id_ as usize
    }
    fn get_query_pos(&self) -> usize {
        self.query_pos_ as usize
    }
    fn get_history_pos(&self) -> usize {
        self.history_pos_ as usize
    }

    fn reset(&mut self) {
        *self = State::new();
    }
}

struct Nav {
    state_: State
}

impl Nav {
    fn has_child(&self) -> bool {
        panic!("not implemented")
    }
    fn go_to_child(&mut self) -> bool {
        panic!("not implemented")
    }
    fn has_sibling(&self) -> bool {
        panic!("not implemented")
    }
    fn go_to_sibling(&mut self) -> bool {
        panic!("not implemented")
    }
    fn has_parent(&self) -> bool {
        panic!("not implemented")
    }
    fn go_to_parent(&self) -> bool {
        panic!("not implemented")
    }
    fn is_terminal(&self) -> bool {
        panic!("not implemented")
    }
    fn get_string(&self) -> &str {
        panic!("not implemented")
    }
    fn is_end(&self) -> bool {
        panic!("not implemented")
    }
}

