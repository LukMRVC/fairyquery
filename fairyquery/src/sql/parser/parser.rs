use crate::{error::Result, syntax_error};

use super::{Keyword, Lexer, Token, ast};
use std::iter::Peekable;

/// Operator precedence
type Precedence = u8;

/// Operator associativity
const LEFT_ASSOCIATIVE: Precedence = 1;
const RIGHT_ASSOCIATIVE: Precedence = 0;

trait Operator {
    fn precedence(&self) -> Precedence;
    fn associativity(&self) -> Precedence;
}

trait UnaryOperator: Operator {
    fn build(self, ohs: ast::Expression) -> ast::Expression;
}

trait BinaryOperator: Operator {
    fn build(self, lhs: ast::Expression, rhs: ast::Expression) -> ast::Expression;
}

enum PrefixOperator {
    Not,   // NOT <expr>
    Plus,  // +<expr>
    Minus, // -<expr>
}

impl Operator for PrefixOperator {
    fn precedence(&self) -> Precedence {
        match self {
            Self::Not => 3,
            _ => 10,
        }
    }

    fn associativity(&self) -> Precedence {
        RIGHT_ASSOCIATIVE
    }
}

impl UnaryOperator for PrefixOperator {
    fn build(self, rhs: ast::Expression) -> ast::Expression {
        let rhs = Box::new(rhs);
        match self {
            Self::Not => ast::Operator::Not(rhs).into(),
            Self::Plus => ast::Operator::Identity(rhs).into(),
            Self::Minus => ast::Operator::Negate(rhs).into(),
        }
    }
}

enum InfixOperator {
    Add,                 // <expr> + <expr>
    Subtract,            // <expr> - <expr>
    Multiply,            // <expr> * <expr>
    Divide,              // <expr> / <expr>
    Modulo,              // <expr> % <expr>
    And,                 // <expr> AND <expr>
    Or,                  // <expr> OR <expr>
    Equals,              // <expr> = <expr>
    NotEquals,           // <expr> != <expr>
    LessThan,            // <expr> < <expr>
    LessThanOrEquals,    // <expr> <= <expr>
    GreaterThan,         // <expr> > <expr>
    GreaterThanOrEquals, // <expr> >= <expr>
    Like,                // <expr> LIKE <expr>
    Exponentiate,        // <expr> ^ <expr>
}

impl Operator for InfixOperator {
    fn precedence(&self) -> Precedence {
        match self {
            Self::Or => 1,
            Self::And => 2,
            Self::Equals | Self::NotEquals | Self::Like => 4,
            Self::LessThan
            | Self::LessThanOrEquals
            | Self::GreaterThan
            | Self::GreaterThanOrEquals => 5,
            Self::Add | Self::Subtract => 6,
            Self::Multiply | Self::Divide | Self::Modulo => 7,
            Self::Exponentiate => 8,
        }
    }

    fn associativity(&self) -> Precedence {
        match self {
            Self::Exponentiate => RIGHT_ASSOCIATIVE,
            _ => LEFT_ASSOCIATIVE,
        }
    }
}

impl BinaryOperator for InfixOperator {
    fn build(self, lhs: ast::Expression, rhs: ast::Expression) -> ast::Expression {
        let (lhs, rhs) = (Box::new(lhs), Box::new(rhs));
        match self {
            Self::Add => ast::Operator::Add(lhs, rhs).into(),
            Self::Subtract => ast::Operator::Subtract(lhs, rhs).into(),
            Self::Multiply => ast::Operator::Multiply(lhs, rhs).into(),
            Self::Divide => ast::Operator::Divide(lhs, rhs).into(),
            Self::Modulo => ast::Operator::Modulo(lhs, rhs).into(),
            Self::And => ast::Operator::And(lhs, rhs).into(),
            Self::Or => ast::Operator::Or(lhs, rhs).into(),
            Self::Equals => ast::Operator::Equals(lhs, rhs).into(),
            Self::NotEquals => ast::Operator::NotEquals(lhs, rhs).into(),
            Self::LessThan => ast::Operator::LessThan(lhs, rhs).into(),
            Self::LessThanOrEquals => ast::Operator::LessThanOrEquals(lhs, rhs).into(),
            Self::GreaterThan => ast::Operator::GreaterThan(lhs, rhs).into(),
            Self::GreaterThanOrEquals => ast::Operator::GreaterThanOrEquals(lhs, rhs).into(),
            Self::Like => ast::Operator::Like(lhs, rhs).into(),
            Self::Exponentiate => ast::Operator::Exponentiate(lhs, rhs).into(),
        }
    }
}

enum PostfixOperator {
    Factorial,           // <expr>!
    Is(ast::Literal),    // <expr> IS <literal>
    IsNot(ast::Literal), // <expr> IS NOT <literal>
}

impl Operator for PostfixOperator {
    fn precedence(&self) -> Precedence {
        match self {
            Self::Is(_) | Self::IsNot(_) => 4,
            Self::Factorial => 9,
        }
    }

