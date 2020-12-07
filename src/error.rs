use crate::location::Location;
use crate::token::Token;
use std::fmt::{self, Debug, Formatter};
use thiserror::Error;

pub type ParseError = lalrpop_util::ParseError<Location, Token, LexicalError>;

#[derive(Clone, Error)]
pub enum LexicalError {
    #[error("코드해석중 {1}번째 줄에서 에러가 발생했습니다 `{0}`")]
    InvalidCode(&'static str, usize),
    #[error("잘못된 문자 `{0}`가 {1}번째 줄에서 발견됐습니다")]
    InvalidChar(char, usize),
    #[error("예상치 못한 토큰 `{0}`가 {1}번째 줄에서 발견됐습니다")]
    UnexpectedToken(String, usize),
    #[error("컴파일중 {1}번째 줄에서 에러가 발생했습니다 `{0}`")]
    CompileError(String, usize),
    #[error("예상치 못하게 코드가 끝났습니다")]
    UnexpectedEndOfToken,
}

impl Debug for LexicalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub type LexicalResult<T> = Result<T, LexicalError>;

#[derive(Clone, Error)]
pub enum RuntimeError {
    #[error("{1}번째 줄 실행중 에러발생 {0}")]
    ExecutionError(&'static str, usize),
    #[error("{1}번째 줄 실행중 잘못된 `{0}` 타입이 들어왔습니다")]
    TypeError(&'static str, usize),
}

impl Debug for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;
