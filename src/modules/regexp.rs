use super::automata::Automata;
use super::parser::{parse, Error};

pub struct RegExp {
    automata: Automata,
}

impl RegExp {
    pub fn new(expr: &str) -> Result<Self, Error> {
        Ok(RegExp {
            automata: parse(expr)?,
        })
    }

    pub fn full_match(&self, expr: &str) -> bool {
        self.automata.full_match(expr)
    }

    pub fn greedy_search(&self, expr: &str) -> Option<String> {
        self.automata.greedy_search(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_search() {
        let testexpr = "(a|b)*cd?e+f*";

        let regexp = RegExp::new(testexpr);
        assert!(regexp.is_ok());

        if let Ok(regex) = regexp {
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

        let regexp = RegExp::new(testexpr);
        assert!(regexp.is_ok());

        if let Ok(regex) = regexp {
            assert_eq!(regex.greedy_search("baababaaa"), Some(String::from("baaa")));
            assert_eq!(regex.greedy_search("b"), Some(String::from("b")));
            assert_eq!(regex.greedy_search("xby"), Some(String::from("b")));
            assert_eq!(regex.greedy_search("xb"), Some(String::from("b")));
            assert_eq!(regex.greedy_search("by"), Some(String::from("b")));
            assert_eq!(regex.greedy_search("ba"), Some(String::from("ba")));
            assert_eq!(regex.greedy_search("bb"), Some(String::from("b")));
            assert_eq!(regex.greedy_search("baaaaa"), Some(String::from("baaaaa")));
            assert_eq!(regex.greedy_search("baaaaam"), Some(String::from("baaaaa")));
            assert_eq!(
                regex.greedy_search("kabaaaaam"),
                Some(String::from("baaaaa"))
            );
            assert_eq!(
                regex.greedy_search("zzzzbaaaam"),
                Some(String::from("baaaa"))
            );
            assert_eq!(regex.greedy_search("babaaaa"), Some(String::from("baaaa")));
            assert_eq!(regex.greedy_search("ace"), None);
        }
    }

    #[test]
    fn regex_start_anchor() {
        let testexpr = "^abc+";

        let regexp = RegExp::new(testexpr);
        assert!(regexp.is_ok());

        if let Ok(regex) = regexp {
            assert!(regex.full_match("abc"));
            assert!(regex.full_match("abcccc"));
            assert!(!regex.full_match("abcd"));
            assert!(!regex.full_match("zabc"));

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

        let regexp = RegExp::new(testexpr);
        assert!(regexp.is_ok());

        if let Ok(regex) = regexp {
            assert!(regex.full_match("xyz"));
            assert!(regex.full_match("xyzzzzz"));
            assert!(!regex.full_match("wxyz"));
            assert!(!regex.full_match("xyza"));

            assert_eq!(regex.greedy_search("xyz"), Some(String::from("xyz")));
            assert_eq!(regex.greedy_search("wxyz"), Some(String::from("xyz")));
            assert_eq!(regex.greedy_search("xxxyzwxyz"), Some(String::from("xyz")));
            assert_eq!(regex.greedy_search("xyzzzz"), Some(String::from("xyzzzz")));
            assert_eq!(regex.greedy_search("xyzaa"), None);
        }
    }

    #[test]
    fn regex_anchors_with_empty_matcher() {
        let testexpr = "^a*$";

        let regexp = RegExp::new(testexpr);
        assert!(regexp.is_ok());

        if let Ok(regex) = regexp {
            assert!(regex.full_match(""));
            assert!(regex.full_match("a"));
            assert!(!regex.full_match("b"));
            assert!(!regex.full_match("ab"));


            assert_eq!(regex.greedy_search(""), Some(String::from("")));
            assert_eq!(regex.greedy_search("a"), Some(String::from("a")));
            assert_eq!(regex.greedy_search("b"), None);
            assert_eq!(regex.greedy_search("ab"), None);
        }
    }

    #[test]
    fn bad_anchor() {
        let testexpr = "$Dhelmise";

        let regexp = RegExp::new(testexpr);
        assert!(regexp.is_ok());

        if let Ok(regex) = regexp {
            assert_eq!(None, regex.greedy_search("Dhelmise"));
        }
    }
}
