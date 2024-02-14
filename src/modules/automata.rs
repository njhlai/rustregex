use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;
use std::str::Chars;

use super::regex::Anchor;
use super::state::{AnchorState, LambdaState, State, TokenState, TrivialState};

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

    pub fn from_lambda<F: Fn(char) -> bool + 'static>(lambda: F) -> Self {
        let end = TrivialState::make_rc();
        let start = Rc::new(RefCell::new(LambdaState::new(lambda, end.clone())));

        Automata { start, end }
    }

    pub fn from_closure(closure: Box<dyn Fn(char) -> bool>) -> Self {
        let end = TrivialState::make_rc();
        let start = Rc::new(RefCell::new(LambdaState::new_with_box(closure, end.clone())));

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
        if let Some(matched) = self.greedy_search(expr) {
            matched.len() == expr.len()
        } else {
            false
        }
    }

    pub fn greedy_search(&self, expr: &str) -> Option<String> {
        self.search(expr).0
    }

    pub fn global_search(&self, expr: &str) -> Vec<String> {
        self.search(expr).1
    }

    fn search(&self, expr: &str) -> (Option<String>, Vec<String>) {
        // First element is the end of the best match, second are the states
        let mut current_states: Vec<(Option<usize>, Vec<StatePtr>)> = vec![];

        for transition in transition_iter(expr) {
            match transition {
                TransitionItem::Char(c) => {
                    for (_, states) in &mut current_states {
                        *states = states
                            .iter()
                            .filter_map(|s| s.borrow().transition(c))
                            .collect();
                    }
                }
                TransitionItem::Epsilon((r, anchors)) => {
                    current_states.push((None, vec![self.start.clone()]));

                    for (match_r, states) in &mut current_states {
                        *states = exhaust_epsilons(states, &anchors);

                        if states.contains(&self.get_end()) {
                            // We have a better match
                            *match_r = Some(r);
                        }
                    }
                }
            }
        }

        let matches = current_states
            .iter()
            .enumerate()
            .filter_map(|(l, (maybe_r, _))| maybe_r.map(|r| (l, r)));

        let mut results = vec![];
        let mut results_rightmost = None;
        for (left, right) in matches {
            // Check if we overlap anything already in our results
            if results_rightmost.map_or(true, |rm| rm <= left && rm < right) {
                results.push(String::from(&expr[left..right]));
                results_rightmost = Some(right);
            }
        }

        // max_by_key will return the last maximal element. Reverse to get the first.
        let greedy_result = results.iter().rev().max_by_key(|s| s.len()).cloned();
        (greedy_result, results)
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
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("NFA")
            .field("start", &self.start)
            .finish_non_exhaustive()
    }
}

