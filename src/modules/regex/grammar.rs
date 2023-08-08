use std::iter;

use crate::union;

use super::alphabet::{any, character, end, escaped, number, string};
use super::parser::MonadicParser;

/// A [`MonadicParser`] defining the rules of a formal grammar.
pub type Grammar<T> = MonadicParser<T>;

// Specification of grammar rules for Regex

/// `Regex ::= Expression END`
pub type Regex = Expression;

/// Returns the [`Grammar`] defining Regex's grammar.
pub fn regex() -> Grammar<Regex> {
    expression() << end()
}

/// `Expression ::= Subexpression ( '|' Subexpression )*`
pub type Expression = Vec<SubExpression>;

/// Returns a [`MonadicParser`] associated to the grammar rule [`Expression`].
fn expression() -> MonadicParser<Expression> {
    (subexpression() & (character('|') >> subexpression()).repeat())
        .map(|(first, mut rest)| Some(iter::once(first).chain(rest.drain(..)).collect()))
}

/// `Subexpression ::= BasicExpression+`
pub type SubExpression = Vec<BasicExpression>;

/// Returns a [`MonadicParser`] associated to the grammar rule [`SubExpression`].
fn subexpression() -> MonadicParser<SubExpression> {
    basic_expression().one_or_more()
}

/// `BasicExpression ::= Anchor | Quantified`
#[derive(Debug)]
pub enum BasicExpression {
    Anchor(Anchor),
    Quantified(Quantified),
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`BasicExpression`].
fn basic_expression() -> MonadicParser<BasicExpression> {
    union!(
        anchor().map(|a| Some(BasicExpression::Anchor(a))),
        quantified().map(|q| Some(BasicExpression::Quantified(q))),
    )
}

/// `Anchor ::= '^' | '$' | '\b' | '\B'`
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Anchor {
    Start,
    End,
    WordBoundary,
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

/// `Quantified ::= Quantifiable Quantifier?`
pub type Quantified = (Quantifiable, Option<Quantifier>);

/// Returns a [`MonadicParser`] associated to the grammar rule [`Quantified`].
fn quantified() -> MonadicParser<Quantified> {
    quantifiable() & quantifier().optional()
}

/// `Quantifiable ::= Group | Match | Backreference`
#[derive(Debug)]
pub enum Quantifiable {
    Group(Group),
    Match(Match),
    Backreference(Backreference),
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`Quantifiable`].
fn quantifiable() -> MonadicParser<Quantifiable> {
    union!(
        group().map(|g| Some(Quantifiable::Group(g))),
        r#match().map(|m| Some(Quantifiable::Match(m))),
        backreference().map(|br| Some(Quantifiable::Backreference(br)))
    )
}

/// `Group ::= '(' ":?"? Expression ')'`
#[derive(Debug)]
pub struct Group {
    pub non_capturing: bool,
    pub index: usize,
    pub expr: Box<Expression>,
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`Group`].
fn group() -> MonadicParser<Group> {
    (character('(') >> string(":?").exists() & MonadicParser::lazy(expression) << character(')'))
        .map(|(non_capturing, expr)| Some(Group { non_capturing, index: 0, expr: Box::new(expr) }))
}

/// `Match ::= '.' | CharacterClass | CharacterGroup | Char`
#[derive(Debug)]
pub enum Match {
    Any,
    CharacterClass(CharacterClass),
    CharacterGroup(CharacterGroup),
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
// pub type CharacterGroup = (bool, Vec<CharacterGroupItem>);
// pub struct CharacterGroup {
//     inverted: bool,
//     items: Vec<CharacterGroupItem>,
// }

/// Returns a [`MonadicParser`] associated to the grammar rule [`CharacterGroup`].
fn character_group() -> MonadicParser<CharacterGroup> {
    character('[') >> character_group_item().one_or_more() << character(']')
    // (character('[') >> character('^').exists() & character_group_item().oneOrMore() << character(']'))
    //     .map(|(inverted, items)| Some(CharacterGroup { inverted, items }))
}

/// `CharacterGroupItem ::= CharacterClass | CharacterRange | Char`
#[derive(Debug)]
pub enum CharacterGroupItem {
    CharacterClass(CharacterClass),
    CharacterRange(CharacterRange),
    Char(char),
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

/// Returns a [`MonadicParser`] associated to the grammar rule for `Char` in [`CharacterGroupItem`]
fn character_group_char() -> MonadicParser<char> {
    let char_group_special = |c: &char| matches!(c, '^' | '\\' | '-' | ']');

    union![
        any().exclude(char_group_special),
        escaped().filter(char_group_special),
        control_char(),
    ]
}

/// `CharacterClass ::= '\w' | '\W' | '\d' | '\D | '\s' | '\S'`
#[derive(Debug)]
pub enum CharacterClass {
    Alphanumeric,
    NotAlphanumeric,
    Digit,
    NotDigit,
    Whitespace,
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

/// Returns a [`MonadicParser`] associated to the grammar rule `Char`.
#[rustfmt::skip]
fn char() -> MonadicParser<char> {
    let special_char = |c: &char| {
        matches!(c, '^' | '$' | '|' | '*' | '?' | '+' | '.' | '\\' | '-' | '(' | ')' | '{' | '}' | '[' | ']')
    };

    union![
        any().exclude(special_char),
        escaped().filter(special_char),
        control_char(),
    ]
}

/// Returns a [`MonadicParser`] associated to control characters.
fn control_char() -> MonadicParser<char> {
    escaped().map(|c| match c {
        't' => Some('\t'),
        'n' => Some('\n'),
        'r' => Some('\r'),
        'v' => Some('\x0b'),
        'f' => Some('\x0c'),
        '0' => Some('\0'),
        _ => None,
    })
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
    ZeroOrMore,
    OneOrMore,
    ZeroOrOne,
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
        .map(|(start, maybe_end)| Some((start, maybe_end.or(Some(Some(start)))?)))
}
