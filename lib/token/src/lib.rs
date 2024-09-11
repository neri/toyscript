//! Simple Tokenizer
#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::borrow::{Cow, ToOwned};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::Write;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut, Range};
use core::{cmp, str};
use core::{fmt, ops::ControlFlow};

mod utf8;
use utf8::*;

#[cfg(test)]
mod tests;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType<KEYWORD> {
    /// End of File
    Eof,
    /// White space
    ///
    /// However, normal whitesapce characters are removed during tokenize.
    Whitespace,
    /// Line Comment
    LineComment,
    /// Block Comment
    BlockComment,
    /// New Line
    NewLine,
    /// Known Keyword
    Keyword(KEYWORD),
    /// Identifier
    Identifier,
    /// Symbol Character
    Symbol(char),
    /// Numeric Literal
    NumericLiteral,
    /// Floating Number Literal
    FloatingNumberLiteral,
    /// Broken Numeric Literal
    BrokenNumber,
    /// String Literal
    StringLiteral(QuoteType),
    /// Broken String Literal
    BrokenString,
    /// Uncategorized
    Uncategorized,
}

impl<KEYWORD> TokenType<KEYWORD> {
    pub const SINGLE_QUOTED_STRING_LITERAL: Self = Self::StringLiteral(QuoteType::SingleQuote);
    pub const DOUBLE_QUOTED_STRING_LITERAL: Self = Self::StringLiteral(QuoteType::DoubleQuote);
    pub const BACK_QUOTED_STRING_LITERAL: Self = Self::StringLiteral(QuoteType::BackQuote);

    pub const OPEN_PARENTHESIS: Self = Self::Symbol('(');
    pub const CLOSE_PARENTHESIS: Self = Self::Symbol(')');
    pub const OPEN_BRACE: Self = Self::Symbol('{');
    pub const CLOSE_BRACE: Self = Self::Symbol('}');
    pub const OPEN_BRACKET: Self = Self::Symbol('[');
    pub const CLOSE_BRACKET: Self = Self::Symbol(']');

    #[inline]
    pub fn is_ignorable(&self) -> bool {
        match self {
            TokenType::NewLine
            | TokenType::Whitespace
            | TokenType::LineComment
            | TokenType::BlockComment => true,
            _ => false,
        }
    }

    #[inline]
    pub fn convert<KEYWORD2>(&self) -> TokenType<KEYWORD2> {
        match self {
            TokenType::Keyword(_) | TokenType::Identifier => TokenType::Identifier,
            TokenType::Whitespace => TokenType::Whitespace,
            TokenType::Eof => TokenType::Eof,
            TokenType::LineComment => TokenType::LineComment,
            TokenType::BlockComment => TokenType::BlockComment,
            TokenType::NewLine => TokenType::NewLine,
            TokenType::Symbol(v) => TokenType::Symbol(*v),
            TokenType::NumericLiteral => TokenType::NumericLiteral,
            TokenType::FloatingNumberLiteral => TokenType::FloatingNumberLiteral,
            TokenType::BrokenNumber => TokenType::BrokenNumber,
            TokenType::StringLiteral(v) => TokenType::StringLiteral(*v),
            TokenType::BrokenString => TokenType::BrokenString,
            TokenType::Uncategorized => TokenType::Uncategorized,
        }
    }
}

impl<KEYWORD: core::fmt::Display + core::fmt::Debug> core::fmt::Display for TokenType<KEYWORD>
where
    Self: Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Eof => f.write_str("EndOfFile"),
            TokenType::StringLiteral(_) => f.write_str("StringLiteral"),
            TokenType::Keyword(keyword) => write!(f, "{}", keyword),
            TokenType::Symbol(symbol) => f.write_char(*symbol),
            _ => (self as &dyn core::fmt::Debug).fmt(f),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuoteType {
    /// '"'
    DoubleQuote,
    /// "'"
    SingleQuote,
    /// "`"
    BackQuote,
}

impl QuoteType {
    #[inline]
    pub const fn as_char(self) -> char {
        match self {
            QuoteType::DoubleQuote => '"',
            QuoteType::SingleQuote => '\'',
            QuoteType::BackQuote => '`',
        }
    }
}

impl fmt::Debug for QuoteType {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_char())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Radix {
    Bin,
    Oct,
    Dec,
    Hex,
}

impl Radix {
    pub const fn is_valid_chars(&self, ch: char) -> bool {
        match self {
            Radix::Bin => matches!(ch, '0'..='1' | '_'),
            Radix::Oct => matches!(ch, '0'..='7' | '_'),
            Radix::Dec => matches!(ch, '0'..='9' | '_'),
            Radix::Hex => matches!(ch, '0'..='9' | 'A'..='F' | 'a'..='f' | '_'),
        }
    }

    #[inline]
    pub const fn value(&self) -> u32 {
        match self {
            Radix::Bin => 2,
            Radix::Oct => 8,
            Radix::Dec => 10,
            Radix::Hex => 16,
        }
    }

    #[inline]
    pub const fn is_invalid_chars(&self, ch: char) -> bool {
        if self.is_valid_chars(ch) {
            false
        } else {
            match ch {
                '0'..='9' | 'A'..='Z' | 'a'..='z' => true,
                _ => false,
            }
        }
    }
}

pub struct Tokenizer<KEYWORD>
where
    KEYWORD: Clone + Copy,
{
    fragments: Vec<TokenFragment<KEYWORD>>,
    line_positions: Vec<(usize, usize)>,
    line_number: usize,
    column_number: usize,
    phase: ParserPhase,
    start: usize,
    start_column: usize,
}

impl<KEYWORD: Copy + Clone> Tokenizer<KEYWORD> {
    pub const MAX_FILE_SIZE: usize = 0x00FF_FFFF;

