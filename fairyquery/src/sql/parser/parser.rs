use crate::{error::Result, syntax_error};

use super::{Keyword, Lexer, Token, ast};
use std::{
    fmt::{Debug, Display},
    iter::Peekable,
};

/// The `Parser` struct is responsible for parsing SQL queries.
/// It takes a lexer as input and processes the tokens to generate
/// a structured representation of the SQL query (AST). This structured
/// representation can then be used for further processing, such as
/// query optimization or execution. Parser ensures only that the given SQL
/// is syntactically correct, but does not perform any semantic analysis nor does
/// if any objects actually exists.
///
/// # Fields
///
/// * `lexer` - A `Peekable` iterator over the `Lexer` which provides
///   the tokens to be parsed.
pub struct Parser<'a> {
    pub lexer: Peekable<Lexer<'a>>,
}

impl Parser<'_> {
    pub fn new(statement: &str) -> Parser {
        Parser {
            lexer: Lexer::new(statement).peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<ast::Statement> {
        // Parse the statement
        let statement = self.parse_statement()?;

        // Ensure there are no more tokens left
        if let Some(token) = self.lexer.next().transpose()? {
            return syntax_error!(0, "Unexpected token: {token}");
        }

        Ok(statement)
    }

    fn parse_statement(&mut self) -> Result<ast::Statement> {
        let Some(token) = self.lexer.peek_transposed()? else {
            return syntax_error!(0, "Unexpected end of input");
        };
        match token {
            Token::Keyword(Keyword::Select) => self.parse_select(),
            Token::Keyword(Keyword::Insert) => self.parse_insert(),
            Token::Keyword(Keyword::Update) => self.parse_update(),
            Token::Keyword(Keyword::Delete) => self.parse_delete(),
            _ => syntax_error!(0, "Unexpected token: {token}"),
        }
    }

    /// Parses a SELECT statement
    ///
    /// # Example
    ///
    /// ```sql
    /// SELECT <expressions> FROM <tables and joins> WHERE <expressions>
    /// GROUP BY <expressions> HAVING <expressions>
    /// ORDER BY <expressions> LIMIT <expression> OFFSET <expression>;
    /// ```
    fn parse_select(&mut self) -> Result<ast::Statement> {
        Ok(ast::Statement::Select {
            select: self.parse_select_clause()?,
            from: todo!(),
            r#where: todo!(),
            group_by: todo!(),
            having: todo!(),
            order_by: todo!(),
            limit: todo!(),
            offset: todo!(),
        })
    }

    fn parse_insert(&mut self) -> Result<ast::Statement> {
        todo!("Implement INSERT parsing")
    }

    fn parse_update(&mut self) -> Result<ast::Statement> {
        todo!("Implement UPDATE parsing")
    }

    fn parse_delete(&mut self) -> Result<ast::Statement> {
        todo!("Implement DELETE parsing")
    }

    fn parse_select_clause(&mut self) -> Result<Vec<ast::AliasedExpression>> {
        let mut select = vec![];
        if !self.lexer.next_is(Keyword::Select.into()) {
            return Ok(select);
        }

        loop {
            let expression = self.parse_expression()?;
            let mut alias = None;

            if self.lexer.next_is(Keyword::As.into()) {
                alias = self.lexer.next_ident()?;
            }

            select.push(ast::AliasedExpression(expression, alias));
        }

        todo!()
    }

    fn parse_expression(&mut self) -> Result<ast::Expression> {
        todo!("Implement expression parsing")
    }
}

/// TokenPeekableExt is an extension trait that extends the Peekable iterator over Tokens an
/// optional `peek_transposed` method. This method allows to peek at the next
/// element in the iterator without consuming it, and returns an `Result<Option<T>>`
trait TokenPeekableExt {
    fn peek_transposed(&mut self) -> crate::error::Result<Option<&Token>>;
    fn expect_next(&mut self, expected: Token) -> crate::error::Result<()>;
    fn next_is(&mut self, expected: Token) -> bool;
    fn next_ident(&mut self) -> crate::error::Result<Option<String>>;
}

impl<'a, I: Iterator<Item = crate::error::Result<Token>>> TokenPeekableExt for Peekable<I> {
    fn peek_transposed(&mut self) -> crate::error::Result<Option<&Token>> {
        self.peek()
            .map(|r| r.as_ref().map_err(|err| err.clone()))
            .transpose()
    }

    fn expect_next(&mut self, expected: Token) -> crate::error::Result<()> {
        let token = self.next().transpose()?;
        match token {
            Some(token) if token == expected => Ok(()),
            Some(token) => syntax_error!(0, "Unexpected token: {token}"),
            None => syntax_error!(0, "Unexpected end of input"),
        }
    }

    fn next_is(&mut self, expected: Token) -> bool {
        self.next_if_eq(&Ok(expected)).is_some()
    }

    fn next_ident(&mut self) -> crate::error::Result<Option<String>> {
        let token = self.next().transpose()?;
        match token {
            Some(Token::Ident(ident)) => Ok(Some(ident)),
            Some(token) => syntax_error!(0, "Unexpected token: {token}, expect an identifier"),
            None => Ok(None),
        }
    }
}
