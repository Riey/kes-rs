use lalrpop_util::lalrpop_mod;

pub mod ast;
// pub mod builtin;
pub mod compiler;
// pub mod context;
pub mod error;
lalrpop_mod!(
    #[allow(unused)]
    grammar
);
pub mod instruction;
mod lexer;
pub mod location;
pub mod operator;
pub mod parser;
pub mod token;
pub mod value;

pub use async_trait::async_trait;