    #[inline]
    pub fn with_slice<V>(src: &[u8], keyword_resolver: V) -> Result<Tokens<KEYWORD>, TokenError>
    where
        V: Fn(&str) -> Option<KEYWORD>,
    {
        let tokenizer = Self {
            fragments: Default::default(),
            line_positions: Default::default(),
            line_number: Default::default(),
            column_number: Default::default(),
            phase: Default::default(),
            start: Default::default(),
            start_column: Default::default(),
        };
        tokenizer._tokenize(Arc::new(src.to_vec()), keyword_resolver)
    }

    #[inline]
    pub fn new<V>(src: Arc<Vec<u8>>, keyword_resolver: V) -> Result<Tokens<KEYWORD>, TokenError>
    where
        V: Fn(&str) -> Option<KEYWORD>,
    {
        let tokenizer = Self {
            fragments: Default::default(),
            line_positions: Default::default(),
            line_number: Default::default(),
            column_number: Default::default(),
            phase: Default::default(),
            start: Default::default(),
            start_column: Default::default(),
        };
        tokenizer._tokenize(src, keyword_resolver)
    }

    fn _tokenize<V>(
        mut self,
        src: Arc<Vec<u8>>,
        keyword_resolver: V,
    ) -> Result<Tokens<KEYWORD>, TokenError>
    where
        V: Fn(&str) -> Option<KEYWORD>,
    {
        if src.len() > Self::MAX_FILE_SIZE {
            return Err(TokenError::new(TokenErrorKind::TooLargeFile, 0, 0));
        }

        self.start = 0;
        self.start_column = 0;
        self.line_number = 1;
        self.column_number = 1;
        self.phase = ParserPhase::default();
        self.fragments.clear();
        self.line_positions.clear();

        let mut line_start = 0;
        let mut next_finalize = false;
        let mut utf = Utf8StateMachine::new();
        let mut prev_index = 0;
        let mut current_index = 0;
        let mut skip_newline = false;
        let mut next_escaped = false;
        for (index, c) in src.iter().enumerate() {
            if utf.len() == 0 {
                current_index = index;
            }
            utf.push(*c).map_err(|_| {
                TokenError::new(
                    TokenErrorKind::InvalidChar,
                    self.line_number,
                    self.column_number,
                )
            })?;
            if utf.needs_trail_bytes() {
                continue;
            }
            let ch = utf.take_valid_char().ok_or(TokenError::new(
                TokenErrorKind::InvalidChar,
                self.line_number,
                self.column_number,
            ))?;

            if next_finalize {
                self._finalize_phase(&src, current_index, false, &keyword_resolver);
                next_finalize = false;
            }

            let sb = str::from_utf8(&src[self.start..current_index]).unwrap();

            match self.phase {
                ParserPhase::WhiteSpace => {
                    self._next_phase(&src, current_index, prev_index, ch, &keyword_resolver);
                }
                ParserPhase::Identifier => {
                    if self.is_id_trail_char(ch) {
                    } else {
                        self._next_phase(&src, current_index, prev_index, ch, &keyword_resolver);
                    }
                }
                ParserPhase::Zero => match ch {
                    'b' | 'B' => self.phase = ParserPhase::Numeric(Radix::Bin),
                    'o' | 'O' => self.phase = ParserPhase::Numeric(Radix::Oct),
                    'x' | 'X' => self.phase = ParserPhase::Numeric(Radix::Hex),
                    '.' => self.phase = ParserPhase::Floating(FloatingPhase::Dot),
                    'e' | 'E' => self.phase = ParserPhase::Floating(FloatingPhase::E),
                    _ => {
                        let radix = Radix::Dec;
                        if radix.is_valid_chars(ch) {
                            self.phase = ParserPhase::Numeric(radix)
                        } else if radix.is_invalid_chars(ch) {
                            self.phase = ParserPhase::BrokenNumber;
                        } else {
                            self._next_phase(
                                &src,
                                current_index,
                                prev_index,
                                ch,
                                &keyword_resolver,
                            );
                        }
                    }
                },
                ParserPhase::Numeric(radix) => {
                    if radix.is_valid_chars(ch) {
                        //
                    } else if radix == Radix::Dec {
                        match ch {
                            '.' => self.phase = ParserPhase::Floating(FloatingPhase::Dot),
                            'e' | 'E' => self.phase = ParserPhase::Floating(FloatingPhase::E),
                            _ => {
                                if radix.is_invalid_chars(ch) {
                                    self.phase = ParserPhase::BrokenNumber;
                                } else {
                                    self._next_phase(
                                        &src,
                                        current_index,
                                        prev_index,
                                        ch,
                                        &keyword_resolver,
                                    );
                                }
                            }
                        }
                    } else {
                        if radix.is_invalid_chars(ch) {
                            self.phase = ParserPhase::BrokenNumber;
                        } else {
                            self._next_phase(
                                &src,
                                current_index,
                                prev_index,
                                ch,
                                &keyword_resolver,
                            );
                        }
                    }
                }
                ParserPhase::Floating(phase) => match phase {
                    FloatingPhase::Dot => match ch {
                        '0'..='9' => self.phase = ParserPhase::Floating(FloatingPhase::Fraction),
                        '.' => self.phase = ParserPhase::NumericAndDoubleDot,
                        _ => {
                            if Radix::Dec.is_invalid_chars(ch) {
                                self.phase = ParserPhase::BrokenNumber;
                            } else {
                                self._next_phase(
                                    &src,
                                    current_index,
                                    prev_index,
                                    ch,
                                    &keyword_resolver,
                                );
                            }
                        }
                    },
                    FloatingPhase::Fraction => match ch {
                        '0'..='9' => {}
                        'e' | 'E' => {
                            self.phase = ParserPhase::Floating(FloatingPhase::E);
                        }
                        _ => {
                            if Radix::Dec.is_invalid_chars(ch) {
                                self.phase = ParserPhase::BrokenNumber;
                            } else {
                                self._next_phase(
                                    &src,
                                    current_index,
                                    prev_index,
                                    ch,
                                    &keyword_resolver,
                                );
                            }
                        }
                    },
                    FloatingPhase::E => match ch {
                        '+' | '-' => {
                            self.phase = ParserPhase::Floating(FloatingPhase::ExpSign);
                        }
                        '0'..='9' => {
                            self.phase = ParserPhase::Floating(FloatingPhase::Exponent);
                        }
                        _ => {
                            if Radix::Dec.is_invalid_chars(ch) {
                                self.phase = ParserPhase::BrokenNumber;
                            } else {
                                self._next_phase(
                                    &src,
                                    current_index,
                                    prev_index,
                                    ch,
                                    &keyword_resolver,
                                );
                            }
                        }
                    },
                    FloatingPhase::ExpSign => match ch {
                        '0'..='9' => {
                            self.phase = ParserPhase::Floating(FloatingPhase::Exponent);
                        }
                        _ => {
                            if Radix::Dec.is_invalid_chars(ch) {
                                self.phase = ParserPhase::BrokenNumber;
                            } else {
                                self._next_phase(
                                    &src,
                                    current_index,
                                    prev_index,
                                    ch,
                                    &keyword_resolver,
                                );
                            }
                        }
                    },
                    FloatingPhase::Exponent => match ch {
                        '0'..='9' => {}
                        _ => {
                            if Radix::Dec.is_invalid_chars(ch) {
                                self.phase = ParserPhase::BrokenNumber;
                            } else {
                                self._next_phase(
                                    &src,
                                    current_index,
                                    prev_index,
                                    ch,
                                    &keyword_resolver,
                                );
                            }
                        }
                    },
                },
                ParserPhase::NumericAndDoubleDot => {
                    self._next_phase(&src, current_index, prev_index, ch, &keyword_resolver);
                }
                ParserPhase::BrokenNumber => {
                    if self.is_id_trail_char(ch) {
                    } else {
                        self._next_phase(&src, current_index, prev_index, ch, &keyword_resolver);
                    }
                }
                ParserPhase::Slash => match ch {
                    '*' => {
                        // `/*` - block comment
                        self.phase = ParserPhase::BlockComment;
                    }
                    '/' => {
                        // `//` - line comment
                        self.phase = ParserPhase::LineComment;
                    }
                    _ => {
                        self._next_phase(&src, current_index, prev_index, ch, &keyword_resolver);
                    }
                },
                ParserPhase::BlockComment => {
                    if sb.ends_with("*/") {
                        self._next_phase(&src, current_index, prev_index, ch, &keyword_resolver);
                        skip_newline = true;
                    }
                }
                ParserPhase::LineComment => {
                    if ch == '\n' {
                        self._next_phase(&src, current_index, prev_index, ch, &keyword_resolver);
                        skip_newline = true;
                    }
                }
                ParserPhase::Symbol => {
                    self._next_phase(&src, current_index, prev_index, ch, &keyword_resolver);
                }
                ParserPhase::Quote(quote) => {
                    if next_escaped {
                        next_escaped = false
                    } else if ch == '\\' {
                        next_escaped = true
                    } else if ch == quote.as_char() {
                        next_finalize = true;
                    }
                }
                ParserPhase::UnicodeEntity => {
                    self._next_phase(&src, current_index, prev_index, ch, &keyword_resolver);
                }
                ParserPhase::UncontinuableChar => {
                    return Err(TokenError::new(
                        TokenErrorKind::InvalidChar,
                        self.line_number,
                        self.column_number,
                    ))
                }
            }

            match ch {
                '\n' => {
                    let line_end = current_index;
                    self.line_positions.push((line_start, line_end));

                    if skip_newline {
                        skip_newline = false;
                    } else if !matches!(self.phase, ParserPhase::BlockComment) {
                        self.fragments.push(TokenFragment::new(
                            TokenType::NewLine,
                            self.start,
                            line_end,
                        ));
                    }

                    self.column_number = 1;
                    self.line_number += 1;
                    line_start = line_end + 1;
                }

                _ => {
                    self.column_number += 1;
                }
            }

            prev_index = current_index;
        }
        if utf.needs_trail_bytes() {
            return Err(TokenError::new(
                TokenErrorKind::UnexpectedEof,
                self.line_number,
                self.column_number,
            ));
        }
        self._finalize_phase(&src, src.len(), !next_finalize, &keyword_resolver);

        let last = TokenFragment::new(TokenType::Eof, src.len(), src.len());
        self.fragments.push(last);

        let Self {
            fragments,
            line_positions,
            line_number: _,
            column_number: _,
            phase: _,
            start: _,
            start_column: _,
        } = self;

        Ok(Tokens {
            arc_buffer: src.clone(),
            fragments: Arc::new(fragments),
            lines: Arc::new(line_positions),
            last,
        })
    }

