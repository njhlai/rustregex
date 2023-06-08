use std::iter;

use crate::union;

use super::alphabet::*;
use super::monadic_parser::MonadicParser;

/// A [`MonadicParser`] defining the rules of a formal grammar
pub type Grammar<S> = MonadicParser<S>;

impl<S: 'static> Grammar<S> {
    // pub fn compile(spec: Spec<T>) -> Self {};

    /// Compiles specification `spec` of a formal grammar to the associated [`Grammar`].
    ///
    /// A specification of a formal grammar is a function `fn() -> Grammar<S>` which returns (the [`MonadicParser`] defining) the rules of the formal grammar.
    pub fn compile(spec: fn() -> Grammar<S>) -> Self {
        spec()
    }
}

/// Returns the [`Grammar`] defining Regex's grammar.
pub fn regex() -> Grammar<Regex> {
    character('^')
        .map(|_| Some(Anchor::Start))
        .optional()
        .chain(expression())
}

// Specification of grammar rules for Regex

/// `Regex ::= '^'? Expression`
pub type Regex = (Option<Anchor>, Expression);

/// `Expression ::= Subexpression ('|' Subexpression)*`
type Expression = Vec<Subexpression>;

/// Returns a [`MonadicParser`] associated to the grammar rule [`Expression`].
fn expression() -> MonadicParser<Expression> {
    (subexpression() & (character('|') >> subexpression()).repeat())
        .map(|(first, mut rest)|
            Some(iter::once(first).chain(rest.drain(..)).collect())
        )
}

/// `Subexpression ::= SubexpressionItem+`
type Subexpression = Vec<SubexpressionItem>;

/// Returns a [`MonadicParser`] associated to the grammar rule [`Subexpression`].
fn subexpression() -> MonadicParser<Subexpression> {
    subexpression_item().repeat().filter(|v| !v.is_empty())
}

/// `SubexpressionItem ::= Match | Group | Anchor | Backreference`
#[derive(Debug)]
pub enum SubexpressionItem {
    /// `Group ::= '(' Expression ')' Quantifier?`
    Group(Group),
    /// `Match ::= MatchItem Quantifier?`
    Match(Match),
    /// `Anchor ::= '^' | '$' | '\b' | '\B'`
    Anchor(Anchor),
    /// `Backreference ::= '\' Integer`
    Backreference(Backreference),
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`SubexpressionItem`].
fn subexpression_item() -> MonadicParser<SubexpressionItem> {
    union![
        MonadicParser::lazy(|| group().map(|group| Some(SubexpressionItem::Group(group)))),
        r#match().map(|r#match| Some(SubexpressionItem::Match(r#match))),
        anchor().map(|anchor| Some(SubexpressionItem::Anchor(anchor))),
        backreference().map(|br| Some(SubexpressionItem::Backreference(br))),
    ]
}

/// `Group ::= '(' Expression ')' Quantifier?`
type Group = Expression;
// type Group = (bool, Expression);

/// Returns a [`MonadicParser`] associated to the grammar rule [`Group`].
fn group() -> MonadicParser<Group> {
    character('(') >> expression() << character(')')
    // (character('(') >> string(":?").optional() & expression() << character(')'))
    //     .map(|(non_capturing, expr)| Some((non_capturing.is_some(), expr)))

}

/// `Match ::= MatchItem Quantifier?`
type Match = (MatchItem, Option<Quantifier>);

/// Returns a [`MonadicParser`] associated to the grammar rule [`Match`].
fn r#match() -> MonadicParser<Match> {
    match_item().chain(quantifier().optional())
}

/// `MatchItem ::= '.' | MatchCharacterClass | Char`
#[derive(Debug)]
pub enum MatchItem {
    /// `.`
    Any,
    /// `MatchCharacterClass ::= CharacterGroup | ChracterClass`
    MatchCharacterClass(MatchCharacterClass),
    /// `Char`
    Char(char),
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`MatchItem`].
fn match_item() -> MonadicParser<MatchItem> {
    union![
        character('.').map(|_| Some(MatchItem::Any)),
        match_character_class().map(|mcc| Some(MatchItem::MatchCharacterClass(mcc))),
        char().map(|c| Some(MatchItem::Char(c))),
    ]
}

/// `MatchCharacterClass ::= CharacterGroup | ChracterClass`
#[derive(Debug)]
pub enum MatchCharacterClass {
    /// `CharacterGroup ::= '['  CharacterGroupItem+ ']'`
    CharacterGroup(CharacterGroup),
    /// `CharacterClass ::= '\w' | '\W' | '\d' | '\D | '\s' | '\S`
    CharacterClass(CharacterClass),
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`CharacterClass`].
fn match_character_class() -> MonadicParser<MatchCharacterClass> {
    union![
        character_group().map(|cg| Some(MatchCharacterClass::CharacterGroup(cg))),
        character_class().map(|cc| Some(MatchCharacterClass::CharacterClass(cc))),
    ]
}

/// `CharacterGroup ::= '['  CharacterGroupItem+ ']'`
type CharacterGroup = Vec<CharacterGroupItem>;
// type CharacterGroup = (bool, Vec<CharacterGroupItem>);

/// Returns a [`MonadicParser`] associated to the grammar rule [`CharacterGroup`].
fn character_group() -> MonadicParser<CharacterGroup> {
    character('[') >> character_group_item().one_or_more() << character(']')
    // (character('[') >> character('^').optional() & character_group_item().oneOrMore() << character(']'))
    //     .map(|(invert, items)| Some((invert.is_some(), items)))
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
        character_group_token().map(|c| Some(CharacterGroupItem::Char(c)))
    ]
}

/// `CharacterRange ::= Char '-' Char`
type CharacterRange = (char, char);

/// Returns a [`MonadicParser`] associated to the grammar rule [`CharacterGroupItem`].
fn character_range() -> MonadicParser<CharacterRange> {
    character_group_token() << character('-') & character_group_token()
}

/// Returns a [`MonadicParse`] associated to the grammar rule for 'Char' in [`CharacterGroupItem`]
fn character_group_token() -> MonadicParser<char> {
    union![
        any().exclude(char_group_special),
        escaped().filter(char_group_special),
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
    ]
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
        escaped().map(|c| match c {
            'b' => Some(Anchor::WordBoundary),
            'B' => Some(Anchor::NotWordBoundary),
            _ => None,
        }),
        character('$').map(|_| Some(Anchor::End)),
    ]
}

/// `Backreference ::= '\' Integer`
type Backreference = u32;

/// Returns a [`MonadicParser`] associated to the grammar rule [`Backreference`].
fn backreference() -> MonadicParser<Backreference> {
    escaped().map(|c| c.to_digit(10))
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
type RangeQuantifier = (u32, Option<u32>);

/// Returns a [`MonadicParser`] associated to the grammar rule [`RangeQuantifier`].
fn range_quantifier() -> MonadicParser<RangeQuantifier> {
    (character('{') >> number() & (character(',') >> number().optional()).optional() << character('}'))
        .map(|(start, maybe_end)| Some((start, maybe_end.and(maybe_end?))))
}
