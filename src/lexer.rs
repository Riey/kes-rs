use crate::error::{ParserError, ParserResult as Result};
use crate::operator::Operator;
use crate::token::Token;

fn is_ident_char(c: char) -> bool {
    match c {
        '_' | '0'..='9' | 'a'..='z' | 'A'..='Z' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ' | '가'..='힣' => {
            true
        }
        _ => false,
    }
}

fn is_not_ident_char(c: char) -> bool {
    !is_ident_char(c)
}

#[derive(Copy, Clone)]
pub struct Lexer<'s> {
    text: &'s str,
    line: usize,
}

impl<'s> Lexer<'s> {
    pub fn new(text: &'s str) -> Self {
        Self { text, line: 1 }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    fn skip_ws(&mut self) {
        loop {
            match self.text.as_bytes().get(0) {
                Some(b'\n') => {
                    self.text = unsafe { self.text.get_unchecked(1..) };
                    self.line += 1;
                }
                Some(b' ') => {
                    self.text = self.text.trim_start_matches(' ');
                }
                Some(b';') => {
                    let pos =
                        memchr::memchr(b'\n', self.text.as_bytes()).unwrap_or(self.text.len());
                    unsafe {
                        self.text = self.text.get_unchecked(pos..);
                    }
                    self.line += 1;
                }
                _ => break,
            }
        }
    }

    #[inline]
    fn make_code_err(&self, msg: &'static str) -> ParserError {
        ParserError::InvalidCode(msg, self.line())
    }

    #[inline]
    fn make_char_err(&self, ch: char) -> ParserError {
        ParserError::InvalidChar(ch, self.line())
    }

    #[inline]
    fn read_ident(&mut self) -> &'s str {
        let pos = self.text.find(is_not_ident_char).unwrap_or(self.text.len());
        unsafe {
            let ret = self.text.get_unchecked(..pos);
            self.text = self.text.get_unchecked(pos..);
            ret
        }
    }

    #[inline]
    fn try_read_ident(&mut self) -> Option<&'s str> {
        let ident = self.read_ident();

        if ident.is_empty() {
            None
        } else {
            Some(ident)
        }
    }

    #[inline]
    fn pop_char(&mut self) -> Option<char> {
        let ch = self.text.chars().next()?;
        self.text = unsafe { self.text.get_unchecked(ch.len_utf8()..) };
        Some(ch)
    }

    #[inline]
    fn try_match_pop_char(&mut self, match_ch: char) -> bool {
        let ch = match self.text.chars().next() {
            Some(ch) => ch,
            None => return false,
        };
        if ch == match_ch {
            self.text = unsafe { self.text.get_unchecked(ch.len_utf8()..) };
            true
        } else {
            false
        }
    }

    fn read_str(&mut self) -> Result<&'s str> {
        let pos = memchr::memchr(b'\'', self.text.as_bytes())
            .ok_or(self.make_code_err("String quote is not paired"))?;
        let lit = unsafe { self.text.get_unchecked(..pos) };
        self.text = unsafe { self.text.get_unchecked(pos + 1..) };
        Ok(lit)
    }

    #[inline]
    fn try_strip_prefix(&mut self, prefix: &str) -> bool {
        if self.text.starts_with(prefix) {
            self.text = unsafe { self.text.get_unchecked(prefix.len()..) };
            true
        } else {
            false
        }
    }

    fn try_read_keyword(&mut self) -> Result<Option<Token<'s>>> {
        if self.try_strip_prefix("그외") {
            Ok(Some(Token::Else))
        } else if self.try_strip_prefix("선택") {
            Ok(Some(Token::Select))
        } else if self.try_strip_prefix("종료") {
            Ok(Some(Token::Exit))
        } else if self.try_strip_prefix("반복") {
            Ok(Some(Token::Loop))
        } else if self.try_strip_prefix("[?]") {
            Ok(Some(Token::Conditional))
        } else if self.try_strip_prefix("[-]") {
            Ok(Some(Token::Pop))
        } else if self.try_strip_prefix("[+]") {
            Ok(Some(Token::Duplicate))
        } else if self.try_strip_prefix("[$") {
            let pos = memchr::memchr(b']', self.text.as_bytes())
                .ok_or(self.make_code_err("Assign bracket is not paired"))?;
            let name = unsafe { self.text.get_unchecked(..pos) };
            self.text = unsafe { self.text.get_unchecked(pos + 1..) };
            Ok(Some(Token::Assign(name)))
        } else {
            Ok(None)
        }
    }

    fn try_read_operator(&mut self) -> Option<Operator> {
        if self.try_match_pop_char('+') {
            Some(Operator::Add)
        } else if self.try_match_pop_char('-') {
            Some(Operator::Sub)
        } else if self.try_match_pop_char('*') {
            Some(Operator::Mul)
        } else if self.try_match_pop_char('/') {
            Some(Operator::Div)
        } else if self.try_match_pop_char('%') {
            Some(Operator::Rem)
        } else if self.try_match_pop_char('&') {
            Some(Operator::And)
        } else if self.try_match_pop_char('|') {
            Some(Operator::Or)
        } else if self.try_match_pop_char('^') {
            Some(Operator::Xor)
        } else if self.try_strip_prefix("<=") {
            Some(Operator::LessOrEqual)
        } else if self.try_strip_prefix(">=") {
            Some(Operator::GreaterOrEqual)
        } else if self.try_strip_prefix("==") {
            Some(Operator::Equal)
        } else if self.try_strip_prefix("<>") {
            Some(Operator::NotEqual)
        } else if self.try_match_pop_char('>') {
            Some(Operator::Greater)
        } else if self.try_match_pop_char('<') {
            Some(Operator::Less)
        } else if self.try_match_pop_char('~') {
            Some(Operator::Not)
        } else {
            None
        }
    }
}

