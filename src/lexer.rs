use crate::error::{LexicalError, LexicalResult as Result};
use crate::interner::Interner;
use crate::location::Location;
use crate::operator::{BinaryOperator, TernaryOperator, UnaryOperator};
use crate::token::Token;

pub type Spanned = (Location, Token, Location);

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

pub struct Lexer<'s, 'i> {
    text: &'s str,
    interner: &'i mut Interner,
    line: usize,
}

impl<'s, 'i> Lexer<'s, 'i> {
    pub fn new(text: &'s str, interner: &'i mut Interner) -> Self {
        Self {
            text,
            interner,
            line: 1,
        }
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
    fn make_code_err(&self, msg: &'static str) -> LexicalError {
        LexicalError::InvalidCode(msg, self.line())
    }

    #[inline]
    fn make_char_err(&self, ch: char) -> LexicalError {
        LexicalError::InvalidChar(ch, self.line())
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
    fn try_match_pop_byte(&mut self, match_byte: u8) -> bool {
        match self.text.as_bytes().get(0) {
            Some(b) if *b == match_byte => {
                debug_assert!(self.text.is_char_boundary(1));
                self.text = unsafe { self.text.get_unchecked(1..) };
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

    fn try_read_keyword(&mut self) -> Result<Option<Token>> {
        if self.try_strip_prefix("만약") {
            Ok(Some(Token::If))
        } else if self.try_strip_prefix("혹은") {
            Ok(Some(Token::ElseIf))
        } else if self.try_strip_prefix("그외") {
            Ok(Some(Token::Else))
        } else if self.try_strip_prefix("종료") {
            Ok(Some(Token::Exit))
        } else if self.try_strip_prefix("반복") {
            Ok(Some(Token::While))
        } else {
            Ok(None)
        }
    }

    fn try_read_unary_operator(&mut self) -> Option<UnaryOperator> {
        if self.try_match_pop_byte(b'!') {
            Some(UnaryOperator::Not)
        } else {
            None
        }
    }

    fn try_read_binary_operator(&mut self) -> Option<BinaryOperator> {
        if self.try_match_pop_byte(b'+') {
            Some(BinaryOperator::Add)
        } else if self.try_match_pop_byte(b'-') {
            Some(BinaryOperator::Sub)
        } else if self.try_match_pop_byte(b'*') {
            Some(BinaryOperator::Mul)
        } else if self.try_match_pop_byte(b'/') {
            Some(BinaryOperator::Div)
        } else if self.try_match_pop_byte(b'%') {
            Some(BinaryOperator::Rem)
        } else if self.try_match_pop_byte(b'&') {
            Some(BinaryOperator::And)
        } else if self.try_match_pop_byte(b'|') {
            Some(BinaryOperator::Or)
        } else if self.try_match_pop_byte(b'^') {
            Some(BinaryOperator::Xor)
        } else if self.try_match_pop_byte(b'>') {
            if self.try_match_pop_byte(b'=') {
                Some(BinaryOperator::GreaterOrEqual)
            } else {
                Some(BinaryOperator::Greater)
            }
        } else if self.try_match_pop_byte(b'<') {
            if self.try_match_pop_byte(b'=') {
                Some(BinaryOperator::LessOrEqual)
            } else {
                Some(BinaryOperator::Less)
            }
        } else if self.try_strip_prefix("==") {
            Some(BinaryOperator::Equal)
        } else if self.try_strip_prefix("!=") {
            Some(BinaryOperator::NotEqual)
        } else {
            None
        }
    }

    fn try_read_ternary_operator(&mut self) -> Option<(TernaryOperator, bool)> {
        if self.try_match_pop_byte(b'?') {
            Some((TernaryOperator::Conditional, true))
        } else if self.try_match_pop_byte(b':') {
            Some((TernaryOperator::Conditional, false))
        } else {
            None
        }
    }

    fn read_next(&mut self) -> Result<Token> {
        if let Ok(Some(token)) = self.try_read_keyword() {
            return Ok(token);
        }

        if let Some(op) = self.try_read_unary_operator() {
            return Ok(Token::UnaryOp(op));
        }

        if let Some(op) = self.try_read_binary_operator() {
            return Ok(Token::BinaryOp(op));
        }

        if self.try_match_pop_byte(b'=') {
            return Ok(Token::Assign);
        }

        if let Some((op, is_start)) = self.try_read_ternary_operator() {
            return Ok(Token::TernaryOp(op, is_start));
        }

        if let Some(ident) = self.try_read_ident() {
            if let b'0'..=b'9' = ident.as_bytes()[0] {
                return ident.parse().map(Token::IntLit).map_err(|_| {
                    self.make_code_err("변수가 아닌 식별자는 숫자부터 시작할수 없습니다")
                });
            } else {
                return Ok(Token::Builtin(self.interner.get_or_intern(ident)));
            }
        }

        if self.try_match_pop_byte(b'\'') {
            self.read_str()
                .map(|s| Token::StrLit(self.interner.get_or_intern(s)))
        } else if self.try_match_pop_byte(b'$') {
            let ident = self.read_ident();
            Ok(Token::Variable(self.interner.get_or_intern(ident)))
        } else if self.try_match_pop_byte(b'{') {
            Ok(Token::OpenBrace)
        } else if self.try_match_pop_byte(b'}') {
            Ok(Token::CloseBrace)
        } else if self.try_match_pop_byte(b'(') {
            Ok(Token::OpenParan)
        } else if self.try_match_pop_byte(b')') {
            Ok(Token::CloseParan)
        } else if self.try_match_pop_byte(b'@') {
            if self.try_match_pop_byte(b'@') {
                Ok(Token::Print)
            } else if self.try_match_pop_byte(b'!') {
                Ok(Token::PrintWait)
            } else {
                Ok(Token::PrintLine)
            }
        } else if self.try_match_pop_byte(b';') {
            Ok(Token::SemiColon)
        } else if self.try_match_pop_byte(b',') {
            Ok(Token::Comma)
        } else {
            Err(self.make_char_err(self.text.chars().next().unwrap()))
        }
    }
}

impl<'s, 'i> Iterator for Lexer<'s, 'i> {
    type Item = Result<Spanned>;

    fn next(&mut self) -> Option<Result<Spanned>> {
        self.skip_ws();

        if self.text.is_empty() {
            None
        } else {
            let start = Location::new(self.line());
            let token = self.read_next();
            let end = Location::new(self.line());

            let triple = token.map(|token| (start, token, end));

            Some(triple)
        }
    }
}

#[test]
fn lex_test() {
    use pretty_assertions::assert_eq;
    let mut interner = Interner::new();
    let abc = interner.get_or_intern("ABC");
    let a = interner.get_or_intern("A");
    let mut ts = Lexer::new("@'ABC'", &mut interner);

    macro_rules! next {
        () => {
            ts.next().unwrap().unwrap().1
        };
    }

    assert_eq!(next!(), Token::PrintLine,);
    assert_eq!(next!(), Token::StrLit(abc),);
    assert!(ts.text.is_empty());

    ts = Lexer::new("@!  A 'ABC';", &mut interner);
    assert_eq!(next!(), Token::PrintWait,);
    assert_eq!(next!(), Token::Builtin(a),);
    assert_eq!(next!(), Token::StrLit(abc),);
    assert_eq!(next!(), Token::SemiColon,);
    assert!(ts.text.is_empty());

    ts = Lexer::new("@ A 'ABC';", &mut interner);
    assert_eq!(next!(), Token::PrintLine,);
    assert_eq!(next!(), Token::Builtin(a),);
    assert_eq!(next!(), Token::StrLit(abc),);
    assert_eq!(next!(), Token::SemiColon,);

    let one = interner.get_or_intern("1");

    ts = Lexer::new("$1 = 1 + 2;", &mut interner);
    assert_eq!(next!(), Token::Variable(one));
    assert_eq!(next!(), Token::Assign);
    assert_eq!(next!(), Token::IntLit(1));
    assert_eq!(next!(), Token::BinaryOp(BinaryOperator::Add));
    assert_eq!(next!(), Token::IntLit(2));
    assert_eq!(next!(), Token::SemiColon,);
    assert!(ts.text.is_empty());
}
