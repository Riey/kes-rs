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
        let mut bytes = self.text.as_bytes().iter();
        while let Some(b) = bytes.next() {
            match b {
                b' ' | b'\t' | b'\r' => {}
                b'\n' => {
                    self.line += 1;
                }
                b';' => {
                    let slice = bytes.as_slice();
                    let pos = memchr::memchr(b'\n', slice).unwrap_or(slice.len());
                    bytes = unsafe { slice.get_unchecked(pos..) }.iter();
                }
                _ => {
                    self.text = unsafe {
                        self.text
                            .get_unchecked(self.text.len() - bytes.as_slice().len() - 1..)
                    };
                    return;
                }
            }
        }

        self.text = "";
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
    unsafe fn try_match_pop_byte(&mut self, match_byte: u8) -> bool {
        match self.text.as_bytes().get(0) {
            Some(b) if *b == match_byte => {
                self.text = self.text.get_unchecked(1..);
                true
            }
            _ => false,
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
        if self.try_strip_prefix("만약") {
            Ok(Some(Token::If))
        } else if self.try_strip_prefix("혹은") {
            Ok(Some(Token::ElseIf))
        } else if self.try_strip_prefix("그외") {
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
        } else if self.try_strip_prefix("[!") {
            let pos = memchr::memchr(b']', self.text.as_bytes())
                .ok_or(self.make_code_err("Assign bracket is not paired"))?;
            let num = unsafe { self.text.get_unchecked(..pos) };
            self.text = unsafe { self.text.get_unchecked(pos + 1..) };
            if num.is_empty() {
                Ok(Some(Token::PopExternal(0)))
            } else {
                num.parse()
                    .map_err(|_| self.make_code_err("[!]안에는 숫자가 와야합니다"))
                    .map(|num| Some(Token::PopExternal(num)))
            }
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
        unsafe {
            if self.try_match_pop_byte(b'+') {
                Some(Operator::Add)
            } else if self.try_match_pop_byte(b'-') {
                Some(Operator::Sub)
            } else if self.try_match_pop_byte(b'*') {
                Some(Operator::Mul)
            } else if self.try_match_pop_byte(b'/') {
                Some(Operator::Div)
            } else if self.try_match_pop_byte(b'%') {
                Some(Operator::Rem)
            } else if self.try_match_pop_byte(b'&') {
                Some(Operator::And)
            } else if self.try_match_pop_byte(b'|') {
                Some(Operator::Or)
            } else if self.try_match_pop_byte(b'^') {
                Some(Operator::Xor)
            } else if self.try_match_pop_byte(b'=') {
                Some(Operator::Equal)
            } else if self.try_match_pop_byte(b'>') {
                if self.try_match_pop_byte(b'=') {
                    Some(Operator::GreaterOrEqual)
                } else {
                    Some(Operator::Greater)
                }
            } else if self.try_match_pop_byte(b'<') {
                if self.try_match_pop_byte(b'=') {
                    Some(Operator::LessOrEqual)
                } else {
                    Some(Operator::Less)
                }
            } else if self.try_match_pop_byte(b'~') {
                if self.try_match_pop_byte(b'=') {
                    Some(Operator::NotEqual)
                } else {
                    Some(Operator::Not)
                }
            } else {
                None
            }
        }
    }

    fn read_next(&mut self) -> Result<Token<'s>> {
        if let Ok(Some(token)) = self.try_read_keyword() {
            return Ok(token);
        }

        if let Some(op) = self.try_read_operator() {
            return Ok(Token::Operator(op));
        }

        if let Some(ident) = self.try_read_ident() {
            if let b'0'..=b'9' = ident.as_bytes()[0] {
                return ident.parse().map(Token::IntLit).map_err(|_| {
                    self.make_code_err("변수가 아닌 식별자는 숫자부터 시작할수 없습니다")
                });
            } else if let [b'_'] = ident.as_bytes() {
                return Ok(Token::Underscore);
            } else {
                return Ok(Token::Builtin(ident));
            }
        }

        unsafe {
            if self.try_match_pop_byte(b'\'') {
                self.read_str().map(Token::StrLit)
            } else if self.try_match_pop_byte(b'$') {
                Ok(Token::Variable(self.read_ident()))
            } else if self.try_match_pop_byte(b'{') {
                Ok(Token::OpenBrace)
            } else if self.try_match_pop_byte(b'}') {
                Ok(Token::CloseBrace)
            } else if self.try_match_pop_byte(b'(') {
                Ok(Token::OpenParan)
            } else if self.try_match_pop_byte(b')') {
                Ok(Token::CloseParan)
            } else if self.try_match_pop_byte(b'#') {
                Ok(Token::Sharp)
            } else if self.try_match_pop_byte(b'@') {
                Ok(Token::At)
            } else if self.try_match_pop_byte(b':') {
                Ok(Token::Colon)
            } else {
                Err(self.make_char_err(self.text.chars().next().unwrap()))
            }
        }
    }
}

impl<'s> Iterator for Lexer<'s> {
    type Item = Result<Token<'s>>;

    fn next(&mut self) -> Option<Result<Token<'s>>> {
        self.skip_ws();

        if self.text.is_empty() {
            None
        } else {
            Some(self.read_next())
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

    ts = Lexer::new("_ A 'ABC' @");
    assert_eq!(ts.next().unwrap().unwrap(), Token::Underscore,);
    assert_eq!(ts.next().unwrap().unwrap(), Token::Builtin("A"),);
    assert_eq!(ts.next().unwrap().unwrap(), Token::StrLit("ABC"),);
    assert_eq!(ts.next().unwrap().unwrap(), Token::At,);
    assert!(ts.text.is_empty());

    ts = Lexer::new(";foo\n_ A 'ABC' @");
    assert_eq!(ts.next().unwrap().unwrap(), Token::Underscore,);
    assert_eq!(ts.next().unwrap().unwrap(), Token::Builtin("A"),);
    assert_eq!(ts.next().unwrap().unwrap(), Token::StrLit("ABC"),);
    assert_eq!(ts.next().unwrap().unwrap(), Token::At,);
    assert!(ts.text.is_empty());
}
