use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod builtin;
mod compiler;
pub mod context;
pub mod error;
pub mod formatter;
lalrpop_mod!(
    #[allow(unused)]
    grammar
);
mod instruction;
pub mod interner;
mod lexer;
pub mod location;
mod operator;
pub mod parser;
pub mod program;
mod token;
pub mod value;

pub use async_trait::async_trait;
