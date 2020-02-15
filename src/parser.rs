use crate::instruction::Instruction;
use crate::lexer::Lexer;
use crate::token::Token;
#[cfg(test)]
use crate::operator::Operator;
use bumpalo::collections::Vec;
use bumpalo::Bump;

use crate::error::{ParserError as Error, ParserResult as Result};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum State {
    End,
    OpenBrace,
    CloseBrace,
    Else,
    Loop,
    Select,
}

struct Parser<'s, 'b> {
    bump: &'b Bump,
    lexer: Lexer<'s>,
    ret: Vec<'b, Instruction<'b>>,
}

impl<'s, 'b> Parser<'s, 'b> {
    fn new(bump: &'b Bump, source: &'s str) -> Self {
        let mut ret = Self {
            bump,
            lexer: Lexer::new(source),
            ret: Vec::with_capacity_in(1000, bump),
        };

        ret.push(Instruction::StartBlock);

        ret
    }

    #[inline(always)]
    fn push(&mut self, instruction: Instruction<'b>) {
        self.ret.push(instruction);
    }

    #[inline(always)]
    fn next_pos(&self) -> usize {
        self.ret.len()
    }

    #[inline(always)]
    fn next_token(&mut self) -> Result<Option<Token<'s>>> {
        self.lexer.next().transpose()
    }

    #[inline(always)]
    fn expect_next_token(&mut self) -> Result<Token<'s>> {
        self.lexer.next().ok_or(Error::UnexpectedEndOfToken)?
    }

    #[inline(always)]
    fn intern(&self, s: &str) -> &'b str {
        self.bump.alloc_str(s)
    }

    fn move_next_open_brace(&mut self) -> Result<()> {
        loop {
            let state = self.step()?;

            if state == State::OpenBrace {
                break Ok(());
            } else if state == State::End {
                return Err(Error::UnexpectedEndOfToken);
            }
        }
    }

    fn make_unexpected_token_err(&self, tok: Token) -> Error {
        Error::UnexpectedToken(format!("{:?}", tok), self.lexer.line())
    }

    fn make_unexpected_state_err(&self, state: State) -> Error {
        match state {
            State::End => Error::UnexpectedEndOfToken,
            state => Error::UnexpectedToken(format!("{:?}", state), self.lexer.line()),
        }
    }

    fn expect_next_open_brace(&mut self) -> Result<()> {
        match self.expect_next_token()? {
            Token::OpenBrace => Ok(()),
            token => Err(Error::UnexpectedToken(
                format!("{:?}가 아니라 {{가 와야합니다", token),
                self.lexer.line(),
            )),
        }
    }

    fn process_token(&mut self, token: Token) -> Option<State> {
        match token {
            Token::Conditional => self.push(Instruction::Conditional),
            Token::Duplicate => self.push(Instruction::Duplicate),
            Token::Pop => self.push(Instruction::Pop),
            Token::Exit => self.push(Instruction::Exit),
            Token::IntLit(num) => self.push(Instruction::LoadInt(num)),
            Token::StrLit(text) => self.push(Instruction::LoadStr(self.intern(text))),
            Token::Variable(ident) => self.push(Instruction::LoadVar(self.intern(ident))),
            Token::Builtin(ident) => self.push(Instruction::CallBuiltin(self.intern(ident))),
            Token::Assign(ident) => self.push(Instruction::StoreVar(self.intern(ident))),
            Token::Operator(op) => self.push(Instruction::Operator(op)),
            Token::Colon => self.push(Instruction::Print),
            Token::At => self.push(Instruction::PrintLine),
            Token::Sharp => self.push(Instruction::PrintWait),
            Token::Else => return Some(State::Else),
            Token::OpenBrace => return Some(State::OpenBrace),
            Token::CloseBrace => return Some(State::CloseBrace),
            Token::Loop => return Some(State::Loop),
            Token::Select => return Some(State::Select),
        }

        None
    }

    fn step(&mut self) -> Result<State> {
        while let Some(token) = self.next_token()? {
            match self.process_token(token) {
                Some(state) => return Ok(state),
                _ => continue,
            }
        }
        Ok(State::End)
    }

    fn read_until_open_brace(&mut self) -> Result<()> {
        match self.step()? {
            State::OpenBrace => Ok(()),
            state => Err(self.make_unexpected_state_err(state)),
        }
    }

    fn expect_none_state(&mut self, token: Token<'s>) -> Result<()> {
        match self.process_token(token) {
            Some(state) => Err(self.make_unexpected_token_err(token)),
            None => Ok(()),
        }
    }

    fn process_else_block(&mut self) -> Result<()> {
        self.push(Instruction::StartBlock);

        loop {
            match self.step()? {
                State::CloseBrace => {
                    self.push(Instruction::EndBlock);
                    break Ok(());
                }
                State::OpenBrace => self.process_if_block()?,
                State::Loop => self.process_loop_block()?,
                state => return Err(self.make_unexpected_state_err(state)),
            }
        }
    }

    fn process_if_block(&mut self) -> Result<()> {
        let if_top = self.next_pos();
        // goto endif
        self.push(Instruction::Nop);

        // if block
        self.push(Instruction::StartBlock);

        loop {
            match self.step()? {
                State::CloseBrace => {
                    self.push(Instruction::EndBlock);
                    self.ret[if_top] = Instruction::GotoIfNot(self.next_pos());
                    break Ok(());
                }
                State::Else => {
                    self.ret[if_top] = Instruction::GotoIfNot(self.next_pos());
                    self.push(Instruction::EndBlock);
                    let endif = self.next_pos();
                    self.push(Instruction::Nop);

                    match self.expect_next_token()? {
                        Token::OpenBrace => {
                            self.process_else_block()?;
                        }
                        token => {
                            // else if
                            self.expect_none_state(token)?;
                            self.read_until_open_brace()?;
                            self.process_if_block()?;
                        }
                    }

                    self.ret[endif] = Instruction::Goto(self.next_pos());
                    break Ok(());
                }
                State::OpenBrace => {
                    self.process_if_block()?;
                }
                State::Loop => self.process_loop_block()?,
                state => return Err(self.make_unexpected_state_err(state)),
            }
        }
    }

    fn process_loop_block(&mut self) -> Result<()> {
        self.push(Instruction::StartBlock);

        let loop_top = self.next_pos();
        self.read_until_open_brace()?;
        let loop_jmp = self.next_pos();
        self.push(Instruction::Nop);

        loop {
            match self.step()? {
                State::CloseBrace => {
                    self.push(Instruction::Goto(loop_top));
                    self.ret[loop_jmp] = Instruction::GotoIfNot(self.next_pos());
                    self.push(Instruction::EndBlock);
                    break Ok(());
                }
                State::OpenBrace => self.process_if_block()?,
                State::Loop => self.process_loop_block()?,
                state => break Err(self.make_unexpected_state_err(state)),
            }
        }
    }

    fn process(&mut self) -> Result<()> {
        loop {
            match self.step()? {
                State::OpenBrace => {
                    self.process_if_block()?;
                }
                State::Loop => {
                    self.process_loop_block()?;
                }
                State::End => break Ok(()),
                state => break Err(self.make_unexpected_state_err(state)),
            }
        }
    }

    fn parse(mut self) -> Result<Vec<'b, Instruction<'b>>> {
        self.process()?;

        Ok(self.ret)
    }
}

