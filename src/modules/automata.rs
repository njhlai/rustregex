use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use super::state::{LambdaState, State, TokenState, TrivialState};

type StatePtr = Rc<RefCell<dyn State>>;

pub struct Automata {
    start: StatePtr,
    end: Rc<RefCell<TrivialState>>,
}

impl Automata {
    pub fn from_token(token: char) -> Self {
        let end = TrivialState::make_rc();
        let start = Rc::new(RefCell::new(TokenState::new(token, end.clone())));

        Automata { start, end }
    }

    pub fn from_lambda(lambda: fn(char) -> bool) -> Self {
        let end = TrivialState::make_rc();
        let start = Rc::new(RefCell::new(LambdaState::new(lambda, end.clone())));

        Automata { start, end }
    }

    fn push_to_end(&self, state: StatePtr) {
        self.end.borrow_mut().push(state);
    }

    pub fn full_match(&self, expr: &str) -> bool {
        let mut current_states = exhaust_epsilons(vec![self.start.clone()]);

        for c in expr.chars() {
            current_states = exhaust_epsilons(current_states.iter().filter_map(|s| s.borrow().transition(c)).collect());
        }

        current_states.contains(&(self.end.clone() as StatePtr))
    }

    pub fn greedy_search(&self, expr: &str) -> Option<String> {
        let mut current_states = vec![];
        let (mut start, mut end) = (0, 0);

        for (r, c) in expr.chars().enumerate() {
            current_states.push(exhaust_epsilons(vec![self.start.clone()]));

            let mut found = false;
            for (l, states) in current_states.iter_mut().enumerate() {
                *states = exhaust_epsilons(states.iter().filter_map(|s| s.borrow().transition(c)).collect());

                if !found && states.contains(&(self.end.clone() as StatePtr)) && end - start < r - l + 1 {
                    start = l;
                    end = r + 1;
                    found = true;
                }
            }
        }

        if end - start > 0 {
            Some(String::from(&expr[start..end]))
        } else { None }
    }
}

impl Debug for Automata {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("NFA").field("start", &self.start).finish()
    }
}

fn exhaust_epsilons(states: Vec<StatePtr>) -> Vec<StatePtr> {
    fn traverse_epsilons(destinations: &mut Vec<StatePtr>, visited_states: &mut Vec<StatePtr>, state: &StatePtr) {
        let state_locked = state.borrow();
        let reachables = state_locked.epsilon();

        if reachables.is_empty() { destinations.push(state.clone()); }

        for candidate in reachables {
            if visited_states.contains(candidate) { continue; }

            visited_states.push(candidate.clone());
            traverse_epsilons(destinations, visited_states, candidate);
        }
    }

    let mut destinations: Vec<StatePtr> = Vec::new();
    let mut visited_states: Vec<StatePtr> = Vec::new();

    states
        .iter()
        .for_each(|s| traverse_epsilons(&mut destinations, &mut visited_states, s));

    destinations
}

pub fn concat(a: Automata, b: Automata) -> Automata {
    a.push_to_end(b.start.clone());
    Automata { start: a.start, end: b.end }
}

pub fn or(a: Automata, b: Automata) -> Automata {
    let start = TrivialState::make_rc();
    let end = TrivialState::make_rc();

    start.borrow_mut().push(a.start.clone());
    start.borrow_mut().push(b.start.clone());

    a.push_to_end(end.clone());
    b.push_to_end(end.clone());

    Automata { start, end }
}

pub fn closure(a: Automata) -> Automata {
    let start = TrivialState::make_rc();
    let end = TrivialState::make_rc();

    start.borrow_mut().push(a.start.clone());
    start.borrow_mut().push(end.clone());

    a.push_to_end(a.start.clone());
    a.push_to_end(end.clone());

    Automata { start, end }
}

pub fn optional(a: Automata) -> Automata {
    let start = TrivialState::make_rc();
    let end = TrivialState::make_rc();

    start.borrow_mut().push(a.start.clone());
    start.borrow_mut().push(end.clone());

    a.push_to_end(end.clone());

    Automata { start, end }
}

pub fn plus(a: Automata) -> Automata {
    let start = TrivialState::make_rc();
    let end = TrivialState::make_rc();

    start.borrow_mut().push(a.start.clone());

    a.push_to_end(a.start.clone());
    a.push_to_end(end.clone());

    Automata { start, end }
}