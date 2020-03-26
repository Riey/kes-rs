use std::fmt::Debug;
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub type ParserResult<T> = Result<T, ParserError>;
