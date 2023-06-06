use std::matches;

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
    subexpression()
        .chain(character('|').chain(subexpression()).repeat())
        .map(|(subexp, remaining)| {
            let expression = remaining
                .into_iter()
                .fold(vec![subexp], |mut acc, (_, nextsubexp)| {
                    acc.push(nextsubexp);

                    acc
                });

            Some(expression)
        })
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
    crate::union![
        MonadicParser::lazy(|| group().map(|group| Some(SubexpressionItem::Group(group)))),
        r#match().map(|r#match| Some(SubexpressionItem::Match(r#match))),
        anchor().map(|anchor| Some(SubexpressionItem::Anchor(anchor))),
        backreference().map(|br| Some(SubexpressionItem::Backreference(br))),
    ]
}

/// `Group ::= '(' Expression ')' Quantifier?`
type Group = (Expression, Option<Quantifier>);

/// Returns a [`MonadicParser`] associated to the grammar rule [`Group`].
fn group() -> MonadicParser<Group> {
    character('(')
        // .chain(character(':').chain(character('?')).optional())
        // .map(|(_, _)| Some(()))
        .chain(expression())
        .map(|(_, e)| Some(e))
        .chain(character(')'))
        .map(|(e, _)| Some(e))
        .chain(quantifier().optional())
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
    crate::union![
        character('.').map(|_| Some(MatchItem::Any)),
        match_character_class().map(|mcc| Some(MatchItem::MatchCharacterClass(mcc))),
        char(),
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
    crate::union![
        character_group().map(|cg| Some(MatchCharacterClass::CharacterGroup(cg))),
        character_class().map(|cc| Some(MatchCharacterClass::CharacterClass(cc))),
    ]
}

/// `CharacterGroup ::= '['  CharacterGroupItem+ ']'`
type CharacterGroup = Vec<CharacterGroupItem>;

/// Returns a [`MonadicParser`] associated to the grammar rule [`CharacterGroup`].
fn character_group() -> MonadicParser<CharacterGroup> {
    character('[')
        // .chain(character('^').optional())
        // .map(|(_, maybe_start)| maybe_start.map(|_| Some(???)))
        .chain(character_group_item().repeat().filter(|v| !v.is_empty()))
        .map(|(_, cg)| Some(cg))
        .chain(character(']'))
        .map(|(cg, _)| Some(cg))
}

/// `CharacterGroupItem ::= CharacterClass | CharacterRange`
#[derive(Debug)]
pub enum CharacterGroupItem {
    /// `CharacterClass ::= '\w' | '\W' | '\d' | '\D | '\s' | '\S`
    CharacterClass(CharacterClass),
    /// `CharacterRange ::= Char '-' Char`
    CharacterRange(CharacterRange),
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`CharacterGroupItem`].
fn character_group_item() -> MonadicParser<CharacterGroupItem> {
    crate::union![
        character_class().map(|cc| Some(CharacterGroupItem::CharacterClass(cc))),
        character_range().map(|range| Some(CharacterGroupItem::CharacterRange(range))),
    ]
}

/// `CharacterRange ::= Char '-' Char`
type CharacterRange = (char, Option<char>);

/// Returns a [`MonadicParser`] associated to the grammar rule [`CharacterRange`].
fn character_range() -> MonadicParser<CharacterRange> {
    legal_character()
        .chain(character('-').chain(legal_character()).optional())
        .map(|(l, maybe)| {
            let maybe_r = maybe.map(|(_, r)| r);

            Some((l, maybe_r))
        })
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
    escaped_character(|c| matches!(c, 'd' | 'D' | 's' | 'S' | 'w' | 'W')).map(|c| match c {
        'w' => Some(CharacterClass::Alphanumeric),
        'W' => Some(CharacterClass::NotAlphanumeric),
        'd' => Some(CharacterClass::Digit),
        'D' => Some(CharacterClass::NotDigit),
        's' => Some(CharacterClass::Whitespace),
        'S' => Some(CharacterClass::NotWhitespace),
        _ => panic!("Should not happen, something is seriously wrong!"),
    })
}

/// Returns a [`MonadicParser`] associated to the grammar rule [`Char`].
///
/// [`Char`]: MatchItem::Char
fn char() -> MonadicParser<MatchItem> {
    crate::union![
        legal_character().map(|c| Some(MatchItem::Char(c))),
        escaped_character(|c| {
            matches!(c, '^' | '$' | '|' | '*' | '?' | '+' | '.' | '\\' | '-' | '(' | ')' | '{' | '}' | '[' | ']')
        })
        .map(|c| Some(MatchItem::Char(c))),
    ]
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
    crate::union![
        escaped_character(|c| matches!(c, 'b' | 'B')).map(|c| match c {
            'b' => Some(Anchor::WordBoundary),
            'B' => Some(Anchor::NotWordBoundary),
            _ => panic!("Should not happen, something is seriously wrong!"),
        }),
        character('$').map(|_| Some(Anchor::End)),
    ]
}

/// `Backreference ::= '\' Integer`
type Backreference = usize;

/// Returns a [`MonadicParser`] associated to the grammar rule [`Backreference`].
fn backreference() -> MonadicParser<Backreference> {
    escaped_character(char::is_ascii_digit).map(|c| usize::try_from(c.to_digit(10)?).ok())
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
    crate::union![
        character('*').map(|_| Some(Quantifier::ZeroOrMore)),
        character('+').map(|_| Some(Quantifier::OneOrMore)),
        character('?').map(|_| Some(Quantifier::ZeroOrOne)),
        range_quantifier().map(|range| Some(Quantifier::Range(range))),
    ]
}

/// `RangeQuantifier ::= '{' RangeQuantifierLowerBound? ( ',' RangeQuantifierUpperBound? )? '}'`
type RangeQuantifier = (usize, Option<usize>);

/// Returns a [`MonadicParser`] associated to the grammar rule [`RangeQuantifier`].
fn range_quantifier() -> MonadicParser<RangeQuantifier> {
    character('{')
        .chain(number().optional())
        .map(|(_, l)| l.or(Some(0)))
        .chain(character(',').chain(number().optional()).optional())
        .map(|(l, maybe)| {
            let maybe_r = maybe.map_or_else(|| Some(l), |(_, r)| r);

            Some((l, maybe_r))
        })
        .chain(character('}'))
        .map(|(range, _)| Some(range))
}
