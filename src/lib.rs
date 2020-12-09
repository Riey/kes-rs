use lalrpop_util::lalrpop_mod;

/// kes implementation in Rust
///
/// ## Examples
/// ```rust
/// use kes::builtin::RecordBuiltin;
/// use kes::context::Context;
/// use kes::program::Program;
/// use futures_executor::block_on;
///
/// let source = "$1 = 1 + 2; @$1;";
///
/// let program = Program::from_source(source).unwrap();
/// let mut builtin = RecordBuiltin::new();
/// let mut ctx = Context::new(&program);
/// block_on(ctx.run(&mut builtin)).unwrap();
///
/// assert_eq!(builtin.text(), "3");
/// ```

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
