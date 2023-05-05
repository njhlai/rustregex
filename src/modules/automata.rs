use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::rc::Rc;
use std::str::Chars;

use super::parser::Error;
use super::state::{Anchor, AnchorState, LambdaState, State, TokenState, TrivialState};

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

    pub fn from_anchor(anchor: Anchor) -> Self {
        let end = TrivialState::make_rc();
        let start = Rc::new(RefCell::new(AnchorState::new(anchor, end.clone())));

        Automata { start, end }
    }

    pub fn concat(mut self, other: Automata) -> Self {
        self.push_to_end(other.start.clone());
        self.end = other.end.clone();
        self
    }

    pub fn or(mut self, mut other: Automata) -> Self {
        let start = TrivialState::make_rc();
        let end = TrivialState::make_rc();

        start.borrow_mut().push(self.start.clone());
        start.borrow_mut().push(other.start.clone());

        self.push_to_end(end.clone());
        other.push_to_end(end.clone());

        self.start = start;
        self.end = end;
        self
    }

    pub fn closure(mut self) -> Self {
        let start = TrivialState::make_rc();
        let end = TrivialState::make_rc();

        start.borrow_mut().push(self.start.clone());
        start.borrow_mut().push(end.clone());

        self.push_to_end(self.start.clone());
        self.push_to_end(end.clone());

        self.start = start;
        self.end = end;
        self
    }

    pub fn optional(mut self) -> Self {
        let start = TrivialState::make_rc();
        let end = TrivialState::make_rc();

        start.borrow_mut().push(self.start.clone());
        start.borrow_mut().push(end.clone());

        self.push_to_end(end.clone());

        self.start = start;
        self.end = end;
        self
    }

    pub fn plus(mut self) -> Self {
        let start = TrivialState::make_rc();
        let end = TrivialState::make_rc();

        start.borrow_mut().push(self.start.clone());

        self.push_to_end(self.start.clone());
        self.push_to_end(end.clone());

        self.start = start;
        self.end = end;
        self
    }

    pub fn full_match(&self, expr: &str) -> bool {
        match self.greedy_search(expr) {
            Ok(result) => {
                if let Some(matched) = result {
                    matched.len() == expr.len()
                } else {
                    false
                }
            }
            Err(error) => {
                println!("Warning: {error:#?}");
                false
            }
        }
    }

    pub fn greedy_search(&self, expr: &str) -> Result<Option<String>, Error> {
        let mut search_results = self.search(expr, true);

        if search_results.len() > 1 {
            Err(Error::from("too many results returned for greedy search"))
        } else {
            Ok(search_results.pop())
        }
    }

    pub fn search(&self, expr: &str, greedy: bool) -> Vec<String> {
        let mut results = vec![];
        let mut current_states: Vec<Vec<StatePtr>> = vec![];
        let (mut start, mut end, mut threshold) = (0, None, None);

        for transition in transition_iter(expr) {
            match transition {
                TransitionItem::Char(c) => {
                    for states in &mut current_states {
                        *states = states
                            .iter()
                            .filter_map(|s| s.borrow().transition(c))
                            .collect();
                    }
                }
                TransitionItem::Epsilon((r, anchors)) => {
                    current_states.push(vec![self.start.clone()]);
                    if !greedy {
                        threshold = None;
                    }

                    for (l, states) in current_states.iter_mut().enumerate() {
                        *states = exhaust_epsilons(states, &anchors);

                        if threshold.map_or(true, |len| len < r - l) && states.contains(&self.get_end()) {
                            if greedy {
                                start = l;
                            } else if start < l {
                                if let Some(end) = end {
                                    if end >= start {
                                        results.push(String::from(&expr[start..end]));
                                    }
                                }

                                start = l;
                            }

                            threshold = Some(r - l);
                            end = Some(r);
                        }
                    }
                }
            }
        }

        if let Some(end) = end {
            if end >= start {
                results.push(String::from(&expr[start..end]));
            }
        }

        results
    }

    fn get_end(&self) -> StatePtr {
        // the `clone' function only clones the reference-counted pointer, so this should be ok...
        self.end.clone() as StatePtr
    }

    // It's not necessary to consume a mutable reference, but this function does modify
    // the underlying states. Requiring a mutable reference makes this clearer.
    fn push_to_end(&mut self, state: StatePtr) {
        self.end.borrow_mut().push(state);
    }
}

