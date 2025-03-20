use std::{fmt::Debug, iter::Peekable, str::Chars};

use crate::{error::Result, syntax_error};

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
    Caret,               // ^
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
            Self::Keyword(s) => return std::fmt::Display::fmt(s, f),
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
            Self::Caret => "^",
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
    As,
    Is,
    Not,
    True,
    False,
    Null,
    NaN,
    Infinity,
    Like,
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
            "as" => Self::As,
            "not" => Self::Not,
            "true" => Self::True,
            "false" => Self::False,
            "null" => Self::Null,
            "is" => Self::Is,
            "nan" => Self::NaN,
            "infinity" => Self::Infinity,
            "like" => Self::Like,
            // again, short-circuit here so Err is actually returned
            _ => return Err("not a keyword"),
        })
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Insert => "INSERT",
            Self::Not => "NOT",
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
            Self::As => "AS",
            Self::Having => "HAVING",
            Self::Order => "ORDER",
            Self::Asc => "ASC",
            Self::Desc => "DESC",
            Self::Limit => "LIMIT",
            Self::Offset => "OFFSET",
            Self::True => "TRUE",
            Self::False => "FALSE",
            Self::Null => "NULL",
            Self::NaN => "NAN",
            Self::Infinity => "INFINITY",
            Self::Is => "IS",
            Self::Like => "LIKE",
        })
    }
}

// by implementing an Iterator for Lexer, we can iterate over token or just collect them
/// Use Lexer as a token iterator
impl Iterator for Lexer<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.scan() {
            Ok(Some(token)) => Some(Ok(token)),
            Ok(None) => self
                .chars
                .peek()
                .map(|c| syntax_error!(0, "unexpected character: {}", c)),
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    /// scan the input characters and try to convert them into token
    fn scan(&mut self) -> Result<Option<Token>> {
        // skip any whitespace
        while self.chars.next_if(|c| c.is_whitespace()).is_some() {}
        // Scan the individual characters, and by matching it, convert it into a token type
        match self.chars.peek() {
            Some('\'') => self.scan_literal_string(),
            Some('"') => self.scan_quoted_ident(),
            Some(c) if c.is_ascii_digit() => Ok(self.scan_number()),
            Some(c) if c.is_ascii_alphabetic() => Ok(self.scan_ident_or_keyword()),
            Some(_) => Ok(self.scan_symbol()),
            None => Ok(None),
        }
    }

    /// scan quoted literal string
    fn scan_literal_string(&mut self) -> Result<Option<Token>> {
        if self.chars.next_if_eq(&'\'').is_none() {
            return Ok(None);
        }

        let mut literal = String::new();
        // Now scan and consume the actual string
        // if any a characters '' are encountered, they are handled as escape sequence for an actual
        // single quote character '
        loop {
            match self.chars.next() {
                Some('\'') if self.chars.next_if_eq(&'\'').is_some() => {
                    literal.push('\'');
                }
                Some('\'') => break,
                Some(c) => literal.push(c),
                None => {
                    return syntax_error!(0, "unexpected end of literal string");
                }
            }
        }

        Ok(Some(Token::String(literal)))
    }

    /// scan quoted identifier, like aliases
    fn scan_quoted_ident(&mut self) -> Result<Option<Token>> {
        if self.chars.next_if_eq(&'"').is_none() {
            return Ok(None);
        }

        let mut ident = String::new();

        loop {
            match self.chars.next() {
                Some('"') if self.chars.next_if_eq(&'"').is_some() => {
                    ident.push('"');
                }
                Some('"') => break,
                Some(c) => ident.push(c),
                None => {
                    return syntax_error!(0, "unexpected end of quoted identifier");
                }
            }
        }
        Ok(Some(Token::Ident(ident)))
    }

    /// Scan number literal, exponent notation is not supported at the moment
    fn scan_number(&mut self) -> Option<Token> {
        // get the initial digit
        let mut number_literal = self.chars.next_if(|c| c.is_ascii_digit())?.to_string();
        // now scan all the digits before encountering a radix dot
        while let Some(c) = self.chars.next_if(|c| c.is_ascii_digit()) {
            number_literal.push(c);
        }

        if let Some('.') = self.chars.peek() {
            number_literal.push(self.chars.next().unwrap());
            // now scan all the digits after the radix dot
            while let Some(c) = self.chars.next_if(|c| c.is_ascii_digit()) {
                number_literal.push(c);
            }
        }

        Some(Token::Number(number_literal))
    }

    /// Scan any identifier or keyword as lowercase by SQL convention
    fn scan_ident_or_keyword(&mut self) -> Option<Token> {
        // first of all, it must start with a alphabetic character
        let mut token_str = self
            .chars
            .next_if(|c| c.is_ascii_alphabetic())?
            .to_lowercase()
            .to_string();

        // now scan all the alphanumeric characters, and allow for underscores
        while let Some(c) = self
            .chars
            .next_if(|c| c.is_ascii_alphanumeric() || c == &'_')
        {
            token_str.extend(c.to_lowercase());
        }

        match Keyword::try_from(token_str.as_str()) {
            Ok(keyword) => Some(Token::Keyword(keyword)),
            Err(_) => Some(Token::Ident(token_str)),
        }
    }

