mod alphabet;
mod ast;
mod grammar;
mod language;
mod monadic_parser;

pub use self::grammar::Anchor;

use super::automata::Automata;
use super::error::Error;

use self::grammar::Regex;
use self::language::Language;

pub struct RegExp {
    automata: Automata,
}

impl RegExp {
    pub fn new(automata: Automata) -> Self {
        RegExp { automata }
    }

    pub fn full_match(&self, expr: &str) -> bool {
        self.automata.full_match(expr)
    }

    pub fn greedy_search(&self, expr: &str) -> Option<String> {
        self.automata.greedy_search(expr)
    }

    pub fn global_search(&self, expr: &str) -> Vec<String> {
        self.automata.global_search(expr)
    }
}

pub fn lang() -> Language<Regex> {
    language::regex()
}

impl Language<Regex> {
    /// Compiles `expr` as a regular expression into a [`RegExp`].
    pub fn compile(&self, expr: &str) -> Result<RegExp, Error> {
        Ok(RegExp::new(self.parse(expr)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_realistic() {
        let regexp_lang = language::regex();

        let regexp = regexp_lang.compile("(a|b)*cd?e+f*");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

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

        assert_eq!(regex.greedy_search("ce"), Some(String::from("ce")));
        assert_eq!(regex.greedy_search("ace"), Some(String::from("ace")));
        assert_eq!(regex.greedy_search("aaabbbababce"), Some(String::from("aaabbbababce")));
        assert_eq!(regex.greedy_search("cde"), Some(String::from("cde")));
        assert_eq!(regex.greedy_search("cef"), Some(String::from("cef")));
        assert_eq!(regex.greedy_search("cefffff"), Some(String::from("cefffff")));
        assert_eq!(regex.greedy_search("bacdefffff"), Some(String::from("bacdefffff")));
        assert_eq!(regex.greedy_search("aababacdefffff"), Some(String::from("aababacdefffff")));
        assert_eq!(regex.greedy_search("cdde"), None);
        assert_eq!(regex.greedy_search("cdde"), None);
        assert_eq!(regex.greedy_search("aacbdde"), None);
        assert_eq!(regex.greedy_search("e"), None);
        assert_eq!(regex.greedy_search("cdde"), None);
        assert_eq!(regex.greedy_search("cdd"), None);
        assert_eq!(regex.greedy_search(""), None);

        assert_eq!(regex.global_search("ce"), vec!["ce"]);
        assert_eq!(regex.global_search("ace"), vec!["ace"]);
        assert_eq!(regex.global_search("aaabbbababce"), vec!["aaabbbababce"]);
        assert_eq!(regex.global_search("cde"), vec!["cde"]);
        assert_eq!(regex.global_search("cef"), vec!["cef"]);
        assert_eq!(regex.global_search("cefffff"), vec!["cefffff"]);
        assert_eq!(regex.global_search("bacdefffff"), vec!["bacdefffff"]);
        assert_eq!(regex.global_search("aababacdefffff"), vec!["aababacdefffff"]);
        assert_eq!(regex.global_search("cdde"), Vec::<String>::new());
        assert_eq!(regex.global_search("cdde"), Vec::<String>::new());
        assert_eq!(regex.global_search("aacbdde"), Vec::<String>::new());
        assert_eq!(regex.global_search("e"), Vec::<String>::new());
        assert_eq!(regex.global_search("cdde"), Vec::<String>::new());
        assert_eq!(regex.global_search("cdd"), Vec::<String>::new());
        assert_eq!(regex.global_search(""), Vec::<String>::new());
    }

    #[test]
    fn regex_simple() {
        let regexp_lang = language::regex();

        let regexp = regexp_lang.compile("ba*");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

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
        assert_eq!(regex.greedy_search("zzbabaaaabbam"), Some(String::from("baaaa")));
        assert_eq!(regex.greedy_search("ace"), None);

        assert_eq!(regex.global_search("baababaaa"), vec!["baa", "ba", "baaa"]);
        assert_eq!(regex.global_search("b"), vec!["b"]);
        assert_eq!(regex.global_search("xby"), vec!["b"]);
        assert_eq!(regex.global_search("xb"), vec!["b"]);
        assert_eq!(regex.global_search("by"), vec!["b"]);
        assert_eq!(regex.global_search("ba"), vec!["ba"]);
        assert_eq!(regex.global_search("bb"), vec!["b", "b"]);
        assert_eq!(regex.global_search("baaaaa"), vec!["baaaaa"]);
        assert_eq!(regex.global_search("baaaaam"), vec!["baaaaa"]);
        assert_eq!(regex.global_search("kabaaaaam"), vec!["baaaaa"]);
        assert_eq!(regex.global_search("zzzzbaaaam"), vec!["baaaa"]);
        assert_eq!(regex.global_search("zzbabaaaabbam"), vec!["ba", "baaaa", "b", "ba"]);
        assert_eq!(regex.global_search("ace"), Vec::<String>::new());
    }

    #[test]
    fn regex_character_classes() {
        let regexp_lang = language::regex();

        let regexp = regexp_lang.compile(r"\d*");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

        assert!(regex.full_match(""));
        assert!(regex.full_match("1234567890"));
        assert!(!regex.full_match("123d"));
        assert!(!regex.full_match("d345"));

        assert_eq!(regex.global_search(""), vec![""]);
        assert_eq!(regex.global_search("1234567890"), vec!["1234567890"]);
        assert_eq!(regex.global_search("123d"), vec!["123", ""]);
        assert_eq!(regex.global_search("d345"), vec!["", "345"]);

        let regexp = regexp_lang.compile(r"\D+");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

        assert!(regex.full_match("a"));
        assert!(!regex.full_match("1234567890"));
        assert!(!regex.full_match("123d"));
        assert!(!regex.full_match("d345"));

        assert_eq!(regex.global_search(""), Vec::<String>::new());
        assert_eq!(regex.global_search("1234567890"), Vec::<String>::new());
        assert_eq!(regex.global_search("123d"), vec!["d"]);
        assert_eq!(regex.global_search("d345"), vec!["d"]);

        let regexp = regexp_lang.compile(r"\w?");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

        assert!(regex.full_match("a"));
        assert!(regex.full_match("1"));
        assert!(!regex.full_match(r"\"));
        assert!(!regex.full_match("+"));

        assert_eq!(regex.global_search("a"), vec!["a"]);
        assert_eq!(regex.global_search("1"), vec!["1"]);
        assert_eq!(regex.global_search(r"\"), vec!["", ""]);
        assert_eq!(regex.global_search("+"), vec!["", ""]);

        let regexp = regexp_lang.compile(r"\W.");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

        assert!(!regex.full_match("ab"));
        assert!(!regex.full_match("12"));
        assert!(!regex.full_match(r"\"));
        assert!(regex.full_match("+-"));

        assert_eq!(regex.global_search("ab"), Vec::<String>::new());
        assert_eq!(regex.global_search("12"), Vec::<String>::new());
        assert_eq!(regex.global_search(r"\"), Vec::<String>::new());
        assert_eq!(regex.global_search("+-"), vec!["+-"]);

        let regexp = regexp_lang.compile(r".\s.");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

        assert!(regex.full_match("a 1"));
        assert!(regex.full_match(r"\ ?"));

        assert_eq!(regex.global_search("a 1"), vec!["a 1"]);
        assert_eq!(regex.global_search(r"\ ?"), vec![r"\ ?"]);
        let regexp = regexp_lang.compile(r".\S.");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

        assert!(!regex.full_match("a 1"));
        assert!(regex.full_match(r"\*?"));

        assert_eq!(regex.global_search("a 1"), Vec::<String>::new());
        assert_eq!(regex.global_search(r"\*?"), vec![r"\*?"]);
    }

    #[test]
    fn regex_multichar_closure() {
        let regexp_lang = language::regex();

        let regexp = regexp_lang.compile("(ab)*");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

        assert!(regex.full_match(""));
        assert!(regex.full_match("ab"));
        assert!(regex.full_match("abab"));
        assert!(!regex.full_match("b"));
        assert!(!regex.full_match("aba"));
        assert!(!regex.full_match("abc"));

        assert_eq!(regex.greedy_search(""), Some(String::from("")));
        assert_eq!(regex.greedy_search("b"), Some(String::from("")));
        assert_eq!(regex.greedy_search("abab"), Some(String::from("abab")));
        assert_eq!(regex.greedy_search("abaabab"), Some(String::from("abab")));
        assert_eq!(regex.greedy_search("abc"), Some(String::from("ab")));
        assert_eq!(regex.greedy_search("ababaab"), Some(String::from("abab")));

        assert_eq!(regex.global_search("aba"), vec!["ab", ""]);
        assert_eq!(regex.global_search("abab"), vec!["abab"]);
        assert_eq!(regex.global_search("abaab"), vec!["ab", "ab"]);
    }

    #[test]
    fn regex_overlapping_union() {
        let regexp_lang = language::regex();

        let regexp = regexp_lang.compile("(ab)*|(ba)*");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

        assert!(regex.full_match(""));
        assert!(!regex.full_match("aba"));
        assert!(regex.full_match("abab"));
        assert!(!regex.full_match("baab"));
        assert!(!regex.full_match("bab"));

        assert_eq!(regex.greedy_search(""), Some(String::from("")));
        assert_eq!(regex.greedy_search("aba"), Some(String::from("ab")));
        assert_eq!(regex.greedy_search("bab"), Some(String::from("ba")));
        assert_eq!(regex.greedy_search("abba"), Some(String::from("ab")));
        assert_eq!(regex.greedy_search("ababa"), Some(String::from("abab")));

        assert_eq!(regex.global_search("aba"), vec!["ab", ""]);
        assert_eq!(regex.global_search("abba"), vec!["ab", "ba"]);
        assert_eq!(regex.global_search("ababa"), vec!["abab", ""]);
        assert_eq!(regex.global_search("abbaab"), vec!["ab", "ba", "ab"]);
    }

    #[test]
    fn regex_start_anchor() {
        let regexp_lang = language::regex();

        let regexp = regexp_lang.compile("^abc+");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

        assert!(regex.full_match("abc"));
        assert!(regex.full_match("abcccc"));
        assert!(!regex.full_match("abcd"));
        assert!(!regex.full_match("abcdabccc"));
        assert!(!regex.full_match("zabc"));
        assert!(!regex.full_match("eeabc"));

        assert_eq!(regex.greedy_search("abc"), Some(String::from("abc")));
        assert_eq!(regex.greedy_search("abcccc"), Some(String::from("abcccc")));
        assert_eq!(regex.greedy_search("abcd"), Some(String::from("abc")));
        assert_eq!(regex.greedy_search("abcdabccc"), Some(String::from("abc")));
        assert_eq!(regex.greedy_search("zabc"), None);
        assert_eq!(regex.greedy_search("eeabc"), None);

        assert_eq!(regex.global_search("abc"), vec!["abc"]);
        assert_eq!(regex.global_search("abcccc"), vec!["abcccc"]);
        assert_eq!(regex.global_search("abcd"), vec!["abc"]);
        assert_eq!(regex.global_search("abcdabccc"), vec!["abc"]);
        assert_eq!(regex.global_search("zabc"), Vec::<String>::new());
        assert_eq!(regex.global_search("eeabc"), Vec::<String>::new());
    }

    #[test]
    fn regex_end_anchor() {
        let regexp_lang = language::regex();

        let regexp = regexp_lang.compile("xyz+$");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

        assert!(regex.full_match("xyz"));
        assert!(!regex.full_match("xxxyzwxyz"));
        assert!(regex.full_match("xyzzzzz"));
        assert!(!regex.full_match("wxyz"));
        assert!(!regex.full_match("xyza"));

        assert_eq!(regex.greedy_search("xyz"), Some(String::from("xyz")));
        assert_eq!(regex.greedy_search("xxxyzwxyz"), Some(String::from("xyz")));
        assert_eq!(regex.greedy_search("xyzzzz"), Some(String::from("xyzzzz")));
        assert_eq!(regex.greedy_search("wxyz"), Some(String::from("xyz")));
        assert_eq!(regex.greedy_search("xyzaa"), None);

        assert_eq!(regex.global_search("xyz"), vec!["xyz"]);
        assert_eq!(regex.global_search("xxxyzwxyz"), vec!["xyz"]);
        assert_eq!(regex.global_search("xyzzzz"), vec!["xyzzzz"]);
        assert_eq!(regex.global_search("wxyz"), vec!["xyz"]);
        assert_eq!(regex.global_search("xyzaa"), Vec::<String>::new());
    }

    #[test]
    fn regex_escaped_characters() {
        let regexp_lang = language::regex();

        let regexp = regexp_lang.compile(r"\^\$\|\*\?\+\.\(\)\{\}\\\n\t\r\f\v\0");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

        assert!(regex.full_match("^$|*?+.(){}\\\n\t\r\x0c\x0b\0"));
        assert!(!regex.full_match("^$|*?+.(){}\\\n\t\r\x0c\x0b"));

        assert_eq!(
            regex.greedy_search("Ignore this. ^$|*?+.(){}\\\n\t\r\x0c\x0b\0"),
            Some(String::from("^$|*?+.(){}\\\n\t\r\x0c\x0b\0"))
        );
    }

    #[test]
    fn regex_anchors() {
        let regexp_lang = language::regex();

        let regexp = regexp_lang.compile("^a*$");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

        assert!(regex.full_match(""));
        assert!(regex.full_match("a"));
        assert!(!regex.full_match("b"));
        assert!(!regex.full_match("ab"));

        assert_eq!(regex.greedy_search(""), Some(String::from("")));
        assert_eq!(regex.greedy_search("a"), Some(String::from("a")));
        assert_eq!(regex.greedy_search("b"), None);
        assert_eq!(regex.greedy_search("ab"), None);

        assert_eq!(regex.global_search(""), vec![""]);
        assert_eq!(regex.global_search("a"), vec!["a"]);
        assert_eq!(regex.global_search("b"), Vec::<String>::new());
        assert_eq!(regex.global_search("ab"), Vec::<String>::new());
    }

    #[test]
    fn regex_bad_anchor() {
        let regexp_lang = language::regex();

        let regexp = regexp_lang.compile("$Dhelmise");
        assert!(regexp.is_ok());
        let regex = regexp.unwrap();

        assert!(!regex.full_match("Dhelmise"));

        assert_eq!(regex.greedy_search("Dhelmise"), None);

        assert_eq!(regex.global_search("Dhelmise"), Vec::<String>::new());
    }
}