impl Debug for Automata {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("NFA").field("start", &self.start).finish()
    }
}

fn exhaust_epsilons(states: &[StatePtr], anchors: &[Anchor]) -> Vec<StatePtr> {
    fn traverse_epsilons(
        destinations: &mut Vec<StatePtr>,
        visited_states: &mut Vec<StatePtr>,
        state: &StatePtr,
        anchors: &[Anchor],
    ) {
        let state_locked = state.borrow();
        let reachables = state_locked.epsilon(anchors);

        if reachables.is_empty() {
            destinations.push(state.clone());
        }

        for candidate in reachables {
            if visited_states.contains(candidate) {
                continue;
            }

            visited_states.push(candidate.clone());
            traverse_epsilons(destinations, visited_states, candidate, anchors);
        }
    }

    let mut destinations: Vec<StatePtr> = Vec::new();
    let mut visited_states: Vec<StatePtr> = Vec::new();

    states
        .iter()
        .for_each(|s| traverse_epsilons(&mut destinations, &mut visited_states, s, anchors));

    destinations
}

struct TransitionIter<'a> {
    it: Chars<'a>,
    current: Option<char>,
    index: usize,
}

impl<'a> Iterator for TransitionIter<'a> {
    type Item = TransitionItem;

    fn next(&mut self) -> Option<Self::Item> {
        let return_char = self.index % 2 == 1;
        self.index += 1;

        if return_char {
            self.current.map(TransitionItem::Char)
        } else {
            let next = self.it.next();
            let eps = TransitionItem::get_anchors(self.index / 2, self.current, next);
            self.current = next;

            Some(eps)
        }
    }
}

fn transition_iter(expr: &str) -> TransitionIter {
    TransitionIter { it: expr.chars(), current: None, index: 0 }
}

enum TransitionItem {
    Char(char),
    Epsilon((usize, Vec<Anchor>)),
}

