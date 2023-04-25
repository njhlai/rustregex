use std::any::Any;
use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;

use super::{Anchor, State};

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
    fn epsilon(&self, _: &[Anchor]) -> &[Rc<RefCell<dyn State>>] {
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn equals(&self, other: &dyn State) -> bool {
        other
            .as_any()
            .downcast_ref::<TrivialState>()
            .map_or(false, |a| ptr::eq(self, a))
    }
}
