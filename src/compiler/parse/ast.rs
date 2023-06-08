use std::{fmt::{Debug, Display}};

use arbitrary::Arbitrary;

use crate::common::ui::{Span, Spanned};

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier(pub String);

impl<'a> Arbitrary<'a> for Identifier {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let s: String = u.arbitrary()?;
        if s.chars().next().is_some_and(|c| c.is_alphabetic()) && s.chars().all(|c| c.is_alphanumeric()) {
            Ok(Identifier(s))
        } else {
            Err(arbitrary::Error::IncorrectFormat)
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Self(value)
    }
}

pub type Node<T> = Box<Spanned<T>>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(Arbitrary)]
pub enum BinaryKind {
    Equals,
    NotEquals,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Plus,
    Minus,
    Multiply,
    Divide,
    And,
    Or,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(Arbitrary)]
pub enum UnaryKind {
    Not,
    Neg,
}

#[derive(Debug, PartialEq, Clone)]
#[derive(Arbitrary)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

#[derive(Debug, PartialEq, Clone)]
#[derive(Arbitrary)]
pub struct BinaryExpr {
    pub kind: Spanned<BinaryKind>,
    pub lhs: Node<Expression>,
    pub rhs: Node<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
#[derive(Arbitrary)]
pub struct Call {
    pub callee: Node<Expression>,
    pub args: Vec<Spanned<Expression>>,
}

#[derive(Debug, PartialEq, Clone)]
#[derive(Arbitrary)]
pub enum Expression {
    Assignment {
        id: Spanned<Identifier>,
        rhs: Node<Expression>,
    },
    Binary(BinaryExpr),
    Unary {
        kind: Spanned<UnaryKind>,
        val: Node<Expression>,
    },
    Literal(Spanned<Literal>),
    Identifier(Spanned<Identifier>),
    Call(Call),
}

#[derive(Debug, PartialEq, Clone)]
#[derive(Arbitrary)]
pub struct Statements(pub Vec<Spanned<Statement>>);

#[derive(Arbitrary, Clone)]
pub struct FuzzStatements(Statements);

impl Display for FuzzStatements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for FuzzStatements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // just for fuzzing, to get a cleaner display
        writeln!(f, "{:#?}\n", self.0)?;
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Clone)]
#[derive(Arbitrary)]
pub struct FunctionDeclaration {
    pub name: Spanned<Identifier>,
    pub args: Vec<Spanned<Identifier>>,
    pub body: Spanned<Statements>,
}

#[derive(Debug, PartialEq, Clone)]
#[derive(Arbitrary)]
pub enum Statement {
    Expr(Spanned<Expression>),
    Print(Spanned<Expression>),
    VarDeclaration {
        id: Spanned<Identifier>,
        rhs: Option<Spanned<Expression>>,
    },
    FunctionDeclaration(FunctionDeclaration),
    Block(Spanned<Statements>),
    IfElse {
        cond: Spanned<Expression>,
        then_branch: Spanned<Statements>,
        else_branch: Option<Spanned<Statements>>,
    },
    While {
        cond: Spanned<Expression>,
        body: Spanned<Statements>,
    },
    Return {
        span: Span,
        value: Option<Spanned<Expression>>,
    },
}
