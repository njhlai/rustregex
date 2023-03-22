use std::cell::RefCell;
use std::rc::Rc;

use super::State;

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