impl TransitionItem {
    fn get_anchors(index: usize, current: Option<char>, next: Option<char>) -> Self {
        let mut anchors = vec![];

        if let (Some(p), Some(n)) = (current, next) {
            if p.is_alphanumeric() != n.is_alphanumeric() {
                anchors.push(Anchor::WordBoundary);
            }
        } else {
            if current.is_none() {
                anchors.push(Anchor::Start);
            }
            if next.is_none() {
                anchors.push(Anchor::End);
            }
            anchors.push(Anchor::WordBoundary);
        }

        TransitionItem::Epsilon((index, anchors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfa_concat() {
        let nfa = Automata::from_token('c').concat(Automata::from_token('d'));

        assert!(!nfa.full_match(""));
        assert!(!nfa.full_match("c"));
        assert!(!nfa.full_match("d"));
        assert!(nfa.full_match("cd"));
        assert!(!nfa.full_match("dc"));
        assert!(!nfa.full_match("abcde"));
        assert!(!nfa.full_match("monty python"));

        assert_eq!(nfa.greedy_search(""), Ok(None));
        assert_eq!(nfa.greedy_search("c"), Ok(None));
        assert_eq!(nfa.greedy_search("d"), Ok(None));
        assert_eq!(nfa.greedy_search("cd"), Ok(Some(String::from("cd"))));
        assert_eq!(nfa.greedy_search("dc"), Ok(None));
        assert_eq!(nfa.greedy_search("abcde"), Ok(Some(String::from("cd"))));
        assert_eq!(nfa.greedy_search("monty python"), Ok(None));

        assert_eq!(nfa.search("", false), Vec::<String>::new());
        assert_eq!(nfa.search("c", false), Vec::<String>::new());
        assert_eq!(nfa.search("d", false), Vec::<String>::new());
        assert_eq!(nfa.search("cd", false), vec!["cd"]);
        assert_eq!(nfa.search("dc", false), Vec::<String>::new());
        assert_eq!(nfa.search("abcde", false), vec!["cd"]);
        assert_eq!(nfa.search("monty python", false), Vec::<String>::new());
    }

    #[test]
    fn nfa_union() {
        let nfa = Automata::from_token('c').or(Automata::from_token('d'));

        assert!(!nfa.full_match(""));
        assert!(nfa.full_match("c"));
        assert!(nfa.full_match("d"));
        assert!(!nfa.full_match("cd"));
        assert!(!nfa.full_match("dc"));
        assert!(!nfa.full_match("abcde"));
        assert!(!nfa.full_match("monty python"));

        assert_eq!(nfa.greedy_search(""), Ok(None));
        assert_eq!(nfa.greedy_search("c"), Ok(Some(String::from("c"))));
        assert_eq!(nfa.greedy_search("d"), Ok(Some(String::from("d"))));
        assert_eq!(nfa.greedy_search("cd"), Ok(Some(String::from("c"))));
        assert_eq!(nfa.greedy_search("dc"), Ok(Some(String::from("d"))));
        assert_eq!(nfa.greedy_search("abcde"), Ok(Some(String::from("c"))));
        assert_eq!(nfa.greedy_search("monty python"), Ok(None));

        assert_eq!(nfa.search("", false), Vec::<String>::new());
        assert_eq!(nfa.search("c", false), vec!["c"]);
        assert_eq!(nfa.search("d", false), vec!["d"]);
        assert_eq!(nfa.search("cd", false), vec!["c", "d"]);
        assert_eq!(nfa.search("dc", false), vec!["d", "c"]);
        assert_eq!(nfa.search("abcde", false), vec!["c", "d"]);
        assert_eq!(nfa.search("monty python", false), Vec::<String>::new());
    }

    #[test]
    fn nfa_closure() {
        let nfa = Automata::from_token('a').closure();

        assert!(nfa.full_match(""));
        assert!(nfa.full_match("a"));
        assert!(nfa.full_match("aaa"));
        assert!(!nfa.full_match("b"));
        assert!(!nfa.full_match("ab"));
        assert!(!nfa.full_match("ba"));
        assert!(!nfa.full_match("basic"));
        assert!(!nfa.full_match("this is a string"));

        assert_eq!(nfa.greedy_search(""), Ok(Some(String::from(""))));
        assert_eq!(nfa.greedy_search("a"), Ok(Some(String::from("a"))));
        assert_eq!(nfa.greedy_search("aaa"), Ok(Some(String::from("aaa"))));
        assert_eq!(nfa.greedy_search("b"), Ok(Some(String::from(""))));
        assert_eq!(nfa.greedy_search("ab"), Ok(Some(String::from("a"))));
        assert_eq!(nfa.greedy_search("ba"), Ok(Some(String::from("a"))));
        assert_eq!(nfa.greedy_search("basic"), Ok(Some(String::from("a"))));
        assert_eq!(nfa.greedy_search("this is a string"), Ok(Some(String::from("a"))));

        assert_eq!(nfa.search("", false), vec![""]);
        assert_eq!(nfa.search("a", false), vec!["a"]);
        assert_eq!(nfa.search("aaa", false), vec!["aaa"]);
        assert_eq!(nfa.search("b", false), vec!["", ""]);
        assert_eq!(nfa.search("ab", false), vec!["a", ""]);
        assert_eq!(nfa.search("ba", false), vec!["", "a"]);
        assert_eq!(nfa.search("basic", false), vec!["", "a", "", "", ""]);
        assert_eq!(
            nfa.search("this is a string", false),
            vec!["", "", "", "", "", "", "", "", "a", "", "", "", "", "", "", ""]
        );
    }

    #[test]
    fn nfa_plus() {
        let nfa = Automata::from_token('a').plus();

        assert!(!nfa.full_match(""));
        assert!(nfa.full_match("a"));
        assert!(nfa.full_match("aaa"));
        assert!(!nfa.full_match("b"));
        assert!(!nfa.full_match("ab"));
        assert!(!nfa.full_match("ba"));
        assert!(!nfa.full_match("basic"));
        assert!(!nfa.full_match("this is a string"));

        assert_eq!(nfa.greedy_search(""), Ok(None));
        assert_eq!(nfa.greedy_search("a"), Ok(Some(String::from("a"))));
        assert_eq!(nfa.greedy_search("aaa"), Ok(Some(String::from("aaa"))));
        assert_eq!(nfa.greedy_search("b"), Ok(None));
        assert_eq!(nfa.greedy_search("ab"), Ok(Some(String::from("a"))));
        assert_eq!(nfa.greedy_search("ba"), Ok(Some(String::from("a"))));
        assert_eq!(nfa.greedy_search("basic"), Ok(Some(String::from("a"))));
        assert_eq!(nfa.greedy_search("this is a string"), Ok(Some(String::from("a"))));

        assert_eq!(nfa.search("", false), Vec::<String>::new());
        assert_eq!(nfa.search("a", false), vec!["a"]);
        assert_eq!(nfa.search("aaa", false), vec!["aaa"]);
        assert_eq!(nfa.search("b", false), Vec::<String>::new());
        assert_eq!(nfa.search("ab", false), vec!["a"]);
        assert_eq!(nfa.search("ba", false), vec!["a"]);
        assert_eq!(nfa.search("basic", false), vec!["a"]);
        assert_eq!(nfa.search("this is a string", false), vec!["a"]);
    }

    #[test]
    fn nfa_optional() {
        let nfa = Automata::from_token('a').optional();

        assert!(nfa.full_match(""));
        assert!(nfa.full_match("a"));
        assert!(!nfa.full_match("aaa"));
        assert!(!nfa.full_match("b"));
        assert!(!nfa.full_match("ab"));
        assert!(!nfa.full_match("ba"));
        assert!(!nfa.full_match("basic"));
        assert!(!nfa.full_match("this is a string"));

        assert_eq!(nfa.greedy_search(""), Ok(Some(String::from(""))));
        assert_eq!(nfa.greedy_search("a"), Ok(Some(String::from("a"))));
        assert_eq!(nfa.greedy_search("aaa"), Ok(Some(String::from("a"))));
        assert_eq!(nfa.greedy_search("b"), Ok(Some(String::from(""))));
        assert_eq!(nfa.greedy_search("ab"), Ok(Some(String::from("a"))));
        assert_eq!(nfa.greedy_search("ba"), Ok(Some(String::from("a"))));
        assert_eq!(nfa.greedy_search("basic"), Ok(Some(String::from("a"))));
        assert_eq!(nfa.greedy_search("this is a string"), Ok(Some(String::from("a"))));

        assert_eq!(nfa.search("", false), vec![""]);
        assert_eq!(nfa.search("a", false), vec!["a"]);
        assert_eq!(nfa.search("aaa", false), vec!["a", "a", "a"]);
        assert_eq!(nfa.search("b", false), vec!["", ""]);
        assert_eq!(nfa.search("ab", false), vec!["a", ""]);
        assert_eq!(nfa.search("ba", false), vec!["", "a"]);
        assert_eq!(nfa.search("basic", false), vec!["", "a", "", "", ""]);
        assert_eq!(
            nfa.search("this is a string", false),
            vec!["", "", "", "", "", "", "", "", "a", "", "", "", "", "", "", ""]
        );
    }

    #[test]
    fn nfa_realistic() {
        // (ab?)*|c
        let nfa = Automata::from_token('a')
            .concat(Automata::from_token('b').optional())
            .closure()
            .or(Automata::from_token('c'));

        assert!(nfa.full_match("abaaaaaa"));
        assert!(nfa.full_match("c"));
        assert!(nfa.full_match(""));
        assert!(!nfa.full_match("bb"));
        assert!(!nfa.full_match("aaaaaaac"));
        assert!(!nfa.full_match("cc"));

        assert_eq!(nfa.greedy_search("abaaaaaa"), Ok(Some(String::from("abaaaaaa"))));
        assert_eq!(nfa.greedy_search("c"), Ok(Some(String::from("c"))));
        assert_eq!(nfa.greedy_search(""), Ok(Some(String::from(""))));
        assert_eq!(nfa.greedy_search("bb"), Ok(Some(String::from(""))));
        assert_eq!(nfa.greedy_search("aaaaaaac"), Ok(Some(String::from("aaaaaaa"))));
        assert_eq!(nfa.greedy_search("cc"), Ok(Some(String::from("c"))));

        assert_eq!(nfa.search("abaaaaaa", false), vec!["abaaaaaa"]);
        assert_eq!(nfa.search("c", false), vec!["c"]);
        assert_eq!(nfa.search("", false), vec![""]);
        assert_eq!(nfa.search("bb", false), vec!["", "", ""]);
        assert_eq!(nfa.search("aaaaaaac", false), vec!["aaaaaaa", "c"]);
        assert_eq!(nfa.search("cc", false), vec!["c", "c"]);
    }
}
