use std::fmt::{self, Debug, Formatter};
use thiserror::Error;

#[derive(Clone, Error)]
pub enum ParserError {
    #[error("코드해석중 {1}번째 줄에서 에러가 발생했습니다 {0}")]
    InvalidCode(&'static str, usize),
    #[error("잘못된 문자 {0}가 {1}번째 줄에서 발견됐습니다")]
    InvalidChar(char, usize),
    #[error("예상치 못한 토큰 {0}가 {1}번째 줄에서 발견됐습니다")]
    UnexpectedToken(String, usize),
    #[error("컴파일중 {0}번째 줄에서 에러가 발생했습니다 {1}")]
    CompileError(String, usize),
    #[error("예상치 못하게 코드가 끝났습니다")]
    UnexpectedEndOfToken,
}

impl Debug for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub type ParserResult<T> = Result<T, ParserError>;

#[derive(Clone, Eq, PartialEq)]
pub enum InterruptState {
    Builtin(String),
    PrintWait,
}

impl fmt::Display for InterruptState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            InterruptState::Builtin(name) => write!(f, "bulitin({})", name),
            InterruptState::PrintWait => f.write_str("#"),
        }
    }
}

#[derive(Clone, Error, Eq, PartialEq)]
pub enum RuntimeError {
    #[error("{1}번째 줄 실행중 에러발생 {0}")]
    ExecutionError(String, usize),
    #[error("{1}번째 줄 실행중 잘못된 `{0}` 타입이 들어왔습니다")]
    TypeError(&'static str, usize),

    /// has interrupted
    #[error("실행중 인터럽트 되었습니다")]
    Interrupted(InterruptState),
}

impl Debug for RuntimeError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;
