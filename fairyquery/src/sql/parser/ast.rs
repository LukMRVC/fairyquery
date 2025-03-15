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

#[derive(Debug)]
pub struct Expression;

#[derive(Debug)]
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

#[derive(Debug)]
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
