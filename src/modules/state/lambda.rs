use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::{ptr, slice};

use super::{Anchor, State};

pub struct LambdaState {
    dest: Rc<RefCell<dyn State>>,
    lambda: fn(char) -> bool,
}

impl LambdaState {
    pub fn new(lambda: fn(char) -> bool, dest: Rc<RefCell<dyn State>>) -> Self {
        LambdaState { dest, lambda }
    }
}

impl State for LambdaState {
    fn epsilon(&self, _: &[Anchor]) -> &[Rc<RefCell<dyn State>>] {
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
        slice::from_ref(&self.dest)
    }

    fn get_state_type(&self) -> String {
        String::from("Lambda State")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn equals(&self, other: &dyn State) -> bool {
        other
            .as_any()
            .downcast_ref::<LambdaState>()
            .map_or(false, |a| ptr::eq(self, a))
    }
}