pub fn parse<'b>(bump: &'b Bump, source: &str) -> Result<Vec<'b, Instruction<'b>>> {
    Parser::new(bump, source).parse()
}

#[cfg(test)]
fn parse_test(source: &str, instructions: &[Instruction]) {
    use pretty_assertions::assert_eq;

    let bump = Bump::new();

    assert_eq!(parse(&bump, source).unwrap(), instructions);
}

#[test]
fn parse_condition() {
    parse_test(
        "
5 [$0]
$0 2 % '$0은 짝수' '$0은 홀수' [?]
",
        &[
            Instruction::StartBlock,
            Instruction::LoadInt(5),
            Instruction::StoreVar("0"),
            Instruction::LoadVar("0"),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Rem),
            Instruction::LoadStr("$0은 짝수"),
            Instruction::LoadStr("$0은 홀수"),
            Instruction::Conditional,
        ],
    );
}

#[test]
fn parse_assign() {
    parse_test(
        "
1 2 + [$1]
",
        &[
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Add),
            Instruction::StoreVar("1"),
        ],
    );
}

#[test]
fn parse_if_test() {
    parse_test(
        "
1 2 < {
    '1은 2보다 작다'@
}
'3 + 4 = ' 3 4 + @
",
        &[
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Less),
            Instruction::GotoIfNot(9),
            Instruction::StartBlock,
            Instruction::LoadStr("1은 2보다 작다"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::LoadStr("3 + 4 = "),
            Instruction::LoadInt(3),
            Instruction::LoadInt(4),
            Instruction::Operator(Operator::Add),
            Instruction::PrintLine,
        ],
    );
}