fn exhaust_epsilons(states: &[StatePtr], anchors: &[Anchor]) -> Vec<StatePtr> {
    fn traverse_epsilons(
        destinations: &mut Vec<StatePtr>, visited_states: &mut Vec<StatePtr>, state: &StatePtr, anchors: &[Anchor],
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
    IntoIter { it: expr.chars(), current: None, index: 0 }
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

        assert_eq!(nfa.greedy_search(""), None);
        assert_eq!(nfa.greedy_search("c"), None);
        assert_eq!(nfa.greedy_search("d"), None);
        assert_eq!(nfa.greedy_search("cd"), Some(String::from("cd")));
        assert_eq!(nfa.greedy_search("dc"), None);
        assert_eq!(nfa.greedy_search("abcde"), Some(String::from("cd")));
        assert_eq!(nfa.greedy_search("monty python"), None);

        assert_eq!(nfa.global_search(""), Vec::<String>::new());
        assert_eq!(nfa.global_search("c"), Vec::<String>::new());
        assert_eq!(nfa.global_search("d"), Vec::<String>::new());
        assert_eq!(nfa.global_search("cd"), vec!["cd"]);
        assert_eq!(nfa.global_search("dc"), Vec::<String>::new());
        assert_eq!(nfa.global_search("abcde"), vec!["cd"]);
        assert_eq!(nfa.global_search("monty python"), Vec::<String>::new());
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

        assert_eq!(nfa.greedy_search(""), None);
        assert_eq!(nfa.greedy_search("c"), Some(String::from("c")));
        assert_eq!(nfa.greedy_search("d"), Some(String::from("d")));
        assert_eq!(nfa.greedy_search("cd"), Some(String::from("c")));
        assert_eq!(nfa.greedy_search("dc"), Some(String::from("d")));
        assert_eq!(nfa.greedy_search("abcde"), Some(String::from("c")));
        assert_eq!(nfa.greedy_search("monty python"), None);

        assert_eq!(nfa.global_search(""), Vec::<String>::new());
        assert_eq!(nfa.global_search("c"), vec!["c"]);
        assert_eq!(nfa.global_search("d"), vec!["d"]);
        assert_eq!(nfa.global_search("cd"), vec!["c", "d"]);
        assert_eq!(nfa.global_search("dc"), vec!["d", "c"]);
        assert_eq!(nfa.global_search("abcde"), vec!["c", "d"]);
        assert_eq!(nfa.global_search("monty python"), Vec::<String>::new());
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

        assert_eq!(nfa.greedy_search(""), Some(String::from("")));
        assert_eq!(nfa.greedy_search("a"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("aaa"), Some(String::from("aaa")));
        assert_eq!(nfa.greedy_search("b"), Some(String::from("")));
        assert_eq!(nfa.greedy_search("ab"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("ba"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("basic"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("this is a string"), Some(String::from("a")));

        assert_eq!(nfa.global_search(""), vec![""]);
        assert_eq!(nfa.global_search("a"), vec!["a"]);
        assert_eq!(nfa.global_search("aaa"), vec!["aaa"]);
        assert_eq!(nfa.global_search("b"), vec!["", ""]);
        assert_eq!(nfa.global_search("ab"), vec!["a", ""]);
        assert_eq!(nfa.global_search("ba"), vec!["", "a"]);
        assert_eq!(nfa.global_search("basic"), vec!["", "a", "", "", ""]);
        assert_eq!(
            nfa.global_search("this is a string"),
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

        assert_eq!(nfa.greedy_search(""), None);
        assert_eq!(nfa.greedy_search("a"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("aaa"), Some(String::from("aaa")));
        assert_eq!(nfa.greedy_search("b"), None);
        assert_eq!(nfa.greedy_search("ab"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("ba"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("basic"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("this is a string"), Some(String::from("a")));

        assert_eq!(nfa.global_search(""), Vec::<String>::new());
        assert_eq!(nfa.global_search("a"), vec!["a"]);
        assert_eq!(nfa.global_search("aaa"), vec!["aaa"]);
        assert_eq!(nfa.global_search("b"), Vec::<String>::new());
        assert_eq!(nfa.global_search("ab"), vec!["a"]);
        assert_eq!(nfa.global_search("ba"), vec!["a"]);
        assert_eq!(nfa.global_search("basic"), vec!["a"]);
        assert_eq!(nfa.global_search("this is a string"), vec!["a"]);
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

        assert_eq!(nfa.greedy_search(""), Some(String::from("")));
        assert_eq!(nfa.greedy_search("a"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("aaa"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("b"), Some(String::from("")));
        assert_eq!(nfa.greedy_search("ab"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("ba"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("basic"), Some(String::from("a")));
        assert_eq!(nfa.greedy_search("this is a string"), Some(String::from("a")));

        assert_eq!(nfa.global_search(""), vec![""]);
        assert_eq!(nfa.global_search("a"), vec!["a"]);
        assert_eq!(nfa.global_search("aaa"), vec!["a", "a", "a"]);
        assert_eq!(nfa.global_search("b"), vec!["", ""]);
        assert_eq!(nfa.global_search("ab"), vec!["a", ""]);
        assert_eq!(nfa.global_search("ba"), vec!["", "a"]);
        assert_eq!(nfa.global_search("basic"), vec!["", "a", "", "", ""]);
        assert_eq!(
            nfa.global_search("this is a string"),
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
        assert!(nfa.full_match("abab"));
        assert!(!nfa.full_match("abad"));
        assert!(nfa.full_match("c"));
        assert!(nfa.full_match(""));
        assert!(!nfa.full_match("bb"));
        assert!(!nfa.full_match("aaaaaaac"));
        assert!(!nfa.full_match("cc"));

        assert_eq!(nfa.greedy_search("abaaaaaa"), Some(String::from("abaaaaaa")));
        assert_eq!(nfa.greedy_search("abab"), Some(String::from("abab")));
        assert_eq!(nfa.greedy_search("abad"), Some(String::from("aba")));
        assert_eq!(nfa.greedy_search("c"), Some(String::from("c")));
        assert_eq!(nfa.greedy_search(""), Some(String::from("")));
        assert_eq!(nfa.greedy_search("bb"), Some(String::from("")));
        assert_eq!(nfa.greedy_search("aaaaaaac"), Some(String::from("aaaaaaa")));
        assert_eq!(nfa.greedy_search("cc"), Some(String::from("c")));

        assert_eq!(nfa.global_search("abaaaaaa"), vec!["abaaaaaa"]);
        assert_eq!(nfa.global_search("abab"), vec!["abab"]);
        assert_eq!(nfa.global_search("abad"), vec!["aba", ""]);
        assert_eq!(nfa.global_search("c"), vec!["c"]);
        assert_eq!(nfa.global_search(""), vec![""]);
        assert_eq!(nfa.global_search("bb"), vec!["", "", ""]);
        assert_eq!(nfa.global_search("aaaaaaac"), vec!["aaaaaaa", "c"]);
        assert_eq!(nfa.global_search("cc"), vec!["c", "c"]);
    }
}
