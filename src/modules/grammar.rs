use std::iter;

use crate::union;

use super::alphabet::*;
use super::monadic_parser::MonadicParser;

/// A [`MonadicParser`] defining the rules of a formal grammar
pub type Grammar<S> = MonadicParser<S>;

impl<S: 'static> Grammar<S> {
    /// Compiles specification `spec` of a formal grammar to the associated [`Grammar`].
    ///
    /// A specification of a formal grammar is a function `fn() -> Grammar<S>` which returns (the [`MonadicParser`] defining) the rules of the formal grammar.
    pub fn compile(spec: fn() -> Grammar<S>) -> Self {
        spec()
    }
}

// Specification of grammar rules for Regex

/// `Regex ::= '^'? Expression`
pub type Regex = Expression;

/// Returns the [`Grammar`] defining Regex's grammar.
pub fn regex() -> Grammar<Regex> {
    expression() << end()
}


/// `Expression ::= Subexpression ('|' Subexpression)*`
pub type Expression = Vec<SubExpression>;

/// Returns a [`MonadicParser`] associated to the grammar rule [`Expression`].
fn expression() -> MonadicParser<Expression> {
    (subexpression() & (character('|') >> subexpression()).repeat())
        .map(|(first, mut rest)|
            Some(iter::once(first).chain(rest.drain(..)).collect())
        )
}


/// `Subexpression ::= BasicExpression+`
pub type SubExpression = Vec<BasicExpression>;

/// Returns a [`MonadicParser`] associated to the grammar rule [`Subexpression`].
fn subexpression() -> MonadicParser<SubExpression> {
    basic_expression().one_or_more()
}


/// `BasicExpression ::= Anchor | Quantified`
#[derive(Debug)]
pub enum BasicExpression {
    /// `Anchor ::= '^' | '$' | '\b' | '\B'`
    Anchor(Anchor),
    /// `Quantified ::= Quantified`
    Quantified(Quantified),
}

fn basic_expression() -> MonadicParser<BasicExpression> {
    union!(
        quantified().map(|q| Some(BasicExpression::Quantified(q))),
        anchor().map(|a| Some(BasicExpression::Anchor(a))),
    )
}


/// `Quantified ::= Quantifiable Quantifier?`
#[derive(Debug)]
pub struct Quantified {
	quantified: Quantifiable,
	quantifier: Option<Quantifier>,
}

fn quantified() -> MonadicParser<Quantified> {
    (quantifiable() & quantifier().optional())
        .map(|(qd, qr)| Some(Quantified{ quantified:qd, quantifier:qr }))
}


/// `Quantifiable ::= Group | Match | Backreference`
#[derive(Debug)]
pub enum Quantifiable {
    /// `Group ::= '(' Expression ')'`
    Group(Group),
    /// `Match ::= MatchItem Quantifier?`
    Match(Match),
    /// `Backreference ::= '\' 1..9`
    Backreference(Backreference),
}

fn quantifiable() -> MonadicParser<Quantifiable> {
    union!(
        group().map(|g| Some(Quantifiable::Group(g))),
        r#match().map(|m| Some(Quantifiable::Match(m))),
        backreference().map(|br| Some(Quantifiable::Backreference(br)))
    )
}


/// `Group ::= '(' Expression ')'`
pub type Group = Expression;
// /// `Group ::= '(' ":?"? Expression ')'
// pub struct Group {
//     non_capturing: bool,
//     expr: Box<Expression>,
// }
// type Group = (bool, Expression);

/// Returns a [`MonadicParser`] associated to the grammar rule [`Group`].
fn group() -> MonadicParser<Group> {
    character('(') >> MonadicParser::lazy(expression) << character(')')
    // (character('(') >> string(":?").exists() & expression() << character(')')).map(
    //     |(non_capturing, expr)| {
    //         Some(Group {
    //             non_capturing: non_capturing,
    //             expr: Box::new(expr),
    //         })
    //     },
    // )
}

/// `Match ::= MatchItem`
#[derive(Debug)]
pub enum Match {
    /// `.`
    Any,
    /// `ChracterClass | ChracterClass`
    CharacterClass(CharacterClass),
    /// `CharacterGroup`
    CharacterGroup(CharacterGroup),
    /// `Char`
    Char(char),
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`Match`].
fn r#match() -> MonadicParser<Match> {
    union![
        character('.').map(|_| Some(Match::Any)),
        character_class().map(|cc| Some(Match::CharacterClass(cc))),
        character_group().map(|cg| Some(Match::CharacterGroup(cg))),
        char().map(|c| Some(Match::Char(c))),
    ]
}


/// `CharacterGroup ::= '['  CharacterGroupItem+ ']'`
pub type CharacterGroup = Vec<CharacterGroupItem>;
// /// `CharacterGroup ::= '[' ^? CharacterGroupItem+ ']'`
// pub struct CharacterGroup {
//     inverted: bool,
//     items: Vec<CharacterGroupItem>,
// }

/// Returns a [`MonadicParser`] associated to the grammar rule [`CharacterGroup`].
fn character_group() -> MonadicParser<CharacterGroup> {
    character('[') >> character_group_item().one_or_more() << character(']')
    // (character('[') >> character('^').exists() & character_group_item().oneOrMore() << character(']'))
    //     .map(|(inverted, items)| Some(CharacterGroup{inverted:inverted, items}))
}


