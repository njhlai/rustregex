use std::any::Any;
use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;
use std::slice;

use super::State;

#[derive(PartialEq, PartialOrd)]
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
    fn epsilon(&self, anchor: &Option<Anchor>) -> &[Rc<RefCell<dyn State>>] {
        if let Some(anchor_type) = anchor {
            if *anchor_type <= self.anchor { return slice::from_ref(&self.dest); }
        } 

        &([] as [Rc<RefCell<dyn State>>; 0])
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