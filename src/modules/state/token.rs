use std::cell::RefCell;
use std::rc::Rc;
use std::slice;

use super::State;

pub struct TokenState {
    dest: Rc<RefCell<dyn State>>,
    token: char,
}

impl TokenState {
    pub fn new(token: char, dest: Rc<RefCell<dyn State>>) -> Self {
        TokenState { dest, token }
    }
}

impl State for TokenState {
    fn epsilon(&self) -> &[Rc<RefCell<dyn State>>] {
        &([] as [Rc<RefCell<dyn State>>; 0])
    }

    fn transition(&self, token: char) -> Option<Rc<RefCell<dyn State>>> {
        if self.token == token {
            Some(self.dest.clone())
        } else {
            None
        }
    }

    fn get_dest(&self) -> &[Rc<RefCell<dyn State>>] {
        slice::from_ref(&self.dest)
    }

    fn get_state_type(&self) -> String {
        format!("Token State: {}", self.token)
    }
}