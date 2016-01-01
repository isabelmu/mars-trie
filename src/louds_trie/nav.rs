use history::State;
use history::History;

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

