use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use super::state::{State, TokenState, TrivialState};

pub struct Automata {
    start: Rc<RefCell<dyn State>>,
    end: Rc<RefCell<TrivialState>>,
}

impl Automata {
    pub fn from_token(token: char) -> Self {
        let end = TrivialState::make_rc();
        let start = Rc::new(RefCell::new(TokenState::new(token, end.clone())));
        Automata { start, end }
    }

    fn push_to_end(&self, state: Rc<RefCell<dyn State>>) {
        self.end.borrow_mut().push(state);
    }
}

impl Debug for Automata {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("NFA").field("start", &self.start).finish()
    }
}

pub fn concat(a: Automata, b: Automata) -> Automata {
    a.push_to_end(b.start.clone());
    Automata { start: a.start, end: b.end }
}

pub fn union(a: &Automata, b: &Automata) -> Automata {
    let start = TrivialState::make_rc();
    let end = TrivialState::make_rc();

    start.borrow_mut().push(a.start.clone());
    start.borrow_mut().push(b.start.clone());

    a.push_to_end(end.clone());
    b.push_to_end(end.clone());

    Automata { start, end }
}

pub fn closure(a: &Automata) -> Automata {
    let start = TrivialState::make_rc();
    let end = TrivialState::make_rc();

    start.borrow_mut().push(a.start.clone());
    start.borrow_mut().push(end.clone());

    a.push_to_end(a.start.clone());
    a.push_to_end(end.clone());

    Automata { start, end }
}