/// `CharacterGroupItem ::= CharacterClass | CharacterRange | Char`
#[derive(Debug)]
pub enum CharacterGroupItem {
    /// `CharacterClass ::= '\w' | '\W' | '\d' | '\D | '\s' | '\S`
    CharacterClass(CharacterClass),
    /// `CharacterRange ::= Char '-' Char`
    CharacterRange(CharacterRange),
    /// `Char ::= Char`
    Char(char)
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`CharacterGroupItem`].
fn character_group_item() -> MonadicParser<CharacterGroupItem> {
    union![
        character_class().map(|cc| Some(CharacterGroupItem::CharacterClass(cc))),
        character_range().map(|range| Some(CharacterGroupItem::CharacterRange(range))),
        character_group_char().map(|c| Some(CharacterGroupItem::Char(c)))
    ]
}


/// `CharacterRange ::= Char '-' Char`
pub type CharacterRange = (char, char);

/// Returns a [`MonadicParser`] associated to the grammar rule [`CharacterGroupItem`].
fn character_range() -> MonadicParser<CharacterRange> {
    character_group_char() << character('-') & character_group_char()
}

/// Returns a [`MonadicParse`] associated to the grammar rule for 'Char' in [`CharacterGroupItem`]
fn character_group_char() -> MonadicParser<char> {
    union![
        any().exclude(char_group_special),
        escaped().filter(char_group_special),
        control_char(),
    ]
}

fn char_group_special(c: &char) -> bool {
    matches!(c, '^' | '\\' | '-' | ']')
}


/// `CharacterClass ::= '\w' | '\W' | '\d' | '\D | '\s' | '\S'`
#[derive(Debug)]
pub enum CharacterClass {
    /// `\w`
    Alphanumeric,
    /// `\W`
    NotAlphanumeric,
    /// `\d`
    Digit,
    /// `\D`
    NotDigit,
    /// `\s`
    Whitespace,
    /// `\S`
    NotWhitespace,
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`CharacterClass`].
fn character_class() -> MonadicParser<CharacterClass> {
    escaped().map(|c| match c {
        'w' => Some(CharacterClass::Alphanumeric),
        'W' => Some(CharacterClass::NotAlphanumeric),
        'd' => Some(CharacterClass::Digit),
        'D' => Some(CharacterClass::NotDigit),
        's' => Some(CharacterClass::Whitespace),
        'S' => Some(CharacterClass::NotWhitespace),
        _ => None,
    })
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`Char`].
///
/// [`Char`]: MatchItem::Char
fn char() -> MonadicParser<char> {
    union![
        any().exclude(special_char),
        escaped().filter(special_char),
        control_char(),
    ]
}

/// Returns a [`MonadicParser`] associated to control characters
fn control_char() -> MonadicParser<char> {
    escaped().map(|c| match c {
        't' => Some('\t'),
        'n' => Some('\n'),
        'r' => Some('\r'),
        'v' => Some('\x0b'),
        'f' => Some('\x0c'),
        '0' => Some('\0'),
        _ => None
    })
}

fn special_char(c: &char) -> bool {
    matches!(c, '^' | '$' | '|' | '*' | '?' | '+' | '.' | '\\' | '-' | '(' | ')' | '{' | '}' | '[' | ']')
}

/// `Anchor ::= '^' | '$' | '\b' | '\B'`
#[derive(Debug)]
pub enum Anchor {
    /// `^`
    Start,
    /// `$`
    End,
    /// `\b`
    WordBoundary,
    /// `\B`
    NotWordBoundary,
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`Anchor`].
fn anchor() -> MonadicParser<Anchor> {
    union![
        character('^').map(|_| Some(Anchor::Start)),
        escaped().map(|c| match c {
            'b' => Some(Anchor::WordBoundary),
            'B' => Some(Anchor::NotWordBoundary),
            _ => None,
        }),
        character('$').map(|_| Some(Anchor::End)),
    ]
}

/// `Backreference ::= '\' 1..9`
pub type Backreference = u32;

/// Returns a [`MonadicParser`] associated to the grammar rule [`Backreference`].
fn backreference() -> MonadicParser<Backreference> {
    escaped().map(|c| c.to_digit(10)).exclude(|&n| n == 0)
}

/// `Quantifier ::= '*' | '+' | '?' | RangeQuantifier`
#[derive(Debug)]
pub enum Quantifier {
    /// `*`
    ZeroOrMore,
    /// `+`
    OneOrMore,
    /// `?`
    ZeroOrOne,
    /// `RangeQuantifier ::= '{' RangeQuantifierLowerBound ( ',' RangeQuantifierUpperBound? )? '}'`
    Range(RangeQuantifier),
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`Quantifier`].
fn quantifier() -> MonadicParser<Quantifier> {
    union![
        character('*').map(|_| Some(Quantifier::ZeroOrMore)),
        character('+').map(|_| Some(Quantifier::OneOrMore)),
        character('?').map(|_| Some(Quantifier::ZeroOrOne)),
        range_quantifier().map(|range| Some(Quantifier::Range(range))),
    ]
}

/// `RangeQuantifier ::= '{' RangeQuantifierLowerBound ( ',' RangeQuantifierUpperBound? )? '}'`
pub type RangeQuantifier = (u32, Option<u32>);

/// Returns a [`MonadicParser`] associated to the grammar rule [`RangeQuantifier`].
fn range_quantifier() -> MonadicParser<RangeQuantifier> {
    (character('{') >> number() & (character(',') >> number().optional()).optional() << character('}'))
        .map(|(start, maybe_end)| Some((start, maybe_end.and(maybe_end?))))
}
