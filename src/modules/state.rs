use std::rc::Rc;

pub trait State {
    fn epsilon(&self) -> &[Rc<dyn State>];

    fn transition(&self, token: char) -> Option<Rc<dyn State>>;
}

#[derive(Default)]
pub struct TrivialState {
    dest: Vec<Rc<dyn State>>,
}

impl State for TrivialState {
    fn epsilon(&self) -> &[Rc<dyn State>] {
        &self.dest[..]
    }

    fn transition(&self, _: char) -> Option<Rc<dyn State>> {
        None
    }
}

pub struct TokenState {
    dest: Rc<dyn State>,
    token: char,
}

impl TokenState {
    pub fn new(token: char) -> Self {
        Self::init(token, Rc::new(TrivialState::default()))
    }

    fn init(token: char, dest: Rc<dyn State>) -> Self {
        TokenState { dest, token }
    }
}

impl State for TokenState {
    fn epsilon(&self) -> &[Rc<dyn State>] {
        &([] as [Rc<dyn State>; 0])
    }

    fn transition(&self, token: char) -> Option<Rc<dyn State>> {
        if self.token == token { Some(self.dest.clone()) } else { None }
    }
}

pub struct LambdaState {
    dest: Rc<dyn State>,
    lambda: fn(char) -> bool,
}

impl LambdaState {
    pub fn new(lambda: fn(char) -> bool) -> Self {
        Self::init(lambda, Rc::new(TrivialState::default()))
    }

    fn init(lambda: fn(char) -> bool, dest: Rc<dyn State>) -> Self {
        LambdaState { dest, lambda }
    }
}

impl State for LambdaState  {
    fn epsilon(&self) -> &[Rc<dyn State>] {
        &([] as [Rc<dyn State>; 0])
    }

    fn transition(&self, token: char) -> Option<Rc<dyn State>> {
        if (self.lambda)(token) { Some(self.dest.clone()) } else { None }
    }
}