    fn _next_phase<V>(
        &mut self,
        src: &[u8],
        current_index: usize,
        prev_index: usize,
        ch: char,
        keyword_resolver: V,
    ) where
        V: Fn(&str) -> Option<KEYWORD>,
    {
        if current_index > prev_index {
            self._finalize_phase(src, current_index, false, keyword_resolver);
        }
        let next_phase = self._next_phase_by_char(ch);
        self.start = current_index;
        self.start_column = self.column_number;
        self.phase = next_phase;
    }

    fn _finalize_phase<V>(
        &mut self,
        src: &[u8],
        position_end: usize,
        is_eof: bool,
        keyword_resolver: V,
    ) where
        V: Fn(&str) -> Option<KEYWORD>,
    {
        let sb = str::from_utf8(&src[self.start..position_end]).unwrap();

        match self.phase {
            ParserPhase::WhiteSpace => {}

            ParserPhase::Identifier => {
                if let Some(keyword) = keyword_resolver(sb) {
                    self.fragments.push(TokenFragment::new(
                        TokenType::Keyword(keyword),
                        self.start,
                        position_end,
                    ));
                } else {
                    self.fragments.push(TokenFragment::new(
                        TokenType::Identifier,
                        self.start,
                        position_end,
                    ));
                }
            }
            ParserPhase::Zero => {
                self.fragments.push(TokenFragment::new(
                    TokenType::NumericLiteral,
                    self.start,
                    position_end,
                ));
            }
            ParserPhase::Numeric(_radix) => {
                self.fragments.push(TokenFragment::new(
                    TokenType::NumericLiteral,
                    self.start,
                    position_end,
                ));
            }
            ParserPhase::Floating(phase) => match phase {
                FloatingPhase::Dot | FloatingPhase::E | FloatingPhase::ExpSign => {
                    self.fragments.push(TokenFragment::new(
                        TokenType::BrokenNumber,
                        self.start,
                        position_end,
                    ));
                }
                FloatingPhase::Fraction | FloatingPhase::Exponent => {
                    self.fragments.push(TokenFragment::new(
                        TokenType::FloatingNumberLiteral,
                        self.start,
                        position_end,
                    ));
                }
            },
            ParserPhase::NumericAndDoubleDot => {
                self.fragments.push(TokenFragment::new(
                    TokenType::NumericLiteral,
                    self.start,
                    position_end - 2,
                ));
                for position in position_end - 2..position_end {
                    self.fragments.push(TokenFragment::new(
                        TokenType::Symbol('.'),
                        position,
                        position + 1,
                    ));
                }
            }
            ParserPhase::BrokenNumber => {
                self.fragments.push(TokenFragment::new(
                    TokenType::BrokenNumber,
                    self.start,
                    position_end,
                ));
            }
            ParserPhase::Symbol => {
                let ch = sb.chars().next().unwrap();
                self.fragments.push(TokenFragment::new(
                    TokenType::Symbol(ch),
                    self.start,
                    position_end,
                ));
            }
            ParserPhase::Slash => {
                self.fragments.push(TokenFragment::new(
                    TokenType::Symbol('/'),
                    self.start,
                    position_end,
                ));
            }
            ParserPhase::LineComment => {
                self.fragments.push(TokenFragment::new(
                    TokenType::LineComment,
                    self.start,
                    position_end,
                ));
            }
            ParserPhase::BlockComment => {
                self.fragments.push(TokenFragment::new(
                    TokenType::BlockComment,
                    self.start,
                    position_end,
                ));
            }
            ParserPhase::Quote(quote) => {
                if is_eof {
                    self.fragments.push(TokenFragment::new(
                        TokenType::BrokenString,
                        self.start,
                        position_end,
                    ));
                } else {
                    assert!(position_end >= self.start + 2);
                    let start = src[self.start];
                    let end = src[position_end - 1];
                    assert_eq!(start, end);
                    self.fragments.push(TokenFragment::new(
                        TokenType::StringLiteral(quote),
                        self.start,
                        position_end,
                    ));
                }
            }
            ParserPhase::UnicodeEntity => {
                self.fragments.push(TokenFragment::new(
                    TokenType::Uncategorized,
                    self.start,
                    position_end,
                ));
            }

            ParserPhase::UncontinuableChar => unreachable!(),
        }
        self.phase = ParserPhase::WhiteSpace;
    }

