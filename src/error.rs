use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum ParserError {
    #[error("코드해석중 `{1}`번째 줄에서 에러가 발생했습니다 `{0}`")]
    InvalidCode(&'static str, usize),
    #[error("잘못된 문자 `{0}`가 `{1}`번째 줄에서 발견됐습니다")]
    InvalidChar(char, usize),
    #[error("예상치 못한 토큰 `{0}`가 `{1}`번째 줄에서 발견됐습니다")]
    UnexpectedToken(String, usize),
    #[error("컴파일중 `{0}`번째 줄에서 에러가 발생했습니다")]
    CompileError(usize),
    #[error("예상치 못하게 코드가 끝났습니다")]
    UnexpectedEndOfToken,
}

pub type ParserResult<T> = Result<T, ParserError>;