impl<'s> Iterator for Lexer<'s> {
    type Item = Result<Token<'s>>;

    fn next(&mut self) -> Option<Result<Token<'s>>> {
        self.skip_ws();

        if let Ok(Some(token)) = self.try_read_keyword() {
            return Some(Ok(token));
        }

        if let Some(op) = self.try_read_operator() {
            return Some(Ok(Token::Operator(op)));
        }

        if let Some(ident) = self.try_read_ident() {
            if let b'0'..=b'9' = ident.as_bytes()[0] {
                return Some(ident.parse().map(Token::IntLit).map_err(|_| {
                    self.make_code_err("변수가 아닌 식별자는 숫자부터 시작할수 없습니다")
                }));
            } else {
                return Some(Ok(Token::Builtin(ident)));
            }
        }

        match self.pop_char()? {
            '\'' => Some(self.read_str().map(Token::StrLit)),
            '$' => Some(Ok(Token::Variable(self.read_ident()))),
            '{' => Some(Ok(Token::OpenBrace)),
            '}' => Some(Ok(Token::CloseBrace)),
            '#' => Some(Ok(Token::Sharp)),
            '@' => Some(Ok(Token::At)),
            ':' => Some(Ok(Token::Colon)),
            ch => Some(Err(self.make_char_err(ch))),
        }
    }
}

#[test]
fn lex_test() {
    use pretty_assertions::assert_eq;
    let mut ts = Lexer::new("'ABC'#");

    assert_eq!(ts.next().unwrap().unwrap(), Token::StrLit("ABC"),);
    assert_eq!(ts.next().unwrap().unwrap(), Token::Sharp,);
    assert!(ts.text.is_empty());

    ts = Lexer::new("[?][-][+][$123]");

    assert_eq!(ts.next().unwrap().unwrap(), Token::Conditional,);
    assert_eq!(ts.next().unwrap().unwrap(), Token::Pop,);
    assert_eq!(ts.next().unwrap().unwrap(), Token::Duplicate,);
    assert_eq!(ts.next().unwrap().unwrap(), Token::Assign("123"),);
    assert!(ts.text.is_empty());

    ts = Lexer::new("'ABC' @");
    assert_eq!(ts.next().unwrap().unwrap(), Token::StrLit("ABC"),);
    assert_eq!(ts.next().unwrap().unwrap(), Token::At,);
    assert!(ts.text.is_empty());
}