    #[inline]
    fn _next_phase_by_char(&self, ch: char) -> ParserPhase {
        if Self::is_id_leading_char(ch) {
            ParserPhase::Identifier
        } else if Self::is_whitespace(ch) {
            ParserPhase::WhiteSpace
        } else {
            match ch {
                '0' => ParserPhase::Zero,
                '1'..='9' => ParserPhase::Numeric(Radix::Dec),
                '\x22' => ParserPhase::Quote(QuoteType::DoubleQuote),
                '\x27' => ParserPhase::Quote(QuoteType::SingleQuote),
                '\x60' => ParserPhase::Quote(QuoteType::BackQuote),
                '/' => ParserPhase::Slash,
                '\u{FEFF}' | '\u{FFFE}' => ParserPhase::UncontinuableChar,
                _ => {
                    if ch.is_ascii_graphic() {
                        ParserPhase::Symbol
                    } else {
                        ParserPhase::UnicodeEntity
                    }
                }
            }
        }
    }

    pub fn is_whitespace(ch: char) -> bool {
        match ch {
            '\x09'..='\x0D' | '\x20' => true,

            '\u{0085}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}' => true,

            _ => false,
        }
    }

    #[inline]
    pub fn is_numeric(ch: char) -> bool {
        matches!(ch, '0'..='9')
    }

    #[inline]
    pub fn is_id_leading_char(ch: char) -> bool {
        matches!(ch, 'A'..='Z' | 'a'..='z' | '_' | '$')
    }

    #[inline]
    pub fn is_id_trail_char(&self, ch: char) -> bool {
        Self::is_id_leading_char(ch) || Self::is_numeric(ch)
    }
}

#[derive(Debug, Clone, Copy, Default)]
enum ParserPhase {
    #[default]
    WhiteSpace,

    Identifier,
    Symbol,

    Zero,
    Numeric(Radix),
    Floating(FloatingPhase),
    /// numeric and ".."
    NumericAndDoubleDot,
    BrokenNumber,

    /// `/` - maybe comment in C
    Slash,