    fn associativity(&self) -> Precedence {
        match self {
            Self::Is(_) | Self::IsNot(_) => LEFT_ASSOCIATIVE,
            Self::Factorial => RIGHT_ASSOCIATIVE,
        }
    }
}

impl UnaryOperator for PostfixOperator {
    fn build(self, ohs: ast::Expression) -> ast::Expression {
        let lhs = Box::new(ohs);
        match self {
            Self::Factorial => ast::Operator::Factorial(lhs).into(),
            Self::Is(literal) => ast::Operator::Is(lhs, literal).into(),
            Self::IsNot(literal) => {
                ast::Operator::Not(ast::Operator::Is(lhs, literal).into()).into()
            }
        }
    }
}

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
            let expression = self.parse_expression(0)?;
            let mut alias = None;

            if self.lexer.next_is(Keyword::As.into()) {
                alias = Some(self.lexer.next_ident()?);
            }

            select.push(ast::AliasedExpression(expression, alias));
        }

        todo!()
    }

    /// Parses an expression - it must contain at least one atomic element of expression.
    /// The resulting expression is in easy to evaluate format
    fn parse_expression(&mut self, min_precedence: Precedence) -> Result<ast::Expression> {
        // first check for prefix (unary) operator. If there is one, get its rhs.
        // Otherwise, get the lhs of the expression
        let mut lhs = if let Some(prefix) = self.parse_prefix_operator(min_precedence) {
            let at_precedence = prefix.precedence() + prefix.associativity();
            prefix.build(self.parse_expression(at_precedence)?)
        } else {
            self.parse_expression_atom()?
        };
        // apply any postfix operators for left-hand side
        while let Some(postfix) = self.parse_postfix_operator(min_precedence)? {
            lhs = postfix.build(lhs);
        }

        // apply infix operators, parsing right-hand side of the expression
        while let Some(infix) = self.parse_infix_operator(min_precedence) {
            let at_precedence = infix.precedence() + infix.associativity();
            let rhs = self.parse_expression(at_precedence)?;
            lhs = infix.build(lhs, rhs);
        }

        // and finally for some expression, consider also other postfix operators
        // e.g. 1 + NULL IS NULL
        while let Some(postfix) = self.parse_postfix_operator(min_precedence)? {
            lhs = postfix.build(lhs);
        }

        Ok(lhs)
    }

    /// Parses an atomic element of an expression
    /// That can be one of:
    /// - Literal
    /// - Identifier
    /// - Function call
    /// - Parenthesized expression
    fn parse_expression_atom(&mut self) -> Result<ast::Expression> {
        let next = self
            .lexer
            .next()
            .transpose()?
            .ok_or(crate::error::Error::Syntax(
                0,
                format!("unexpected end of input"),
            ))?;

        Ok(match next {
            // All columns `*`
            Token::Asterisk => ast::Expression::All,
            // Literal number
            Token::Number(n) if n.chars().all(|c| c.is_ascii_digit()) => {
                ast::Literal::Integer(n.parse()?).into()
            }
            Token::Number(n) => ast::Literal::Float(n.parse()?).into(),
            Token::String(s) => ast::Literal::String(s).into(),
            Token::Keyword(Keyword::True) => ast::Literal::Boolean(true).into(),
            Token::Keyword(Keyword::False) => ast::Literal::Boolean(false).into(),
            Token::Keyword(Keyword::Null) => ast::Literal::Null.into(),
            Token::Keyword(Keyword::NaN) => ast::Literal::Float(f64::NAN).into(),
            Token::Keyword(Keyword::Infinity) => ast::Literal::Float(f64::INFINITY).into(),
            // function call
            Token::Ident(func_name) if self.lexer.next_is(Token::ParenOpen) => {
                let mut args = Vec::new();
                while !self.lexer.next_is(Token::ParenClose) {
                    if !args.is_empty() {
                        self.lexer.expect_next(Token::Comma)?;
                    }
                    args.push(self.parse_expression(0)?);
                }
                ast::Expression::Function(func_name, args)
            }
            // Column name, can be qualified with table name
            Token::Ident(table_name) if self.lexer.next_is(Token::Period) => {
                ast::Expression::Column(Some(table_name), self.lexer.next_ident()?)
            }
            Token::Ident(col_name) => ast::Expression::Column(None, col_name),
            // Parenthesized expression
            Token::ParenOpen => {
                let expr = self.parse_expression(0)?;
                self.lexer.expect_next(Token::ParenClose)?;
                expr
            }
            token => return syntax_error!(0, "Unexpected token: {token}, expression was expected"),
        })
    }

    fn parse_prefix_operator(&mut self, min_precedence: Precedence) -> Option<PrefixOperator> {
        self.lexer.next_if_map(|token| {
            let op = match token {
                Token::Keyword(Keyword::Not) => PrefixOperator::Not,
                Token::Plus => PrefixOperator::Plus,
                Token::Minus => PrefixOperator::Minus,
                _ => return None,
            };
            Some(op).filter(|op| op.precedence() >= min_precedence)
        })
    }

    /// Parses a postfix operator, if there is any. Minimal precedence is also considered
    fn parse_postfix_operator(
        &mut self,
        min_precedence: Precedence,
    ) -> Result<Option<PostfixOperator>> {
        // Handle IS (NOT) NULL/NAN
        if let Some(Token::Keyword(Keyword::Is)) = self.lexer.peek_transposed()? {
            // cannot consume more tokens unless precedence is satisfied, so
            // IS NULL is assumed
            if PostfixOperator::Is(ast::Literal::Null).precedence() < min_precedence {
                return Ok(None);
            }
            self.lexer.expect_next(Keyword::Is.into())?;
            let is_not = self.lexer.next_is(Keyword::Not.into());
            let value = match self.lexer.next_transposed()? {
                Token::Keyword(Keyword::Null) => ast::Literal::Null,
                Token::Keyword(Keyword::NaN) => ast::Literal::Float(f64::NAN),
                token => return syntax_error!(0, "Unexpected token: {token}"),
            };
            return Ok(Some(match is_not {
                true => PostfixOperator::IsNot(value),
                false => PostfixOperator::Is(value),
            }));
        }

        Ok(self.lexer.next_if_map(|token| {
            let op = match token {
                Token::Exclamation => PostfixOperator::Factorial,
                _ => return None,
            };
            Some(op).filter(|op| op.precedence() >= min_precedence)
        }))
    }

    /// Parses an infix operator, if there is any. Minimal precedence is also considered
    fn parse_infix_operator(&mut self, min_precedence: Precedence) -> Option<InfixOperator> {
        self.lexer.next_if_map(|token| {
            let operator = match token {
                Token::Plus => InfixOperator::Add,
                Token::Minus => InfixOperator::Subtract,
                Token::Asterisk => InfixOperator::Multiply,
                Token::Slash => InfixOperator::Divide,
                Token::Percent => InfixOperator::Modulo,
                Token::Equal => InfixOperator::Equals,
                Token::NotEqual => InfixOperator::NotEquals,
                Token::LessThan => InfixOperator::LessThan,
                Token::LessThanOrEqual => InfixOperator::LessThanOrEquals,
                Token::GreaterThan => InfixOperator::GreaterThan,
                Token::GreaterThanOrEqual => InfixOperator::GreaterThanOrEquals,
                Token::Keyword(Keyword::And) => InfixOperator::And,
                Token::Keyword(Keyword::Or) => InfixOperator::Or,
                Token::Keyword(Keyword::Like) => InfixOperator::Like,
                Token::Caret => InfixOperator::Exponentiate,
                _ => return None,
            };
            Some(operator).filter(|op| op.precedence() >= min_precedence)
        })
    }
}

