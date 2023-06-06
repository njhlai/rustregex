use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::{ptr, slice};

use super::State;

#[derive(Debug, PartialEq)]
pub enum Anchor {
    Start,
    End,
    WordBoundary,
}

pub struct AnchorState {
    anchor: Anchor,
    dest: Rc<RefCell<dyn State>>,
}

impl AnchorState {
    pub fn new(anchor: Anchor, dest: Rc<RefCell<dyn State>>) -> Self {
        AnchorState { anchor, dest }
    }
}

impl State for AnchorState {
    fn epsilon(&self, anchors: &[Anchor]) -> &[Rc<RefCell<dyn State>>] {
        if anchors.contains(&self.anchor) {
            slice::from_ref(&self.dest)
        } else {
            &([] as [Rc<RefCell<dyn State>>; 0])
        }
    }

    fn transition(&self, _: char) -> Option<Rc<RefCell<dyn State>>> {
        None
    }

    fn get_dest(&self) -> &[Rc<RefCell<dyn State>>] {
        slice::from_ref(&self.dest)
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
            .downcast_ref::<AnchorState>()
            .map_or(false, |a| ptr::eq(self, a))
    }
}