    LineComment,
    BlockComment,

    Quote(QuoteType),

    UnicodeEntity,

    /// Non-tokenizable characters
    UncontinuableChar,
}

#[derive(Debug, Clone, Copy)]
enum FloatingPhase {
    Dot,
    Fraction,
    E,
    ExpSign,
    Exponent,
}

#[derive(Debug, Clone, Copy)]
pub struct TokenFragment<KEYWORD>
where
    KEYWORD: Copy + Clone,
{
    token_type: TokenType<KEYWORD>,
    position: TokenPosition,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TokenPosition(pub (u32, u32));

impl TokenPosition {
    #[inline]
    pub const fn new(start: u32, end: u32) -> Self {
        Self((start, end))
    }

    #[inline]
    pub const fn new_at(position: usize) -> Self {
        Self((position as u32, position as u32))
    }

    #[inline]
    pub const fn empty() -> Self {
        Self((0, 0))
    }

    #[inline]
    pub fn start(&self) -> usize {
        (self.0).0 as usize
    }

    #[inline]
    pub fn end(&self) -> usize {
        (self.0).1 as usize
    }

    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.start()..self.end()
    }

    #[inline]
    pub fn merged(&self, other: &Self) -> Self {
        Self(((self.0).0.min((other.0).0), (self.0).1.max((other.0).1)))
    }

    #[inline]
    pub fn is_continuous(&self, next: &Self) -> bool {
        self.end() == next.start()
    }
}

impl core::fmt::Debug for TokenPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(start: {}, end: {})", self.0 .0, self.0 .1)
    }
}

impl<KEYWORD: Copy + Clone> TokenFragment<KEYWORD> {
    #[inline]
    pub fn new(token_type: TokenType<KEYWORD>, file_start: usize, file_end: usize) -> Self {
        Self {
            token_type,
            position: TokenPosition::new(file_start as u32, file_end as u32),
        }
    }
}

#[derive(Debug)]
pub struct TokenError {
    kind: TokenErrorKind,
    line: usize,
    column: usize,
}

impl TokenError {
    #[inline]
    pub fn new(kind: TokenErrorKind, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }

    #[inline]
    pub const fn kind(&self) -> TokenErrorKind {
        self.kind
    }

    #[inline]
    pub const fn line(&self) -> usize {
        self.line
    }

    #[inline]
    pub const fn column(&self) -> usize {
        self.column
    }

    #[inline]
    pub const fn position(&self) -> (usize, usize) {
        (self.line, self.column)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenErrorKind {
    InvalidChar,
    UnexpectedEof,
    TooLargeFile,
}

pub struct ArcStringSlice {
    buffer: Arc<Vec<u8>>,
    range: TokenPosition,
}

impl ArcStringSlice {
    #[inline]
    unsafe fn from_buffer(buffer: &Arc<Vec<u8>>, range: TokenPosition) -> Self {
        Self {
            buffer: buffer.clone(),
            range,
        }
    }

    #[inline]
    pub fn from_str(str: &str, range: TokenPosition) -> Self {
        Self {
            buffer: Arc::new(str.as_bytes().to_vec()),
            range,
        }
    }

    #[inline]
    pub fn empty_at(position: usize) -> Self {
        Self {
            buffer: Arc::new(Vec::new()),
            range: TokenPosition::new_at(position),
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.buffer
            .get(self.range.range())
            .map(|v| unsafe { core::str::from_utf8_unchecked(v) })
            .unwrap_or_default()
    }

    #[inline]
    pub const fn position(&self) -> TokenPosition {
        self.range
    }
}

impl core::fmt::Debug for ArcStringSlice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArcStringSlice")
            .field("source", &self.as_str())
            .field("range", &self.range)
            .finish()
    }
}

impl ToString for ArcStringSlice {
    #[inline]
    fn to_string(&self) -> String {
        self.as_str().to_owned()
    }
}

impl Clone for ArcStringSlice {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            range: self.range.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Token<KEYWORD> {
    str: ArcStringSlice,
    token_type: TokenType<KEYWORD>,
}

impl<KEYWORD> Token<KEYWORD> {
    #[inline]
    pub fn source(&self) -> &str {
        self.str.as_str()
    }

    #[inline]
    pub fn source_arc(&self) -> &ArcStringSlice {
        &self.str
    }

    #[inline]
    pub fn token_type(&self) -> &TokenType<KEYWORD> {
        &self.token_type
    }

    #[inline]
    pub fn position(&self) -> TokenPosition {
        self.str.range
    }

    #[inline]
    pub fn into_keyword(self) -> Result<KeywordToken<KEYWORD>, Token<KEYWORD>> {
        KeywordToken::from_token(self)
    }

    #[inline]
    pub fn is_continuous(&self, next: &Self) -> bool {
        self.position().is_continuous(&next.position())
    }

    pub fn convert<F, KEYWORD2>(&self, keyword_resolver: F) -> Token<KEYWORD2>
    where
        F: Fn(&str) -> Option<KEYWORD2>,
        KEYWORD: Copy + Clone + PartialEq,
        KEYWORD2: PartialEq,
    {
        match self.token_type() {
            TokenType::Keyword(_) | TokenType::Identifier => {
                if let Some(key2) = keyword_resolver(self.source()) {
                    Token::<KEYWORD2> {
                        str: self.str.clone(),
                        token_type: TokenType::Keyword(key2),
                    }
                } else {
                    Token::<KEYWORD2> {
                        str: self.str.clone(),
                        token_type: TokenType::Identifier,
                    }
                }
            }
            _ => Token::<KEYWORD2> {
                str: self.str.clone(),
                token_type: self.token_type.convert(),
            },
        }
    }