    /// scan any symbol token
    fn scan_symbol(&mut self) -> Option<Token> {
        let mut token = match self.chars.peek()? {
            ';' => Token::Semicolon,
            ',' => Token::Comma,
            ':' => Token::Colon,
            '(' => Token::ParenOpen,
            ')' => Token::ParenClose,
            '.' => Token::Period,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Asterisk,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '&' => Token::Ampersand,
            '|' => Token::Pipe,
            '?' => Token::Question,
            '!' => Token::Exclamation,
            '<' => Token::LessThan,
            '>' => Token::GreaterThan,
            '=' => Token::Equal,
            _ => return None,
        };
        // advance the iterator to consume the character processed earlier
        self.chars.next()?;

        // handle two char tokens
        token = match token {
            Token::LessThan if self.chars.next_if_eq(&'>').is_some() => Token::LessGreaterNotEqual,
            Token::LessThan if self.chars.next_if_eq(&'=').is_some() => Token::LessThanOrEqual,
            Token::GreaterThan if self.chars.next_if_eq(&'=').is_some() => {
                Token::GreaterThanOrEqual
            }
            Token::Exclamation if self.chars.next_if_eq(&'=').is_some() => Token::NotEqual,
            _ => token,
        };

        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn into_tokens(input: &str) -> Result<Vec<Token>> {
        Lexer::new(input).collect()
    }

    #[test]
    fn errors_on_unterminated_literal() {
        let tokens = into_tokens("SELECT 'hello");
        assert_eq!(tokens, syntax_error!(0, "unexpected end of literal string"));
    }

    #[test]
    fn errors_on_expected_character() {
        let tokens = into_tokens("SELECT čšě");
        assert_eq!(tokens, syntax_error!(0, "unexpected character: č"));
    }

    #[test]
    fn errors_on_expected_character_in_middle() {
        let tokens = into_tokens("SELčECT");
        assert_eq!(tokens, syntax_error!(0, "unexpected character: č"));
    }

    #[test]
    fn can_scan_symbols() {
        let tokens = into_tokens("  >=  &  ! != <> * ; ,.+-").expect("Failed to collect tokens");

        let expected = vec![
            Token::GreaterThanOrEqual,
            Token::Ampersand,
            Token::Exclamation,
            Token::NotEqual,
            Token::LessGreaterNotEqual,
            Token::Asterisk,
            Token::Semicolon,
            Token::Comma,
            Token::Period,
            Token::Plus,
            Token::Minus,
        ];
        assert_eq!(tokens, expected, "Tokens did not match");
    }

    #[test]
    fn can_scan_literal() {
        let tokens = into_tokens("'hello, world!'").expect("Failed to collect tokens");

        let expected = vec![Token::String("hello, world!".to_string())];
        assert_eq!(tokens, expected, "Failed to scan string literal");
    }

    #[test]
    fn can_scan_literal_with_escape() {
        let tokens = into_tokens("'hello, ''world''!'").expect("Failed to collect tokens");

        let expected = vec![Token::String("hello, 'world'!".to_string())];
        assert_eq!(tokens, expected, "Failed scanning escaped literal");
    }

    #[test]
    fn can_scan_symbol_and_literal() {
        let lexer = Lexer::new("'hello', '''world''!'");
        let tokens = lexer
            .collect::<Result<Vec<_>>>()
            .expect("Failed to collect tokens");

        let expected = vec![
            Token::String("hello".to_string()),
            Token::Comma,
            Token::String("'world'!".to_string()),
        ];
        assert_eq!(tokens, expected, "Failed scanning escaped literal");
    }

    #[test]
    fn can_scan_number_literal() {
        let lexer = Lexer::new("1234");
        let tokens = lexer
            .collect::<Result<Vec<_>>>()
            .expect("Failed to collect tokens");

        let expected = vec![Token::Number("1234".to_string())];
        assert_eq!(tokens, expected, "Failed to scan number literal");

        let lexer = Lexer::new("1234.456");
        let tokens = lexer
            .collect::<Result<Vec<_>>>()
            .expect("Failed to collect tokens");

        let expected = vec![Token::Number("1234.456".to_string())];
        assert_eq!(
            tokens, expected,
            "Failed to scan number literal with fractional part"
        );
    }

    #[test]
    fn can_scan_quoted_literal() {
        let lexer = Lexer::new("\"Hello, \"\"World!\"");
        let tokens = lexer
            .collect::<Result<Vec<_>>>()
            .expect("Failed to collect tokens");

        let expected = vec![Token::Ident("Hello, \"World!".to_string())];
        assert_eq!(tokens, expected, "Failed to scan quoted identifier");
    }

    #[test]
    fn can_scan_idents() {
        let lexer = Lexer::new("SELECT FROM table WHERE");
        let tokens = lexer
            .collect::<Result<Vec<_>>>()
            .expect("Failed to collect tokens");

        let expected = vec![
            Token::Keyword(Keyword::Select),
            Token::Keyword(Keyword::From),
            Token::Ident("table".to_string()),
            Token::Keyword(Keyword::Where),
        ];
        assert_eq!(tokens, expected, "Failed to scan quoted identifier");
    }
}
