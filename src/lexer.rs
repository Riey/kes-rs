use crate::token::{BooleanOperatorToken, OperatorToken, SimpleOperatorToken, Token};

fn is_ident_char(c: char) -> bool {
    matches!(c, '0'..='9' | 'ㄱ'..='ㅎ' | 'ㅏ'..='ㅣ' | '가'..='힣')
}

fn is_not_ident_char(c: char) -> bool {
    !is_ident_char(c)
}

pub struct Lexer<'s> {
    text: &'s str,
}

impl<'s> Lexer<'s> {
    pub fn text(&self) -> &str {
        self.text
    }

    fn skip_ws(&mut self) {
        loop {
            match self.text.as_bytes().get(0) {
                Some(b' ') | Some(b'\n') => {
                    self.text = self.text.trim_start_matches([' ', '\n'].as_ref());
                }
                Some(b';') => {
                    let pos =
                        memchr::memchr(b'\n', self.text.as_bytes()).unwrap_or(self.text.len());
                    unsafe {
                        self.text = self.text.get_unchecked(pos..);
                    }
                }
                _ => break,
            }
        }
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

    fn read_str(&mut self) -> &'s str {
        let pos = memchr::memchr(b'\'', self.text.as_bytes()).expect("String quote is not paired");
        let lit = unsafe { self.text.get_unchecked(..pos) };
        self.text = unsafe { self.text.get_unchecked(pos + 1..) };
        lit
    }

    #[inline]
    fn try_strip_prefix(&mut self, prefix: &str) -> bool {
        if let Some(left) = self.text.strip_prefix(prefix) {
            self.text = left;
            true
        } else {
            false
        }
    }

    fn try_read_keyword(&mut self) -> Option<Token<'static>> {
        if self.try_strip_prefix("그외") {
            Some(Token::Else)
        } else if self.try_strip_prefix("선택") {
            Some(Token::Select)
        } else {
            None
        }
    }

    fn try_read_boolean_operator(&mut self) -> Option<BooleanOperatorToken> {
        if self.try_strip_prefix("<=") {
            Some(BooleanOperatorToken::LessOrEqual)
        } else if self.try_strip_prefix(">=") {
            Some(BooleanOperatorToken::GreaterOrEqual)
        } else if self.try_strip_prefix("==") {
            Some(BooleanOperatorToken::Equal)
        } else if self.try_strip_prefix("<>") {
            Some(BooleanOperatorToken::NotEqual)
        } else if self.try_match_pop_char('>') {
            Some(BooleanOperatorToken::Greater)
        } else if self.try_match_pop_char('<') {
            Some(BooleanOperatorToken::Less)
        } else if self.try_match_pop_char('~') {
            Some(BooleanOperatorToken::Not)
        } else {
            None
        }
    }

    fn try_read_simple_operator(&mut self) -> Option<SimpleOperatorToken> {
        if self.try_match_pop_char('+') {
            Some(SimpleOperatorToken::Add)
        } else if self.try_match_pop_char('-') {
            Some(SimpleOperatorToken::Sub)
        } else if self.try_match_pop_char('*') {
            Some(SimpleOperatorToken::Mul)
        } else if self.try_match_pop_char('/') {
            Some(SimpleOperatorToken::Div)
        } else if self.try_match_pop_char('%') {
            Some(SimpleOperatorToken::Rem)
        } else if self.try_match_pop_char('&') {
            Some(SimpleOperatorToken::And)
        } else if self.try_match_pop_char('|') {
            Some(SimpleOperatorToken::Or)
        } else if self.try_match_pop_char('^') {
            Some(SimpleOperatorToken::Xor)
        } else {
            None
        }
    }

    fn try_read_operator(&mut self) -> Option<OperatorToken> {
        if let Some(op) = self.try_read_boolean_operator() {
            Some(OperatorToken::Boolean(op))
        } else if let Some(op) = self.try_read_simple_operator() {
            if self.try_match_pop_char('=') {
                Some(OperatorToken::Assign(Some(op)))
            } else {
                Some(OperatorToken::Simple(op))
            }
        } else if self.try_match_pop_char('=') {
            Some(OperatorToken::Assign(None))
        } else {
            None
        }
    }
}

impl<'s> Iterator for Lexer<'s> {
    type Item = Token<'s>;

    fn next(&mut self) -> Option<Token<'s>> {
        self.skip_ws();

        if let token @ Some(_) = self.try_read_keyword() {
            return token;
        }

        if let Some(op) = self.try_read_operator() {
            return Some(Token::Operator(op));
        }

        if let Some(ident) = self.try_read_ident() {
            if let b'0'..=b'9' = ident.as_bytes()[0] {
                return Some(Token::IntLit(ident.parse().unwrap()));
            } else {
                return Some(Token::Builtin(ident));
            }
        }

        match self.pop_char()? {
            '\'' => Some(Token::StrLit(self.read_str())),
            '$' => Some(Token::Variable(self.read_ident())),
            '{' => Some(Token::OpenBrace),
            '}' => Some(Token::CloseBrace),
            '?' => Some(Token::Question),
            '#' => Some(Token::Sharp),
            '@' => Some(Token::At),
            ch => panic!("Unexpected char {}", ch),
        }
    }
}

pub fn lex<'s>(text: &'s str) -> Lexer<'s> {
    Lexer { text }
}

#[test]
fn lex_test() {
    let mut ts = lex("'ABC'#");

    assert_eq!(ts.next().unwrap(), Token::StrLit("ABC"),);
    assert_eq!(ts.next().unwrap(), Token::Sharp,);
    assert!(ts.text().is_empty());

    ts = lex("'ABC' @");
    assert_eq!(ts.next().unwrap(), Token::StrLit("ABC"),);
    assert_eq!(ts.next().unwrap(), Token::At,);
    assert!(ts.text().is_empty());

    ts = lex("+=+");

    assert_eq!(
        ts.next().unwrap(),
        Token::Operator(OperatorToken::Assign(Some(SimpleOperatorToken::Add)))
    );
    assert_eq!(
        ts.next().unwrap(),
        Token::Operator(OperatorToken::Simple(SimpleOperatorToken::Add)),
    );
    assert!(ts.text().is_empty());
}