    pub fn radix(&self) -> Option<(usize, Radix)> {
        let mut chars = self.source().chars();
        match chars.next()? {
            '+' | '-' => match chars.next()? {
                '0' => match chars.next() {
                    Some(next) => match next {
                        'b' | 'B' => Some((3, Radix::Bin)),
                        'o' | 'O' => Some((3, Radix::Oct)),
                        'x' | 'X' => Some((3, Radix::Hex)),
                        '0'..='9' => Some((1, Radix::Dec)),
                        _ => None,
                    },
                    None => Some((1, Radix::Dec)),
                },
                '1'..='9' => Some((1, Radix::Dec)),
                _ => None,
            },
            '0' => match chars.next() {
                Some(next) => match next {
                    'b' | 'B' => Some((2, Radix::Bin)),
                    'o' | 'O' => Some((2, Radix::Oct)),
                    'x' | 'X' => Some((2, Radix::Hex)),
                    '0'..='9' => Some((0, Radix::Dec)),
                    _ => None,
                },
                None => Some((0, Radix::Dec)),
            },
            '1'..='9' => Some((0, Radix::Dec)),
            _ => None,
        }
    }

    pub fn string_literal<'a>(&'a self) -> Result<(Cow<'a, str>, QuoteType), StringLiteralError> {
        let TokenType::StringLiteral(quote_type) = *self.token_type() else {
            return Err(StringLiteralError::NaT);
        };
        let source = &self.source()[1..self.source().len() - 1];
        if self.source().find('\\').is_none() {
            Ok((Cow::Borrowed(source), quote_type))
        } else {
            self.raw_bytes_literal(false)
                .map(|v| (Cow::Owned(String::from_utf8(v.0).unwrap()), v.1))
        }
    }

    pub fn raw_bytes_literal(
        &self,
        allow_binary: bool,
    ) -> Result<(Vec<u8>, QuoteType), StringLiteralError> {
        #[derive(Debug, PartialEq)]
        enum Phase {
            Default,
            /// escaped by back slash (`\`)
            Escaped,
            /// '/'_h_: raw byte value, next is one of `0-9A-F`
            RawByte,
            /// '\u': unicode literal, next is `{`
            UnicodeStart,
            /// '\u{': unicode literal, next is one of `0-9A-F` or '}'
            Unicode,
        }
        let TokenType::StringLiteral(quote_type) = *self.token_type() else {
            return Err(StringLiteralError::NaT);
        };
        let mut phase = Phase::Default;
        let source = &self.source()[1..self.source().len() - 1];
        if self.source().find('\\').is_none() {
            Ok((source.as_bytes().to_vec(), quote_type))
        } else {
            let position_start = 1;
            let mut code_buf = Vec::with_capacity(5);
            let mut code_start_position = 0;
            let mut sb = Vec::new();
            for (index, ch) in source.bytes().enumerate() {
                match phase {
                    Phase::Default => match ch {
                        b'\\' => phase = Phase::Escaped,
                        _ => sb.push(ch),
                    },
                    Phase::Escaped => {
                        match ch {
                            b't' => {
                                sb.push(b'\t');
                            }
                            b'n' => {
                                sb.push(b'\n');
                            }
                            b'r' => {
                                sb.push(b'\r');
                            }
                            b'u' => {
                                phase = Phase::UnicodeStart;
                                continue;
                            }
                            b'\x21'..=b'\x2f'
                            | b'\x3a'..=b'\x3f'
                            | b'\x5b'..=b'\x5f'
                            | b'\x7b'..=b'\x7e' => {
                                // All symbol characters following the backslash pass through as is.
                                sb.push(ch);
                            }
                            b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f' => {
                                code_start_position = index;
                                phase = Phase::RawByte;
                                code_buf.push(ch);
                                continue;
                            }
                            _ => {
                                return Err(StringLiteralError::InvalidCharSequence(
                                    position_start + index,
                                ))
                            }
                        }
                        phase = Phase::Default
                    }
                    Phase::RawByte => match ch {
                        b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f' => {
                            code_buf.push(ch);
                            let hex = str::from_utf8(&code_buf).unwrap();
                            let raw_byte = u32::from_str_radix(&hex, 16).unwrap();

                            if raw_byte < 0x80 || allow_binary {
                                sb.push(raw_byte as u8);
                            } else {
                                return Err(StringLiteralError::InvalidCharSequence(
                                    position_start + code_start_position,
                                ));
                            }

                            code_buf.clear();
                            phase = Phase::Default;
                        }
                        _ => {
                            return Err(StringLiteralError::InvalidCharSequence(
                                position_start + index,
                            ))
                        }
                    },
                    Phase::UnicodeStart => {
                        if ch == b'{' {
                            // `\u{`
                            code_start_position = index - 2;
                            phase = Phase::Unicode;
                        } else {
                            return Err(StringLiteralError::InvalidCharSequence(
                                position_start + index,
                            ));
                        }
                    }
                    Phase::Unicode => match ch {
                        b'}' => {
                            if code_buf.len() == 0 || code_buf.len() > 5 {
                                return Err(StringLiteralError::InvalidCharSequence(
                                    position_start + index,
                                ));
                            }
                            let hex = str::from_utf8(&code_buf).unwrap();
                            let unicode =
                                match char::from_u32(u32::from_str_radix(&hex, 16).unwrap()) {
                                    Some(v) => v,
                                    None => {
                                        return Err(StringLiteralError::InvalidUnicodeChar(
                                            position_start + code_start_position,
                                        ));
                                    }
                                };
                            sb.extend(unicode.to_string().as_bytes());

                            code_buf.clear();
                            phase = Phase::Default;
                        }
                        b'_' => {}
                        b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f' => {
                            code_buf.push(ch);
                        }
                        _ => {
                            return Err(StringLiteralError::InvalidCharSequence(
                                position_start + index,
                            ))
                        }
                    },
                }
            }
            if phase != Phase::Default {
                return Err(StringLiteralError::InvalidCharSequence(
                    self.source().len() - 1,
                ));
            }

