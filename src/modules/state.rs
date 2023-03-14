use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use std::ptr;
use std::rc::Rc;
use std::slice::from_ref;

pub trait State {
    fn epsilon(&self) -> &[Rc<RefCell<dyn State>>];

    fn transition(&self, token: char) -> Option<Rc<RefCell<dyn State>>>;

    fn get_dest(&self) -> &[Rc<RefCell<dyn State>>];

    fn get_state_type(&self) -> String;
}

impl Debug for dyn State {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("State")
            .field("type", &self.get_state_type())
            .field("dest", &self.get_dest())
            .finish()
    }
}

impl PartialEq for dyn State {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

#[derive(Default)]
pub struct TrivialState {
    dest: Vec<Rc<RefCell<dyn State>>>,
}

impl TrivialState {
    pub fn make_rc() -> Rc<RefCell<TrivialState>> {
        Rc::new(RefCell::new(TrivialState::default()))
    }

    pub fn push(&mut self, state: Rc<RefCell<dyn State>>) {
        self.dest.push(state);
    }
}

impl State for TrivialState {
    fn epsilon(&self) -> &[Rc<RefCell<dyn State>>] {
        &self.dest[..]
    }

    fn transition(&self, _: char) -> Option<Rc<RefCell<dyn State>>> {
        None
    }

    fn get_dest(&self) -> &[Rc<RefCell<dyn State>>] {
        &self.dest[..]
    }

    fn get_state_type(&self) -> String {
        String::from("Trivial State")
    }
}

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
        from_ref(&self.dest)
    }

    fn get_state_type(&self) -> String {
        format!("Token State: {}", self.token)
    }
}

pub struct LambdaState {
    pub dest: Rc<RefCell<dyn State>>,
    lambda: fn(char) -> bool,
}

impl LambdaState {
    pub fn new(lambda: fn(char) -> bool, dest: Rc<RefCell<dyn State>>) -> Self {
        LambdaState { dest, lambda }
    }
}

impl State for LambdaState {
    fn epsilon(&self) -> &[Rc<RefCell<dyn State>>] {
        &([] as [Rc<RefCell<dyn State>>; 0])
    }

    fn transition(&self, token: char) -> Option<Rc<RefCell<dyn State>>> {
        if (self.lambda)(token) {
            Some(self.dest.clone())
        } else {
            None
        }
    }

    fn get_dest(&self) -> &[Rc<RefCell<dyn State>>] {
        from_ref(&self.dest)
    }

    fn get_state_type(&self) -> String {
        String::from("Lambda State")
    }
}