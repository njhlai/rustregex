use super::automata::Automata;
use super::error::Error;
use super::parser;

pub struct RegExp {
    automata: Automata,
}

impl RegExp {
    pub fn new(expr: &str) -> Result<Self, Error> {
        Ok(RegExp { automata: parser::parse(expr)? })
    }

    pub fn full_match(&self, expr: &str) -> bool {
        self.automata.full_match(expr)
    }

    pub fn greedy_search(&self, expr: &str) -> Result<Option<String>, Error> {
        self.automata.greedy_search(expr)
    }

    pub fn search(&self, expr: &str) -> Vec<String> {
        self.automata.search(expr, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_realistic() {
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

            assert_eq!(regex.greedy_search("ce"), Ok(Some(String::from("ce"))));
            assert_eq!(regex.greedy_search("ace"), Ok(Some(String::from("ace"))));
            assert_eq!(regex.greedy_search("aaabbbababce"), Ok(Some(String::from("aaabbbababce"))));
            assert_eq!(regex.greedy_search("cde"), Ok(Some(String::from("cde"))));
            assert_eq!(regex.greedy_search("cef"), Ok(Some(String::from("cef"))));
            assert_eq!(regex.greedy_search("cefffff"), Ok(Some(String::from("cefffff"))));
            assert_eq!(regex.greedy_search("bacdefffff"), Ok(Some(String::from("bacdefffff"))));
            assert_eq!(regex.greedy_search("aababacdefffff"), Ok(Some(String::from("aababacdefffff"))));
            assert_eq!(regex.greedy_search("cdde"), Ok(None));
            assert_eq!(regex.greedy_search("cdde"), Ok(None));
            assert_eq!(regex.greedy_search("aacbdde"), Ok(None));
            assert_eq!(regex.greedy_search("e"), Ok(None));
            assert_eq!(regex.greedy_search("cdde"), Ok(None));
            assert_eq!(regex.greedy_search("cdd"), Ok(None));
            assert_eq!(regex.greedy_search(""), Ok(None));

            assert_eq!(regex.search("ce"), vec!["ce"]);
            assert_eq!(regex.search("ace"), vec!["ace"]);
            assert_eq!(regex.search("aaabbbababce"), vec!["aaabbbababce"]);
            assert_eq!(regex.search("cde"), vec!["cde"]);
            assert_eq!(regex.search("cef"), vec!["cef"]);
            assert_eq!(regex.search("cefffff"), vec!["cefffff"]);
            assert_eq!(regex.search("bacdefffff"), vec!["bacdefffff"]);
            assert_eq!(regex.search("aababacdefffff"), vec!["aababacdefffff"]);
            assert_eq!(regex.search("cdde"), Vec::<String>::new());
            assert_eq!(regex.search("cdde"), Vec::<String>::new());
            assert_eq!(regex.search("aacbdde"), Vec::<String>::new());
            assert_eq!(regex.search("e"), Vec::<String>::new());
            assert_eq!(regex.search("cdde"), Vec::<String>::new());
            assert_eq!(regex.search("cdd"), Vec::<String>::new());
            assert_eq!(regex.search(""), Vec::<String>::new());
        }
    }

    #[test]
    fn regex_simple() {
        let testexpr = "ba*";

        let regexp = RegExp::new(testexpr);
        assert!(regexp.is_ok());

        if let Ok(regex) = regexp {
            assert!(!regex.full_match("baababaaa"));
            assert!(regex.full_match("b"));
            assert!(!regex.full_match("xby"));
            assert!(!regex.full_match("xb"));
            assert!(!regex.full_match("by"));
            assert!(regex.full_match("ba"));
            assert!(!regex.full_match("bb"));
            assert!(regex.full_match("baaaaa"));
            assert!(!regex.full_match("baaaaam"));
            assert!(!regex.full_match("kabaaaaam"));
            assert!(!regex.full_match("zzzzbaaaam"));
            assert!(!regex.full_match("zzbabaaaabbbam"));
            assert!(!regex.full_match("ace"));

            assert_eq!(regex.greedy_search("baababaaa"), Ok(Some(String::from("baaa"))));
            assert_eq!(regex.greedy_search("b"), Ok(Some(String::from("b"))));
            assert_eq!(regex.greedy_search("xby"), Ok(Some(String::from("b"))));
            assert_eq!(regex.greedy_search("xb"), Ok(Some(String::from("b"))));
            assert_eq!(regex.greedy_search("by"), Ok(Some(String::from("b"))));
            assert_eq!(regex.greedy_search("ba"), Ok(Some(String::from("ba"))));
            assert_eq!(regex.greedy_search("bb"), Ok(Some(String::from("b"))));
            assert_eq!(regex.greedy_search("baaaaa"), Ok(Some(String::from("baaaaa"))));
            assert_eq!(regex.greedy_search("baaaaam"), Ok(Some(String::from("baaaaa"))));
            assert_eq!(regex.greedy_search("kabaaaaam"), Ok(Some(String::from("baaaaa"))));
            assert_eq!(regex.greedy_search("zzzzbaaaam"), Ok(Some(String::from("baaaa"))));
            assert_eq!(regex.greedy_search("zzbabaaaabbam"), Ok(Some(String::from("baaaa"))));
            assert_eq!(regex.greedy_search("ace"), Ok(None));

            assert_eq!(regex.search("baababaaa"), vec!["baa", "ba", "baaa"]);
            assert_eq!(regex.search("b"), vec!["b"]);
            assert_eq!(regex.search("xby"), vec!["b"]);
            assert_eq!(regex.search("xb"), vec!["b"]);
            assert_eq!(regex.search("by"), vec!["b"]);
            assert_eq!(regex.search("ba"), vec!["ba"]);
            assert_eq!(regex.search("bb"), vec!["b", "b"]);
            assert_eq!(regex.search("baaaaa"), vec!["baaaaa"]);
            assert_eq!(regex.search("baaaaam"), vec!["baaaaa"]);
            assert_eq!(regex.search("kabaaaaam"), vec!["baaaaa"]);
            assert_eq!(regex.search("zzzzbaaaam"), vec!["baaaa"]);
            assert_eq!(regex.search("zzbabaaaabbam"), vec!["ba", "baaaa", "b", "ba"]);
            assert_eq!(regex.search("ace"), Vec::<String>::new());
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
            assert!(!regex.full_match("abcdabccc"));
            assert!(!regex.full_match("zabc"));
            assert!(!regex.full_match("eeabc"));

            assert_eq!(regex.greedy_search("abc"), Ok(Some(String::from("abc"))));
            assert_eq!(regex.greedy_search("abcccc"), Ok(Some(String::from("abcccc"))));
            assert_eq!(regex.greedy_search("abcd"), Ok(Some(String::from("abc"))));
            assert_eq!(regex.greedy_search("abcdabccc"), Ok(Some(String::from("abc"))));
            assert_eq!(regex.greedy_search("zabc"), Ok(None));
            assert_eq!(regex.greedy_search("eeabc"), Ok(None));

            assert_eq!(regex.search("abc"), vec!["abc"]);
            assert_eq!(regex.search("abcccc"), vec!["abcccc"]);
            assert_eq!(regex.search("abcd"), vec!["abc"]);
            assert_eq!(regex.search("abcdabccc"), vec!["abc"]);
            assert_eq!(regex.search("zabc"), Vec::<String>::new());
            assert_eq!(regex.search("eeabc"), Vec::<String>::new());
        }
    }

    #[test]
    fn regex_end_anchor() {
        let testexpr = "xyz+$";

        let regexp = RegExp::new(testexpr);
        assert!(regexp.is_ok());

        if let Ok(regex) = regexp {
            assert!(regex.full_match("xyz"));
            assert!(!regex.full_match("xxxyzwxyz"));
            assert!(regex.full_match("xyzzzzz"));
            assert!(!regex.full_match("wxyz"));
            assert!(!regex.full_match("xyza"));

            assert_eq!(regex.greedy_search("xyz"), Ok(Some(String::from("xyz"))));
            assert_eq!(regex.greedy_search("xxxyzwxyz"), Ok(Some(String::from("xyz"))));
            assert_eq!(regex.greedy_search("xyzzzz"), Ok(Some(String::from("xyzzzz"))));
            assert_eq!(regex.greedy_search("wxyz"), Ok(Some(String::from("xyz"))));
            assert_eq!(regex.greedy_search("xyzaa"), Ok(None));

            assert_eq!(regex.search("xyz"), vec!["xyz"]);
            assert_eq!(regex.search("xxxyzwxyz"), vec!["xyz"]);
            assert_eq!(regex.search("xyzzzz"), vec!["xyzzzz"]);
            assert_eq!(regex.search("wxyz"), vec!["xyz"]);
            assert_eq!(regex.search("xyzaa"), Vec::<String>::new());
        }
    }

    #[test]
    fn regex_anchors() {
        let testexpr = "^a*$";

        let regexp = RegExp::new(testexpr);
        assert!(regexp.is_ok());

        if let Ok(regex) = regexp {
            assert!(regex.full_match(""));
            assert!(regex.full_match("a"));
            assert!(!regex.full_match("b"));
            assert!(!regex.full_match("ab"));

            assert_eq!(regex.greedy_search(""), Ok(Some(String::from(""))));
            assert_eq!(regex.greedy_search("a"), Ok(Some(String::from("a"))));
            assert_eq!(regex.greedy_search("b"), Ok(None));
            assert_eq!(regex.greedy_search("ab"), Ok(None));

            assert_eq!(regex.search(""), vec![""]);
            assert_eq!(regex.search("a"), vec!["a"]);
            assert_eq!(regex.search("b"), Vec::<String>::new());
            assert_eq!(regex.search("ab"), Vec::<String>::new());
        }
    }

    #[test]
    fn regex_bad_anchor() {
        let testexpr = "$Dhelmise";

        let regexp = RegExp::new(testexpr);
        assert!(regexp.is_ok());

        if let Ok(regex) = regexp {
            assert!(!regex.full_match("Dhelmise"));

            assert_eq!(regex.greedy_search("Dhelmise"), Ok(None));

            assert_eq!(regex.search("Dhelmise"), Vec::<String>::new());
        }
    }
}