            Ok((sb, quote_type))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringLiteralError {
    /// Not a Thing
    NaT,
    /// Invalid char sequence
    InvalidCharSequence(usize),
    /// Invalid unicode sequence
    InvalidUnicodeChar(usize),
}

#[derive(Debug)]
pub struct KeywordToken<KEYWORD> {
    str: ArcStringSlice,
    keyword: KEYWORD,
}

impl<KEYWORD> KeywordToken<KEYWORD> {
    #[inline]
    pub fn from_token(token: Token<KEYWORD>) -> Result<KeywordToken<KEYWORD>, Token<KEYWORD>> {
        match token.token_type {
            TokenType::Keyword(keyword) => Ok(KeywordToken {
                str: token.str.clone(),
                keyword,
            }),
            _ => Err(token),
        }
    }

    #[inline]
    pub fn into_token(self) -> Token<KEYWORD> {
        Token {
            str: self.str,
            token_type: TokenType::Keyword(self.keyword),
        }
    }

    #[inline]
    pub fn as_token(&self) -> Token<KEYWORD>
    where
        KEYWORD: Copy,
    {
        Token {
            str: self.str.clone(),
            token_type: TokenType::Keyword(self.keyword),
        }
    }

    #[inline]
    pub fn source(&self) -> &str {
        self.str.as_str()
    }

    #[inline]
    pub fn keyword(&self) -> &KEYWORD {
        &self.keyword
    }

    #[inline]
    pub fn position(&self) -> TokenPosition {
        self.str.range
    }
}

impl<KEYWORD> From<KeywordToken<KEYWORD>> for Token<KEYWORD> {
    #[inline]
    fn from(value: KeywordToken<KEYWORD>) -> Self {
        value.into_token()
    }
}

pub struct Tokens<KEYWORD>
where
    KEYWORD: Copy + Clone,
{
    arc_buffer: Arc<Vec<u8>>,
    fragments: Arc<Vec<TokenFragment<KEYWORD>>>,
    last: TokenFragment<KEYWORD>,
    lines: Arc<Vec<(usize, usize)>>,
}

pub struct TokenStream<KEYWORD>
where
    KEYWORD: Copy + Clone,
{
    arc_buffer: Arc<Vec<u8>>,
    fragments: Arc<Vec<TokenFragment<KEYWORD>>>,
    last: TokenFragment<KEYWORD>,
    lines: Arc<Vec<(usize, usize)>>,
    index: usize,
    range: Range<usize>,
}

pub struct Snapshot<'a, KEYWORD>
where
    KEYWORD: Copy + Clone,
{
    inner: TokenStream<KEYWORD>,
    _phantom: PhantomData<&'a ()>,
}

impl<KEYWORD> Deref for Snapshot<'_, KEYWORD>
where
    KEYWORD: Copy + Clone,
{
    type Target = TokenStream<KEYWORD>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<KEYWORD> DerefMut for Snapshot<'_, KEYWORD>
where
    KEYWORD: Copy + Clone,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<KEYWORD: Copy + Clone> Tokens<KEYWORD> {
    #[inline]
    pub fn line_positions(&self) -> &[(usize, usize)] {
        &self.lines
    }

    pub fn line_index(&self, position: usize) -> Option<usize> {
        self.lines
            .binary_search_by(|(line_start, line_end)| {
                if position < *line_start {
                    cmp::Ordering::Greater
                } else if position > *line_end {
                    cmp::Ordering::Less
                } else {
                    cmp::Ordering::Equal
                }
            })
            .ok()
    }

    #[inline]
    pub fn stream(&self) -> TokenStream<KEYWORD> {
        TokenStream {
            arc_buffer: self.arc_buffer.clone(),
            fragments: self.fragments.clone(),
            last: self.last,
            lines: self.lines.clone(),
            index: 0,
            range: 0..self.fragments.len(),
        }
    }
}

impl<KEYWORD: Copy + Clone> fmt::Debug for Tokens<KEYWORD> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenStream").finish()
    }
}

