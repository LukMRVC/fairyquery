use std::{fmt::Debug, iter::Peekable, str::Chars};

use crate::error::Result;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

/// Token is the representation of a token in the SQL language.
///
/// These are the representation of token strings used in the SQL language,
/// as a concrete types that are better to work with.
/// Any token that needs it's original representation passed to the parser carries
/// an owned string.
// TODO: Maybe these owned strings can be only borrowed
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /// General identified, optional quotes are stripped
    Ident(String),
    /// String literal, with quotes stripped, encoded as UTF-8
    String(String),
    /// Any number literal, floating or integer
    Number(String),
    /// SQL keyword, like FROM or SELECT, WITH, WHERE, etc.
    Keyword(Keyword),

    Period,              // .
    Comma,               // ,
    Semicolon,           // ;
    Colon,               // :
    ParenOpen,           // (
    ParenClose,          // )
    GreaterThan,         // >
    GreaterThanOrEqual,  // >=
    LessThan,            // <
    LessThanOrEqual,     // <=
    NotEqual,            // !=
    LessGreaterNotEqual, // <>
    Equal,               // =
    Plus,                // +
    Minus,               // -
    Asterisk,            // *
    Slash,               // /
    Percent,             // %
    Ampersand,           // &
    Pipe,                // |
    Question,            // ?
    Exclamation,         // !
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Ampersand => "&",
            Self::Asterisk => "*",
            Self::Colon => ":",
            Self::Comma => ",",
            Self::Equal => "=",
            Self::Exclamation => "!",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::Ident(s) => s,
            // shortcircuit current call on just use fmt on Keyword "s"
            Self::Keyword(s) => return s.fmt(f),
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::LessGreaterNotEqual => "<>",
            Self::Minus => "-",
            Self::NotEqual => "!=",
            Self::Number(s) => s,
            Self::ParenClose => ")",
            Self::ParenOpen => "(",
            Self::Period => ".",
            Self::Percent => "%",
            Self::Pipe => "|",
            Self::Plus => "+",
            Self::Question => "?",
            Self::Semicolon => ";",
            Self::Slash => "/",
            Self::String(s) => s,
        })
    }
}

impl From<Keyword> for Token {
    fn from(k: Keyword) -> Self {
        Self::Keyword(k)
    }
}

/// List of reserved SQL Keyword
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    Insert,
    Update,
    Delete,
    Select,
    From,
    Where,
    Join,
    And,
    Or,
    Group,
    By,
    Having,
    Order,
    Asc,
    Desc,
    Limit,
    Offset,
}

impl TryFrom<&str> for Keyword {
    type Error = &'static str;

    fn try_from(lexeme: &str) -> std::result::Result<Self, Self::Error> {
        // to have a simple life, keywords from lexer will be lowercase
        // so we can just compare them directly with &str and just use an assertion here
        debug_assert!(
            lexeme.chars().all(char::is_lowercase),
            "keyword from lexer must be in lowercase"
        );
        Ok(match lexeme {
            "insert" => Self::Insert,
            "update" => Self::Update,
            "delete" => Self::Delete,
            "select" => Self::Select,
            "from" => Self::From,
            "where" => Self::Where,
            "join" => Self::Join,
            "and" => Self::And,
            "or" => Self::Or,
            "group" => Self::Group,
            "by" => Self::By,
            "having" => Self::Having,
            "order" => Self::Order,
            "asc" => Self::Asc,
            "desc" => Self::Desc,
            "limit" => Self::Limit,
            "offset" => Self::Offset,
            // again, short-circuit here so Err is actually returned
            _ => return Err("not a keyword"),
        })
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Insert => "INSERT",
            Self::Update => "UPDATE",
            Self::Delete => "DELETE",
            Self::Select => "SELECT",
            Self::From => "FROM",
            Self::Where => "WHERE",
            Self::Join => "JOIN",
            Self::And => "AND",
            Self::Or => "OR",
            Self::Group => "GROUP",
            Self::By => "BY",
            Self::Having => "HAVING",
            Self::Order => "ORDER",
            Self::Asc => "ASC",
            Self::Desc => "DESC",
            Self::Limit => "LIMIT",
            Self::Offset => "OFFSET",
        })
    }
}

// by implementing an Iterator for Lexer, we can iterate over token or just collect them
/// Use Lexer as a token iterator
impl Iterator for Lexer<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    // scan the input characters and try to convert them into token
    fn scan(&mut self) -> Result<Option<Token>> {
        // skip any whitespace
        self.chars.next_if(|c| c.is_whitespace());
        // Scan the individual characters, and by matching it, convert it into a token type
        match self.chars.peek() {
            Some('\'') => self.scan_literal_string(),
            Some('"') => self.scan_quoted_ident(),
            Some(c) if c.is_ascii_digit() => self.scan_number(),
            Some(c) if c.is_ascii_alphabetic() => self.scan_ident_or_keyword(),
            Some(_) => self.scan_symbol(),
            None => Ok(None),
        }
    }

    fn scan_symbol(&mut self) -> Result<Option<Token>> {
        let mut token = self.chars.next
        Ok(match self.next() {
            Some(_) => todo!(),
            None => todo!(),
        })
    }
}