#[test]
fn parse_if_else_test() {
    parse_test(
        "
1 2 < {
    '1은 2보다 작다'@
} 그외 2 2 == {
    '2와 2는 같다'@
} 그외 1 2 > {
    '1은 2보다 크다'@
} 그외 {
    '1은 2와 같다'@
}
'foo'@
",
        &[
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Less),
            Instruction::GotoIfNot(7),
            Instruction::LoadStr("1은 2보다 작다"),
            Instruction::PrintLine,
            Instruction::Goto(14),
            Instruction::LoadInt(2),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(14),
            Instruction::LoadStr("2와 2는 같다"),
            Instruction::PrintLine,
            Instruction::Goto(21),
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Greater),
            Instruction::GotoIfNot(21),
            Instruction::LoadStr("1은 2보다 크다"),
            Instruction::PrintLine,
            Instruction::Goto(23),
            Instruction::LoadStr("1은 2와 같다"),
            Instruction::PrintLine,
            Instruction::LoadStr("foo"),
            Instruction::PrintLine,
        ],
    );
}

#[test]
fn parse_select_else() {
    parse_test(
        "
선택 1 {
    그외 {
        ''@
    }
}
''@
",
        &[
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::LoadStr(""),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::LoadStr(""),
            Instruction::PrintLine,
        ],
    );
}

#[test]
fn parse_select() {
    parse_test(
        "
선택 1 2 + {
    3 {
        '3'@
    }
    2 {
        '2'@
    }
    1 {
        '1'@
    }
    그외 {
        'other'@
    }
}
'foo'@
",
        &[
            Instruction::StartBlock,
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Add),
            Instruction::Duplicate,
            Instruction::LoadInt(3),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(13),
            Instruction::StartBlock,
            Instruction::LoadStr("3"),
            Instruction::PrintLine,
            Instruction::Goto(21),
            Instruction::Nop,
            Instruction::Duplicate,
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(22),
            Instruction::LoadStr("2"),
            Instruction::PrintLine,
            Instruction::Goto(24),
            Instruction::Duplicate,
            Instruction::LoadInt(1),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(25),
            Instruction::LoadStr("1"),
            Instruction::PrintLine,
            Instruction::Goto(27),
            Instruction::LoadStr("other"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::LoadStr("foo"),
            Instruction::PrintLine,
        ],
    );
}

#[test]
fn parse_select_without_else() {
    parse_test(
        "
선택 1 {
    1 {
        2
    }
    2 {
        3
    }
}
",
        &[
            Instruction::StartBlock,
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::Duplicate,
            Instruction::LoadInt(1),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(11),
            Instruction::StartBlock,
            Instruction::LoadInt(2),
            Instruction::EndBlock,
            Instruction::Goto(18),
            Instruction::Duplicate,
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(18),
            Instruction::StartBlock,
            Instruction::LoadInt(3),
            Instruction::EndBlock,
            Instruction::Nop,
            Instruction::EndBlock,
        ],
    );
}

#[test]
fn parse_loop_test() {
    parse_test(
        "0 [$0] 반복 $0 3 < { $0 1 + [$0] }",
        &[
            Instruction::StartBlock,
            Instruction::LoadInt(0),
            Instruction::StoreVar("0"),
            Instruction::StartBlock,
            Instruction::LoadVar("0"),
            Instruction::LoadInt(3),
            Instruction::Operator(Operator::Less),
            Instruction::GotoIfNot(13),
            Instruction::LoadVar("0"),
            Instruction::LoadInt(1),
            Instruction::Operator(Operator::Add),
            Instruction::StoreVar("0"),
            Instruction::Goto(4),
            Instruction::EndBlock,
        ],
    );
}

// Issue #1
#[test]
fn parse_nested_block_with_loop() {
    parse_test(
        "
반복 0 {
    1 2 + 3 == {
        '4'
    } 그외 {
        '5'
    }
}
    ",
        &[
            Instruction::LoadInt(0),
            Instruction::GotoIfNot(12),
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Add),
            Instruction::LoadInt(3),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(10),
            Instruction::LoadStr("4"),
            Instruction::Goto(11),
            Instruction::LoadStr("5"),
            Instruction::Goto(0),
        ],
    );
}
