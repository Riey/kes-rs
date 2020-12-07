use lalrpop_util::lalrpop_mod;

mod ast;
pub mod builtin;
mod compiler;
pub mod context;
pub mod error;
lalrpop_mod!(
    #[allow(unused)]
    grammar
);
mod instruction;
mod interner;
mod lexer;
pub mod location;
mod operator;
mod parser;
pub mod program;
mod token;
pub mod value;

pub use async_trait::async_trait;