/// TokenPeekableExt is an extension trait that extends the Peekable iterator over Tokens an
/// optional `peek_transposed` method. This method allows to peek at the next
/// element in the iterator without consuming it, and returns an `Result<Option<T>>`
trait TokenPeekableExt {
    fn peek_transposed(&mut self) -> crate::error::Result<Option<&Token>>;
    fn next_transposed(&mut self) -> crate::error::Result<Token>;
    fn expect_next(&mut self, expected: Token) -> crate::error::Result<()>;
    fn next_is(&mut self, expected: Token) -> bool;
    fn next_ident(&mut self) -> crate::error::Result<String>;
    fn next_if_map<T>(&mut self, f: impl Fn(&Token) -> Option<T>) -> Option<T>;
}

impl<'a, I: Iterator<Item = crate::error::Result<Token>>> TokenPeekableExt for Peekable<I> {
    fn peek_transposed(&mut self) -> crate::error::Result<Option<&Token>> {
        self.peek()
            .map(|r| r.as_ref().map_err(|err| err.clone()))
            .transpose()
    }

    fn next_transposed(&mut self) -> crate::error::Result<Token> {
        self.next().transpose()?.ok_or(crate::error::Error::Syntax(
            0,
            "Unexpected end of input".to_string(),
        ))
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

    fn next_ident(&mut self) -> crate::error::Result<String> {
        let token = self.next().transpose()?;
        match token {
            Some(Token::Ident(ident)) => Ok(ident),
            Some(token) => syntax_error!(0, "Unexpected token: {token}, expect an identifier"),
            None => syntax_error!(0, "Unexpected end of input"),
        }
    }

    fn next_if_map<T>(&mut self, f: impl Fn(&Token) -> Option<T>) -> Option<T> {
        // Peek at the next token and map it if it exists call the mapping closure `f`
        let next_mapped = self.peek_transposed().unwrap_or(None).map(f)?;
        // if successful, consume the token
        if next_mapped.is_some() {
            self.next();
        }
        next_mapped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expression() {
        let mut parser = Parser::new("1 + 2 * 3");
        let expr = parser.parse_expression(0).unwrap();
        assert_eq!(
            expr,
            ast::Expression::Operator(ast::Operator::Add(
                Box::new(ast::Expression::Literal(ast::Literal::Integer(1))),
                Box::new(ast::Expression::Operator(ast::Operator::Multiply(
                    Box::new(ast::Expression::Literal(ast::Literal::Integer(2))),
                    Box::new(ast::Expression::Literal(ast::Literal::Integer(3)))
                )))
            ))
        );
    }
}
