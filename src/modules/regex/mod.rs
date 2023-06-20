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

/// A wrapper around an [`Automata`] defining a parsed regular expression.
pub struct RegExp(Automata);

impl RegExp {
    /// Returns `true` if expr matches the regular expression entirely.
    pub fn full_match(&self, expr: &str) -> bool {
        self.0.full_match(expr)
    }

    /// Returns the longest substring of `expr` which matches the regular expression, or `None` if no such substring of `expr` exists.
    pub fn greedy_search(&self, expr: &str) -> Option<String> {
        self.0.greedy_search(expr)
    }

    /// Returns a list of all substrings of `expr` which matches the regular expression.
    pub fn global_search(&self, expr: &str) -> Vec<String> {
        self.0.global_search(expr)
    }
}

/// Initialise an instance of [`Language<Regex>`], a [`Language`] defining the Regex language.
pub fn init() -> Language<Regex> {
    Language::new(grammar::regex())
}

impl Language<Regex> {
    /// Compiles `expr` as a regular expression into a [`RegExp`].
    pub fn compile(&self, expr: &str) -> Result<RegExp, Error> {
        Ok(RegExp(self.parse(expr)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_realistic() {
        let regex = init();

        let compiled_regexp = regex.compile("(a|b)*cd?e+f*");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(regexp.full_match("ce"));
        assert!(regexp.full_match("ace"));
        assert!(regexp.full_match("aaabbbababce"));
        assert!(regexp.full_match("cde"));
        assert!(regexp.full_match("cef"));
        assert!(regexp.full_match("cefffff"));
        assert!(regexp.full_match("bacdefffff"));
        assert!(regexp.full_match("aababacdefffff"));
        assert!(!regexp.full_match("cdde"));
        assert!(!regexp.full_match("cdde"));
        assert!(!regexp.full_match("aacbdde"));
        assert!(!regexp.full_match("e"));
        assert!(!regexp.full_match("cdde"));
        assert!(!regexp.full_match("cdd"));
        assert!(!regexp.full_match(""));

        assert_eq!(regexp.greedy_search("ce"), Some(String::from("ce")));
        assert_eq!(regexp.greedy_search("ace"), Some(String::from("ace")));
        assert_eq!(regexp.greedy_search("aaabbbababce"), Some(String::from("aaabbbababce")));
        assert_eq!(regexp.greedy_search("cde"), Some(String::from("cde")));
        assert_eq!(regexp.greedy_search("cef"), Some(String::from("cef")));
        assert_eq!(regexp.greedy_search("cefffff"), Some(String::from("cefffff")));
        assert_eq!(regexp.greedy_search("bacdefffff"), Some(String::from("bacdefffff")));
        assert_eq!(regexp.greedy_search("aababacdefffff"), Some(String::from("aababacdefffff")));
        assert_eq!(regexp.greedy_search("cdde"), None);
        assert_eq!(regexp.greedy_search("cdde"), None);
        assert_eq!(regexp.greedy_search("aacbdde"), None);
        assert_eq!(regexp.greedy_search("e"), None);
        assert_eq!(regexp.greedy_search("cdde"), None);
        assert_eq!(regexp.greedy_search("cdd"), None);
        assert_eq!(regexp.greedy_search(""), None);

        assert_eq!(regexp.global_search("ce"), vec!["ce"]);
        assert_eq!(regexp.global_search("ace"), vec!["ace"]);
        assert_eq!(regexp.global_search("aaabbbababce"), vec!["aaabbbababce"]);
        assert_eq!(regexp.global_search("cde"), vec!["cde"]);
        assert_eq!(regexp.global_search("cef"), vec!["cef"]);
        assert_eq!(regexp.global_search("cefffff"), vec!["cefffff"]);
        assert_eq!(regexp.global_search("bacdefffff"), vec!["bacdefffff"]);
        assert_eq!(regexp.global_search("aababacdefffff"), vec!["aababacdefffff"]);
        assert_eq!(regexp.global_search("cdde"), Vec::<String>::new());
        assert_eq!(regexp.global_search("cdde"), Vec::<String>::new());
        assert_eq!(regexp.global_search("aacbdde"), Vec::<String>::new());
        assert_eq!(regexp.global_search("e"), Vec::<String>::new());
        assert_eq!(regexp.global_search("cdde"), Vec::<String>::new());
        assert_eq!(regexp.global_search("cdd"), Vec::<String>::new());
        assert_eq!(regexp.global_search(""), Vec::<String>::new());
    }

    #[test]
    fn regex_simple() {
        let regex = init();

        let compiled_regexp = regex.compile("ba*");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(!regexp.full_match("baababaaa"));
        assert!(regexp.full_match("b"));
        assert!(!regexp.full_match("xby"));
        assert!(!regexp.full_match("xb"));
        assert!(!regexp.full_match("by"));
        assert!(regexp.full_match("ba"));
        assert!(!regexp.full_match("bb"));
        assert!(regexp.full_match("baaaaa"));
        assert!(!regexp.full_match("baaaaam"));
        assert!(!regexp.full_match("kabaaaaam"));
        assert!(!regexp.full_match("zzzzbaaaam"));
        assert!(!regexp.full_match("zzbabaaaabbbam"));
        assert!(!regexp.full_match("ace"));

        assert_eq!(regexp.greedy_search("baababaaa"), Some(String::from("baaa")));
        assert_eq!(regexp.greedy_search("b"), Some(String::from("b")));
        assert_eq!(regexp.greedy_search("xby"), Some(String::from("b")));
        assert_eq!(regexp.greedy_search("xb"), Some(String::from("b")));
        assert_eq!(regexp.greedy_search("by"), Some(String::from("b")));
        assert_eq!(regexp.greedy_search("ba"), Some(String::from("ba")));
        assert_eq!(regexp.greedy_search("bb"), Some(String::from("b")));
        assert_eq!(regexp.greedy_search("baaaaa"), Some(String::from("baaaaa")));
        assert_eq!(regexp.greedy_search("baaaaam"), Some(String::from("baaaaa")));
        assert_eq!(regexp.greedy_search("kabaaaaam"), Some(String::from("baaaaa")));
        assert_eq!(regexp.greedy_search("zzzzbaaaam"), Some(String::from("baaaa")));
        assert_eq!(regexp.greedy_search("zzbabaaaabbam"), Some(String::from("baaaa")));
        assert_eq!(regexp.greedy_search("ace"), None);

        assert_eq!(regexp.global_search("baababaaa"), vec!["baa", "ba", "baaa"]);
        assert_eq!(regexp.global_search("b"), vec!["b"]);
        assert_eq!(regexp.global_search("xby"), vec!["b"]);
        assert_eq!(regexp.global_search("xb"), vec!["b"]);
        assert_eq!(regexp.global_search("by"), vec!["b"]);
        assert_eq!(regexp.global_search("ba"), vec!["ba"]);
        assert_eq!(regexp.global_search("bb"), vec!["b", "b"]);
        assert_eq!(regexp.global_search("baaaaa"), vec!["baaaaa"]);
        assert_eq!(regexp.global_search("baaaaam"), vec!["baaaaa"]);
        assert_eq!(regexp.global_search("kabaaaaam"), vec!["baaaaa"]);
        assert_eq!(regexp.global_search("zzzzbaaaam"), vec!["baaaa"]);
        assert_eq!(regexp.global_search("zzbabaaaabbam"), vec!["ba", "baaaa", "b", "ba"]);
        assert_eq!(regexp.global_search("ace"), Vec::<String>::new());
    }

    #[test]
    fn regex_character_classes() {
        let regex = init();

        let compiled_regexp = regex.compile(r"\d*");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(regexp.full_match(""));
        assert!(regexp.full_match("1234567890"));
        assert!(!regexp.full_match("123d"));
        assert!(!regexp.full_match("d345"));

        assert_eq!(regexp.global_search(""), vec![""]);
        assert_eq!(regexp.global_search("1234567890"), vec!["1234567890"]);
        assert_eq!(regexp.global_search("123d"), vec!["123", ""]);
        assert_eq!(regexp.global_search("d345"), vec!["", "345"]);

        let compiled_regexp = regex.compile(r"\D+");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(regexp.full_match("a"));
        assert!(!regexp.full_match("1234567890"));
        assert!(!regexp.full_match("123d"));
        assert!(!regexp.full_match("d345"));

        assert_eq!(regexp.global_search(""), Vec::<String>::new());
        assert_eq!(regexp.global_search("1234567890"), Vec::<String>::new());
        assert_eq!(regexp.global_search("123d"), vec!["d"]);
        assert_eq!(regexp.global_search("d345"), vec!["d"]);

        let compiled_regexp = regex.compile(r"\w?");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(regexp.full_match("a"));
        assert!(regexp.full_match("1"));
        assert!(!regexp.full_match(r"\"));
        assert!(!regexp.full_match("+"));

        assert_eq!(regexp.global_search("a"), vec!["a"]);
        assert_eq!(regexp.global_search("1"), vec!["1"]);
        assert_eq!(regexp.global_search(r"\"), vec!["", ""]);
        assert_eq!(regexp.global_search("+"), vec!["", ""]);

        let compiled_regexp = regex.compile(r"\W.");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(!regexp.full_match("ab"));
        assert!(!regexp.full_match("12"));
        assert!(!regexp.full_match(r"\"));
        assert!(regexp.full_match("+-"));

        assert_eq!(regexp.global_search("ab"), Vec::<String>::new());
        assert_eq!(regexp.global_search("12"), Vec::<String>::new());
        assert_eq!(regexp.global_search(r"\"), Vec::<String>::new());
        assert_eq!(regexp.global_search("+-"), vec!["+-"]);

        let compiled_regexp = regex.compile(r".\s.");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(regexp.full_match("a 1"));
        assert!(regexp.full_match(r"\ ?"));

        assert_eq!(regexp.global_search("a 1"), vec!["a 1"]);
        assert_eq!(regexp.global_search(r"\ ?"), vec![r"\ ?"]);
        let compiled_regexp = regex.compile(r".\S.");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(!regexp.full_match("a 1"));
        assert!(regexp.full_match(r"\*?"));

        assert_eq!(regexp.global_search("a 1"), Vec::<String>::new());
        assert_eq!(regexp.global_search(r"\*?"), vec![r"\*?"]);
    }

    #[test]
    fn regex_multichar_closure() {
        let regex = init();

        let compiled_regexp = regex.compile("(ab)*");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(regexp.full_match(""));
        assert!(regexp.full_match("ab"));
        assert!(regexp.full_match("abab"));
        assert!(!regexp.full_match("b"));
        assert!(!regexp.full_match("aba"));
        assert!(!regexp.full_match("abc"));

        assert_eq!(regexp.greedy_search(""), Some(String::from("")));
        assert_eq!(regexp.greedy_search("b"), Some(String::from("")));
        assert_eq!(regexp.greedy_search("abab"), Some(String::from("abab")));
        assert_eq!(regexp.greedy_search("abaabab"), Some(String::from("abab")));
        assert_eq!(regexp.greedy_search("abc"), Some(String::from("ab")));
        assert_eq!(regexp.greedy_search("ababaab"), Some(String::from("abab")));

        assert_eq!(regexp.global_search("aba"), vec!["ab", ""]);
        assert_eq!(regexp.global_search("abab"), vec!["abab"]);
        assert_eq!(regexp.global_search("abaab"), vec!["ab", "ab"]);
    }

    #[test]
    fn regex_overlapping_union() {
        let regex = init();

        let compiled_regexp = regex.compile("(ab)*|(ba)*");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(regexp.full_match(""));
        assert!(!regexp.full_match("aba"));
        assert!(regexp.full_match("abab"));
        assert!(!regexp.full_match("baab"));
        assert!(!regexp.full_match("bab"));

        assert_eq!(regexp.greedy_search(""), Some(String::from("")));
        assert_eq!(regexp.greedy_search("aba"), Some(String::from("ab")));
        assert_eq!(regexp.greedy_search("bab"), Some(String::from("ba")));
        assert_eq!(regexp.greedy_search("abba"), Some(String::from("ab")));
        assert_eq!(regexp.greedy_search("ababa"), Some(String::from("abab")));

        assert_eq!(regexp.global_search("aba"), vec!["ab", ""]);
        assert_eq!(regexp.global_search("abba"), vec!["ab", "ba"]);
        assert_eq!(regexp.global_search("ababa"), vec!["abab", ""]);
        assert_eq!(regexp.global_search("abbaab"), vec!["ab", "ba", "ab"]);
    }

    #[test]
    fn regex_start_anchor() {
        let regex = init();

        let compiled_regexp = regex.compile("^abc+");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(regexp.full_match("abc"));
        assert!(regexp.full_match("abcccc"));
        assert!(!regexp.full_match("abcd"));
        assert!(!regexp.full_match("abcdabccc"));
        assert!(!regexp.full_match("zabc"));
        assert!(!regexp.full_match("eeabc"));

        assert_eq!(regexp.greedy_search("abc"), Some(String::from("abc")));
        assert_eq!(regexp.greedy_search("abcccc"), Some(String::from("abcccc")));
        assert_eq!(regexp.greedy_search("abcd"), Some(String::from("abc")));
        assert_eq!(regexp.greedy_search("abcdabccc"), Some(String::from("abc")));
        assert_eq!(regexp.greedy_search("zabc"), None);
        assert_eq!(regexp.greedy_search("eeabc"), None);

        assert_eq!(regexp.global_search("abc"), vec!["abc"]);
        assert_eq!(regexp.global_search("abcccc"), vec!["abcccc"]);
        assert_eq!(regexp.global_search("abcd"), vec!["abc"]);
        assert_eq!(regexp.global_search("abcdabccc"), vec!["abc"]);
        assert_eq!(regexp.global_search("zabc"), Vec::<String>::new());
        assert_eq!(regexp.global_search("eeabc"), Vec::<String>::new());
    }

    #[test]
    fn regex_end_anchor() {
        let regex = init();

        let compiled_regexp = regex.compile("xyz+$");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(regexp.full_match("xyz"));
        assert!(!regexp.full_match("xxxyzwxyz"));
        assert!(regexp.full_match("xyzzzzz"));
        assert!(!regexp.full_match("wxyz"));
        assert!(!regexp.full_match("xyza"));

        assert_eq!(regexp.greedy_search("xyz"), Some(String::from("xyz")));
        assert_eq!(regexp.greedy_search("xxxyzwxyz"), Some(String::from("xyz")));
        assert_eq!(regexp.greedy_search("xyzzzz"), Some(String::from("xyzzzz")));
        assert_eq!(regexp.greedy_search("wxyz"), Some(String::from("xyz")));
        assert_eq!(regexp.greedy_search("xyzaa"), None);

        assert_eq!(regexp.global_search("xyz"), vec!["xyz"]);
        assert_eq!(regexp.global_search("xxxyzwxyz"), vec!["xyz"]);
        assert_eq!(regexp.global_search("xyzzzz"), vec!["xyzzzz"]);
        assert_eq!(regexp.global_search("wxyz"), vec!["xyz"]);
        assert_eq!(regexp.global_search("xyzaa"), Vec::<String>::new());
    }

    #[test]
    fn regex_escaped_characters() {
        let regex = init();

        let compiled_regexp = regex.compile(r"\^\$\|\*\?\+\.\(\)\{\}\\\n\t\r\f\v\0");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(regexp.full_match("^$|*?+.(){}\\\n\t\r\x0c\x0b\0"));
        assert!(!regexp.full_match("^$|*?+.(){}\\\n\t\r\x0c\x0b"));

        assert_eq!(
            regexp.greedy_search("Ignore this. ^$|*?+.(){}\\\n\t\r\x0c\x0b\0"),
            Some(String::from("^$|*?+.(){}\\\n\t\r\x0c\x0b\0"))
        );
    }

    #[test]
    fn regex_anchors() {
        let regex = init();

        let compiled_regexp = regex.compile("^a*$");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(regexp.full_match(""));
        assert!(regexp.full_match("a"));
        assert!(!regexp.full_match("b"));
        assert!(!regexp.full_match("ab"));

        assert_eq!(regexp.greedy_search(""), Some(String::from("")));
        assert_eq!(regexp.greedy_search("a"), Some(String::from("a")));
        assert_eq!(regexp.greedy_search("b"), None);
        assert_eq!(regexp.greedy_search("ab"), None);

        assert_eq!(regexp.global_search(""), vec![""]);
        assert_eq!(regexp.global_search("a"), vec!["a"]);
        assert_eq!(regexp.global_search("b"), Vec::<String>::new());
        assert_eq!(regexp.global_search("ab"), Vec::<String>::new());
    }

    #[test]
    fn regex_bad_anchor() {
        let regex = init();

        let compiled_regexp = regex.compile("$Dhelmise");
        assert!(compiled_regexp.is_ok());
        let regexp = compiled_regexp.unwrap();

        assert!(!regexp.full_match("Dhelmise"));

        assert_eq!(regexp.greedy_search("Dhelmise"), None);

        assert_eq!(regexp.global_search("Dhelmise"), Vec::<String>::new());
    }
}
