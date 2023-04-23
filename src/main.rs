mod modules;

use modules::automata::{closure, concat, optional, or, plus, Automata};
use modules::regexp::RegExp;

fn main() {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfa_concat() {
        let nfa = concat(Automata::from_token('c'), Automata::from_token('d'));
        assert!(nfa.full_match("cd"));
        assert!(!nfa.full_match("c"));
        assert!(!nfa.full_match("d"));
        assert!(!nfa.full_match(""));
        assert!(!nfa.full_match("monty python"));
    }

    #[test]
    fn nfa_union() {
        let nfa = or(Automata::from_token('c'), Automata::from_token('d'));
        assert!(nfa.full_match("c"));
        assert!(nfa.full_match("d"));
        assert!(!nfa.full_match("cd"));
        assert!(!nfa.full_match(""));
        assert!(!nfa.full_match("monty python"));
    }

    #[test]
    fn nfa_closure() {
        let nfa = closure(Automata::from_token('a'));
        assert!(nfa.full_match(""));
        assert!(nfa.full_match("a"));
        assert!(nfa.full_match("aaa"));
        assert!(!nfa.full_match("b"));
    }

    #[test]
    fn nfa_plus() {
        let nfa = plus(Automata::from_token('a'));
        assert!(!nfa.full_match(""));
        assert!(nfa.full_match("a"));
        assert!(nfa.full_match("aaa"));
        assert!(!nfa.full_match("b"));
    }

    #[test]
    fn nfa_optional() {
        let nfa = optional(Automata::from_token('a'));
        assert!(nfa.full_match(""));
        assert!(nfa.full_match("a"));
        assert!(!nfa.full_match("b"));
        assert!(!nfa.full_match("ab"));
        assert!(!nfa.full_match("ba"));
    }

    #[test]
    fn regex_full_match() {
        // (ab?)*|c
        let nfa = or(closure(concat(Automata::from_token('a'), optional(Automata::from_token('b')))), Automata::from_token('c'));
        assert!(nfa.full_match("abaaaaaa"));
        assert!(nfa.full_match("c"));
        assert!(nfa.full_match(""));
        assert!(!nfa.full_match("bb"));
        assert!(!nfa.full_match("aaaaaaac"));
        assert!(!nfa.full_match("cc"));
    }

    #[test]
    fn regex_search() {
        let testexpr = "(a|b)*cd?e+f*";

        let parser = RegExp::new(testexpr);
        assert!(parser.is_ok());

        if let Ok(regex) = parser {
            assert!(regex.full_match("ce"));
            assert!(regex.full_match("ace"));
            assert!(regex.full_match("aaabbbababce"));
            assert!(regex.full_match("cde"));
            assert!(regex.full_match("cef"));
            assert!(regex.full_match("cefffff"));
            assert!(regex.full_match("bacdefffff"));
            assert!(regex.full_match("aababacdefffff"));
            assert!(!regex.full_match("cdde"));
            assert!(!regex.full_match("cdde"));
            assert!(!regex.full_match("aacbdde"));
            assert!(!regex.full_match("e"));
            assert!(!regex.full_match("cdde"));
            assert!(!regex.full_match("cdd"));
            assert!(!regex.full_match(""));
        }
    }

    #[test]
    fn simple_regex() {
        let testexpr = "ba*";

        let parser = RegExp::new(testexpr);
        assert!(parser.is_ok());

        if let Ok(regex) = parser {
            assert_eq!(regex.greedy_search("baababaaa"), Some(String::from("baaa")));
            assert_eq!(regex.greedy_search("b"), Some(String::from("b")));
            assert_eq!(regex.greedy_search("xby"), Some(String::from("b")));
            assert_eq!(regex.greedy_search("xb"), Some(String::from("b")));
            assert_eq!(regex.greedy_search("by"), Some(String::from("b")));
            assert_eq!(regex.greedy_search("ba"), Some(String::from("ba")));
            assert_eq!(regex.greedy_search("bb"), Some(String::from("b")));
            assert_eq!(regex.greedy_search("baaaaa"), Some(String::from("baaaaa")));
            assert_eq!(regex.greedy_search("baaaaam"), Some(String::from("baaaaa")));
            assert_eq!(regex.greedy_search("kabaaaaam"), Some(String::from("baaaaa")));
            assert_eq!(regex.greedy_search("zzzzbaaaam"), Some(String::from("baaaa")));
            assert_eq!(regex.greedy_search("babaaaa"), Some(String::from("baaaa")));
            assert_eq!(regex.greedy_search("ace"), None);
        }
    }

    #[test]
    fn regex_start_anchor() {
        let testexpr = "^abc+";

        let parser = RegExp::new(testexpr);
        assert!(parser.is_ok());

        if let Ok(regex) = parser {
            assert_eq!(regex.greedy_search("abc"), Some(String::from("abc")));
            assert_eq!(regex.greedy_search("abcd"), Some(String::from("abc")));
            assert_eq!(regex.greedy_search("abcdabccc"), Some(String::from("abc")));
            assert_eq!(regex.greedy_search("abcccc"), Some(String::from("abcccc")));
            assert_eq!(regex.greedy_search("eeabc"), None);
        }
    }

    #[test]
    fn regex_end_anchor() {
        let testexpr = "xyz+$";

        let parser = RegExp::new(testexpr);
        assert!(parser.is_ok());

        if let Ok(regex) = parser {
            assert_eq!(regex.greedy_search("xyz"), Some(String::from("xyz")));
            assert_eq!(regex.greedy_search("wxyz"), Some(String::from("xyz")));
            assert_eq!(regex.greedy_search("xxxyzwxyz"), Some(String::from("xyz")));
            assert_eq!(regex.greedy_search("xyzzzz"), Some(String::from("xyzzzz")));
            assert_eq!(regex.greedy_search("xyzaa"), None);
        }
    }

    #[test]
    fn bad_anchor() {
        let testexpr = "$Dhelmise";

        let parser = RegExp::new(testexpr);
        assert!(parser.is_ok());

        if let Ok(regex) = parser {
            assert_eq!(None, regex.greedy_search("Dhelmise"));
        }
    }
}
