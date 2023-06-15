mod anchor;
mod lambda;
mod token;
mod trivial;

pub use self::anchor::AnchorState;
pub use self::lambda::LambdaState;
pub use self::token::TokenState;
pub use self::trivial::TrivialState;

use std::any::Any;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use super::grammar::Anchor;

pub trait State {
    fn epsilon(&self, anchors: &[Anchor]) -> &[Rc<RefCell<dyn State>>];

    fn transition(&self, token: char) -> Option<Rc<RefCell<dyn State>>>;

    fn get_dest(&self) -> &[Rc<RefCell<dyn State>>];

    fn get_state_type(&self) -> String;

    fn as_any(&self) -> &dyn Any;

    fn equals(&self, other: &dyn State) -> bool;
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
        self.equals(other)
    }
}
