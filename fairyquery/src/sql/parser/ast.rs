#[derive(Debug)]
pub struct AliasedExpression(pub Expression, pub Option<String>);

/// Statement is the root of SQL AST. It describes the statement,
/// what type the statement is (SELECT, INSERT, UPDATE, DELETE),
/// with what tables it operates on, and what columns are selected and so on.
/// The statement is built by the parser by parsing the tokens from the lexer.
/// The statement should be syntactically correct, and it is then operator on
/// by the binder, planner, and executor.
#[derive(Debug)]
pub enum Statement {
    Select {
        select: Vec<AliasedExpression>,
        from: Vec<From>,
        r#where: Option<Expression>,
        group_by: Vec<Expression>,
        having: Option<Expression>,
        order_by: Vec<OrderBy>,
        limit: Option<Expression>,
        offset: Option<Expression>,
    },
    Insert {
        table: String,
        columns: Vec<Expression>,
        values: Vec<Vec<Expression>>,
    },
    Update {
        table: String,
        set: Vec<(String, Option<Expression>)>,
        r#where: Option<Expression>,
    },
    Delete {
        table: String,
        r#where: Option<Expression>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum From {
    Table {
        name: String,
        alias: Option<String>,
    },
    Join {
        left: Box<From>,
        right: Box<From>,
        on: Option<Expression>,
        join_type: JoinType,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Debug)]
pub struct OrderBy(pub Expression, pub OrderDirection);

#[derive(Debug)]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    /// For all columns in a table (`*`)
    All,
    /// A column reference with optional table name specified
    /// e. g. `table.column` or `column`
    Column(Option<String>, String),
    /// A literal value
    Literal(Literal),
    /// Function call by name with arguments
    Function(String, Vec<Expression>),
    /// Operator expression `lhs op rhs`
    Operator(Operator),
}

/// Literal value of an expression
#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

/// In order to allow Literals in hashmaps and other useful structures,
/// Eq and Hash traits must be implemented.
impl std::cmp::PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(l), Self::Integer(r)) => l == r,
            (Self::Float(l), Self::Float(r)) => l == r,
            (Self::String(l), Self::String(r)) => l == r,
            (Self::Boolean(l), Self::Boolean(r)) => l == r,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl std::cmp::Eq for Literal {}

impl std::hash::Hash for Literal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Self::Integer(i) => i.hash(state),
            Self::Float(f) => f.to_bits().hash(state),
            Self::String(s) => s.hash(state),
            Self::Boolean(b) => b.hash(state),
            Self::Null => {}
        }
    }
}

/// Expression operators. Represents both unary and binary operators.
///
/// In order to evaluate expressions with correct operator precedence,
/// this structure is a recursive one. Thus each child must be boxed.
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Operator {
    // first Unary operators
    /// Logical negation NOT
    Not(Box<Expression>), // NOT <expr>, e. g. NOT 5

    // Arithmetic operators
    Identity(Box<Expression>),  // +<expr>, e. g. +5
    Negate(Box<Expression>),    // -<expr>, e. g. -5
    Factorial(Box<Expression>), // <expr>!, e. g. 5!

    // then Binary operators
    // Logical operators
    And(Box<Expression>, Box<Expression>), // <expr> AND <expr>, e. g. 5 AND 6
    Or(Box<Expression>, Box<Expression>),  // <expr> OR <expr>, e. g. 5 AND 6

    // Comparison operators
    Equals(Box<Expression>, Box<Expression>), // <expr> = <expr>, e. g. 5 = 6
    NotEquals(Box<Expression>, Box<Expression>), // <expr> != <expr>, e. g. 5 != 6 or 5 <> 6
    LessThan(Box<Expression>, Box<Expression>), // <expr> < <expr>, e. g. 5 < 6
    LessThanOrEquals(Box<Expression>, Box<Expression>), // <expr> <= <expr>, e. g. 5 <= 6
    GreaterThan(Box<Expression>, Box<Expression>), // <expr> > <expr>, e. g. 5 > 6
    GreaterThanOrEquals(Box<Expression>, Box<Expression>), // <expr> >= <expr>, e. g. 5 >= 6
    Is(Box<Expression>, Literal),             // <expr> IS <literal>, e. g. 5 IS NULL
    Like(Box<Expression>, Box<Expression>),   // <expr> LIKE <expr>, e. g. 5 LIKE 6

    // Arithmetic operators
    Add(Box<Expression>, Box<Expression>), // <expr> + <expr>, e. g. 5 + 6
    Subtract(Box<Expression>, Box<Expression>), // <expr> - <expr>, e. g. 5 - 6
    Multiply(Box<Expression>, Box<Expression>), // <expr> * <expr>, e. g. 5 * 6
    Divide(Box<Expression>, Box<Expression>), // <expr> / <expr>, e. g. 5 / 6
    Modulo(Box<Expression>, Box<Expression>), // <expr> % <expr>, e. g. 5 % 6
    Exponentiate(Box<Expression>, Box<Expression>), // <expr> ^ <expr>, e. g. 5 ^ 6
}

impl Expression {}

impl core::convert::From<Literal> for Expression {
    fn from(literal: Literal) -> Self {
        Self::Literal(literal)
    }
}

impl core::convert::From<Operator> for Expression {
    fn from(operator: Operator) -> Self {
        Self::Operator(operator)
    }
}

impl core::convert::From<Operator> for Box<Expression> {
    fn from(operator: Operator) -> Self {
        Box::new(operator.into())
    }
}