impl<KEYWORD: Copy + Clone + PartialEq> TokenStream<KEYWORD> {
    #[inline]
    pub fn snapshot<'a>(&'a self) -> Snapshot<'a, KEYWORD> {
        Snapshot {
            inner: Self {
                arc_buffer: self.arc_buffer.clone(),
                fragments: self.fragments.clone(),
                last: self.last,
                lines: self.lines.clone(),
                index: self.index,
                range: self.index..self.range.end,
            },
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn make_replay<'a>(&'a self, snapshot: Snapshot<'a, KEYWORD>) -> TokenStream<KEYWORD> {
        let range_end = if self.index > 0 { self.index - 1 } else { 0 };
        let last_position = TokenPosition::new_at(self.fragments[self.index].position.start());
        TokenStream {
            arc_buffer: self.arc_buffer.clone(),
            fragments: self.fragments.clone(),
            last: TokenFragment {
                token_type: TokenType::Eof,
                position: last_position,
            },
            lines: self.lines.clone(),
            index: snapshot.index,
            range: snapshot.index..range_end.max(snapshot.index),
        }
    }

    pub(crate) fn get(&self, index: usize) -> Option<Token<KEYWORD>> {
        let fragment = self
            .range
            .contains(&index)
            .then(|| self.fragments.get(index))
            .flatten()?;

        Some(Token {
            str: unsafe { ArcStringSlice::from_buffer(&self.arc_buffer, fragment.position) },
            token_type: fragment.token_type,
        })
    }

    #[inline]
    pub const fn index(&self) -> usize {
        self.index
    }

    pub fn eof(&self) -> Token<KEYWORD> {
        Token {
            str: unsafe { ArcStringSlice::from_buffer(&self.arc_buffer, self.last.position) },
            token_type: self.last.token_type,
        }
    }

    #[inline]
    pub fn peek(&self) -> Option<Token<KEYWORD>> {
        self.get(self.index)
    }

    #[inline]
    pub fn peek_last(&self) -> Option<Token<KEYWORD>> {
        if self.index > 0 {
            let fragment = self.fragments.get(self.index - 1)?;

            Some(Token {
                str: unsafe { ArcStringSlice::from_buffer(&self.arc_buffer, fragment.position) },
                token_type: fragment.token_type,
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn shift(&mut self) -> Option<Token<KEYWORD>> {
        self.get(self.index).map(|v| {
            self.index += 1;
            v
        })
    }

    #[inline]
    pub fn unshift(&mut self) {
        if self.index > self.range.start {
            self.index -= 1;
        }
    }

    /// Returns the next non-blank token.
    #[inline]
    pub fn next_non_blank(&mut self) -> Token<KEYWORD> {
        self.skip_ignorable();
        self.shift().unwrap_or(self.eof())
    }

    /// Returns the token immediately following the last token, if one exists.
    pub fn peek_immed(&self) -> Option<Token<KEYWORD>> {
        let Some(current) = self.peek_last() else {
            return None;
        };
        self.peek().map(|token| {
            if current.position().end() == token.position().start() {
                token
            } else {
                Token {
                    str: ArcStringSlice::empty_at(current.position().end()),
                    token_type: TokenType::Whitespace,
                }
            }
        })
    }

    /// Returns the token immediately following the last token, if one exists.
    pub fn next_immed(&mut self) -> Option<Token<KEYWORD>> {
        let Some(current) = self.peek_last() else {
            return None;
        };
        self.peek().map(|token| {
            if current.position().end() == token.position().start() {
                self.index += 1;
                token
            } else {
                Token {
                    str: ArcStringSlice::empty_at(current.position().end()),
                    token_type: TokenType::Whitespace,
                }
            }
        })
    }

    #[inline]
    pub fn skip_ignorable(&mut self) {
        while let Some(token) = self.peek() {
            if token.token_type().is_ignorable() {
                self.index += 1;
            } else {
                break;
            }
        }
    }

    #[inline]
    pub fn transaction<F, B, C>(&mut self, kernel: F) -> Result<C, B>
    where
        F: FnOnce(&mut Snapshot<KEYWORD>) -> ControlFlow<B, C>,
    {
        let mut snapshot = self.snapshot();
        match kernel(&mut snapshot) {
            ControlFlow::Continue(result) => {
                self.index = snapshot.index;
                Ok(result)
            }
            ControlFlow::Break(err) => Err(err),
        }
    }

    pub fn expect_fn<F>(&mut self, kernel: F) -> Result<Token<KEYWORD>, Token<KEYWORD>>
    where
        F: FnOnce(&Token<KEYWORD>) -> bool,
    {
        self.transaction(|tokens| {
            let token = tokens.next_non_blank();
            if kernel(&token) {
                ControlFlow::Continue(token)
            } else {
                ControlFlow::Break(token)
            }
        })
    }

    pub fn expect(
        &mut self,
        expected: &[TokenType<KEYWORD>],
    ) -> Result<Token<KEYWORD>, Token<KEYWORD>> {
        self.expect_fn(|token| expected.contains(token.token_type()))
    }

    pub fn expect_symbol(&mut self, expected: char) -> Result<Token<KEYWORD>, Token<KEYWORD>> {
        self.expect(&[TokenType::Symbol(expected)])
    }

    pub fn expect_keyword(
        &mut self,
        expected: KEYWORD,
    ) -> Result<KeywordToken<KEYWORD>, Token<KEYWORD>> {
        self.expect_keywords(&[expected])
    }

    pub fn expect_keywords(
        &mut self,
        expected: &[KEYWORD],
    ) -> Result<KeywordToken<KEYWORD>, Token<KEYWORD>> {
        self.transaction(|tokens| {
            let token = tokens.next_non_blank();
            match KeywordToken::from_token(token) {
                Ok(c) => {
                    if expected.contains(c.keyword()) {
                        ControlFlow::Continue(c)
                    } else {
                        ControlFlow::Break(c.into_token())
                    }
                }
                Err(b) => ControlFlow::Break(b),
            }
        })
    }

    pub fn expect_any_keyword(&mut self) -> Result<KeywordToken<KEYWORD>, Token<KEYWORD>> {
        self.transaction(|tokens| {
            let token = tokens.next_non_blank();
            match KeywordToken::from_token(token) {
                Ok(c) => ControlFlow::Continue(c),
                Err(b) => ControlFlow::Break(b),
            }
        })
    }

    pub fn expect_immed(
        &mut self,
        expected: &[TokenType<KEYWORD>],
    ) -> Result<Token<KEYWORD>, Token<KEYWORD>> {
        self.transaction(|tokens| match tokens.next_immed() {
            Some(token) => {
                if expected.contains(token.token_type()) {
                    ControlFlow::Continue(token)
                } else {
                    ControlFlow::Break(token)
                }
            }
            None => ControlFlow::Break(tokens.eof()),
        })
    }

    pub fn expect_immed_symbol(
        &mut self,
        expected: char,
    ) -> Result<Token<KEYWORD>, Token<KEYWORD>> {
        self.expect_immed(&[TokenType::Symbol(expected)])
    }
}

impl<KEYWORD: Copy + Clone + PartialEq> Iterator for TokenStream<KEYWORD> {
    type Item = Token<KEYWORD>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.shift()
    }
}
