use crate::instruction::Instruction;
use crate::lexer::Lexer;
use crate::operator::Operator;
use crate::token::Token;
use bumpalo::collections::Vec;
use bumpalo::Bump;

use crate::error::{ParserError as Error, ParserResult as Result};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum State<'s> {
    If,
    ElseIf(usize),
    Else,
    Loop(usize),
    Select,
    Call(&'s str),
    SelectArm(usize),
    SelectElse(usize),
    OpenBrace,
    CloseBrace,
}

#[derive(Clone, Copy)]
enum BlockState<'s> {
    IfBlock(usize),
    ElseIfBlock(usize, usize),
    ElseBlock(usize),
    LoopBlock(usize, usize),
    SelectBlock,
    SelectArmBlock(usize, usize),
    SelectElseBlock(usize),
    CallBlock(&'s str),
}

struct Parser<'b, 's> {
    bump: &'b Bump,
    lexer: Lexer<'s>,
    ret: Vec<'b, Instruction<'s>>,
}

impl<'b, 's> Parser<'b, 's> {
    fn new(bump: &'b Bump, source: &'s str) -> Self {
        let mut ret = Self {
            bump,
            lexer: Lexer::new(source),
            ret: Vec::with_capacity_in(1000, bump),
        };

        ret.push(Instruction::StartBlock);

        ret
    }

    #[inline]
    fn push(&mut self, instruction: Instruction<'s>) {
        self.ret.push(instruction);
    }

    #[inline]
    fn next_pos(&self) -> usize {
        self.ret.len()
    }

    #[inline]
    fn next_token(&mut self) -> Result<Option<Token<'s>>> {
        self.lexer.next().transpose()
    }

    #[inline]
    fn peek_next_is_close_brace(&self) -> bool {
        let mut lexer = self.lexer;
        match lexer.next() {
            Some(Ok(Token::CloseBrace)) => true,
            _ => false,
        }
    }

    #[inline]
    fn try_strip_not_open_brace(&mut self) -> Option<Token<'s>> {
        let mut new_lexer = self.lexer;
        if let Some(Ok(token)) = new_lexer.next() {
            match token {
                Token::OpenBrace => {
                    return None;
                }
                other => {
                    self.lexer = new_lexer;
                    return Some(other);
                }
            }
        } else {
            None
        }
    }

    #[inline]
    fn try_strip_token_else_or_else_if(&mut self) -> Option<State<'s>> {
        let mut new_lexer = self.lexer;
        if let Some(Ok(token)) = new_lexer.next() {
            match token {
                Token::Else => {
                    self.lexer = new_lexer;
                    return Some(State::Else);
                }
                Token::ElseIf => {
                    self.lexer = new_lexer;
                    return Some(State::ElseIf(self.next_pos()));
                }
                _ => {}
            }
        }
        None
    }

    #[inline]
    fn expect_next_token(&mut self) -> Result<Token<'s>> {
        self.lexer.next().ok_or(Error::UnexpectedEndOfToken)?
    }

    fn make_unexpected_token_err(&self, tok: Token) -> Error {
        Error::UnexpectedToken(format!("{:?}", tok), self.lexer.line())
    }

    fn expect_next_builtin(&mut self) -> Result<&'s str> {
        match self.expect_next_token()? {
            Token::Builtin(builtin) => Ok(builtin),
            other => Err(self.make_unexpected_token_err(other)),
        }
    }

    fn process_token(&mut self, token: Token<'s>) -> Result<Option<State<'s>>> {
        match token {
            Token::Conditional => self.push(Instruction::Conditional),
            Token::Duplicate => self.push(Instruction::Duplicate),
            Token::Pop => self.push(Instruction::Pop),
            Token::Exit => self.push(Instruction::Exit),
            Token::IntLit(num) => self.push(Instruction::LoadInt(num)),
            Token::StrLit(text) => self.push(Instruction::LoadStr(text)),
            Token::Variable(ident) => self.push(Instruction::LoadVar(ident)),
            Token::Builtin(ident) => self.push(Instruction::LoadBuiltin(ident)),
            Token::Assign(ident) => self.push(Instruction::StoreVar(ident)),
            Token::Operator(op) => self.push(Instruction::Operator(op)),
            Token::Colon => self.push(Instruction::Print),
            Token::At => self.push(Instruction::PrintLine),
            Token::Sharp => self.push(Instruction::PrintWait),
            Token::If => return Ok(Some(State::If)),
            Token::Else => return Ok(Some(State::Else)),
            Token::OpenBrace => return Ok(Some(State::OpenBrace)),
            Token::CloseBrace => return Ok(Some(State::CloseBrace)),
            Token::Loop => return Ok(Some(State::Loop(self.next_pos()))),
            Token::Select => return Ok(Some(State::Select)),
            Token::Call => return Ok(Some(State::Call(self.expect_next_builtin()?))),
            Token::ElseIf | Token::Underscore => return Err(self.make_unexpected_token_err(token)),
        }

        Ok(None)
    }

    fn read_select_arm(
        &mut self,
        prev_end: usize,
        wait_block_stack: &mut Vec<State<'s>>,
    ) -> Result<()> {
        match self.expect_next_token()? {
            Token::IntLit(num) => {
                self.push(Instruction::Duplicate);
                self.push(Instruction::LoadInt(num));
            }
            Token::StrLit(text) => {
                self.push(Instruction::Duplicate);
                self.push(Instruction::LoadStr(text));
            }
            Token::Underscore => {
                wait_block_stack.push(State::SelectElse(prev_end));
                return Ok(());
            }
            other => return Err(self.make_unexpected_token_err(other)),
        }
        self.push(Instruction::Operator(Operator::Equal));

        loop {
            match self.try_strip_not_open_brace() {
                Some(Token::Operator(Operator::Or)) => {
                    self.push(Instruction::Duplicate);
                    match self.expect_next_token()? {
                        Token::IntLit(num) => {
                            self.push(Instruction::LoadInt(num));
                        }
                        Token::StrLit(text) => {
                            self.push(Instruction::LoadStr(text));
                        }
                        other => return Err(self.make_unexpected_token_err(other)),
                    }
                    self.push(Instruction::Operator(Operator::Equal));
                    self.push(Instruction::Operator(Operator::Or));
                }
                Some(other) => {
                    return Err(self.make_unexpected_token_err(other));
                }
                None => break,
            }
        }

        wait_block_stack.push(State::SelectArm(prev_end));
        Ok(())
    }

    fn parse(mut self) -> Result<Vec<'b, Instruction<'s>>> {
        let mut wait_block_stack = Vec::with_capacity_in(10, self.bump);
        let mut block_stack = Vec::with_capacity_in(50, self.bump);

        while let Some(token) = self.next_token()? {
            match self.process_token(token)? {
                Some(state) => match state {
                    State::OpenBrace => {
                        let wait_block = wait_block_stack.pop().unwrap();

                        match wait_block {
                            State::If => {
                                block_stack.push(BlockState::IfBlock(self.next_pos()));
                                self.push(Instruction::Nop);
                                self.push(Instruction::StartBlock);
                            }
                            State::ElseIf(prev_if) => {
                                block_stack.push(BlockState::ElseIfBlock(prev_if, self.next_pos()));
                                self.push(Instruction::Nop);
                                self.push(Instruction::StartBlock);
                            }
                            State::Else => {
                                block_stack.push(BlockState::ElseBlock(self.next_pos() - 1));
                                self.push(Instruction::StartBlock);
                            }
                            State::Call(builtin) => {
                                block_stack.push(BlockState::CallBlock(builtin));
                                self.push(Instruction::StartBlock);
                            }
                            State::Loop(loop_start) => {
                                block_stack
                                    .push(BlockState::LoopBlock(loop_start, self.next_pos()));
                                self.push(Instruction::Nop);
                                self.push(Instruction::StartBlock);
                            }
                            State::Select => {
                                block_stack.push(BlockState::SelectBlock);
                                if !self.peek_next_is_close_brace() {
                                    self.read_select_arm(0, &mut wait_block_stack)?;
                                }
                            }
                            State::SelectArm(prev_end) => {
                                block_stack
                                    .push(BlockState::SelectArmBlock(prev_end, self.next_pos()));
                                self.push(Instruction::Nop);
                                self.push(Instruction::StartBlock);
                            }
                            State::SelectElse(prev_end) => {
                                block_stack.push(BlockState::SelectElseBlock(prev_end));
                                self.push(Instruction::StartBlock);
                            }
                            State::OpenBrace | State::CloseBrace => unsafe {
                                std::hint::unreachable_unchecked();
                            },
                        }
                    }
                    State::CloseBrace => {
                        let block = block_stack.pop().unwrap();

                        match block {
                            BlockState::IfBlock(top) => {
                                self.push(Instruction::EndBlock);
                                if let Some(state) = self.try_strip_token_else_or_else_if() {
                                    wait_block_stack.push(state);
                                    self.push(Instruction::Nop);
                                }
                                self.ret[top] = Instruction::GotoIfNot(self.next_pos());
                            }
                            BlockState::ElseIfBlock(if_end, top) => {
                                self.push(Instruction::EndBlock);
                                self.ret[if_end] = Instruction::Goto(self.next_pos());
                                if let Some(state) = self.try_strip_token_else_or_else_if() {
                                    wait_block_stack.push(state);
                                    self.push(Instruction::Nop);
                                }
                                self.ret[top] = Instruction::GotoIfNot(self.next_pos());
                            }
                            BlockState::ElseBlock(end) => {
                                self.push(Instruction::EndBlock);
                                self.ret[end] = Instruction::Goto(self.next_pos());
                            }
                            BlockState::CallBlock(builtin) => {
                                self.push(Instruction::CallBuiltin(builtin))
                            }
                            BlockState::LoopBlock(loop_start, cond_end) => {
                                self.push(Instruction::EndBlock);
                                self.push(Instruction::Goto(loop_start));
                                self.ret[cond_end] = Instruction::GotoIfNot(self.next_pos());
                            }
                            BlockState::SelectBlock => {
                                self.push(Instruction::Pop);
                            }
                            BlockState::SelectArmBlock(prev_end, top) => {
                                self.push(Instruction::EndBlock);
                                let pos = self.next_pos();
                                self.push(Instruction::Nop);
                                self.ret[top] = Instruction::GotoIfNot(self.next_pos());
                                if prev_end != 0 {
                                    self.ret[prev_end] = Instruction::Goto(pos);
                                }
                                if !self.peek_next_is_close_brace() {
                                    self.read_select_arm(pos, &mut wait_block_stack)?;
                                } else {
                                    self.ret.pop();
                                }
                            }
                            BlockState::SelectElseBlock(prev_end) => {
                                self.push(Instruction::EndBlock);
                                if prev_end != 0 {
                                    self.ret[prev_end] = Instruction::Goto(self.next_pos());
                                }
                            }
                        }
                    }
                    _ => {
                        wait_block_stack.push(state);
                    }
                },
                _ => continue,
            }
        }

        Ok(self.ret)
    }
}

pub fn parse<'b, 's>(bump: &'b Bump, source: &'s str) -> Result<Vec<'b, Instruction<'s>>> {
    Parser::new(bump, source).parse()
}

#[cfg(test)]
#[cfg_attr(feature = "unstable", track_caller)]
fn parse_test(source: &str, instructions: &[Instruction]) {
    use pretty_assertions::assert_eq;

    let bump = Bump::new();

    assert_eq!(parse(&bump, source).unwrap(), instructions);
}

#[test]
fn parse_builtin_in_if() {
    parse_test(
        "
        만약 테스트 { 1 } 2
        ",
        &[
            Instruction::StartBlock,
            Instruction::LoadBuiltin("테스트"),
            Instruction::GotoIfNot(6),
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::EndBlock,
            Instruction::LoadInt(2),
        ],
    );
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
만약 1 2 < {
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
fn parse_if_else_if_test() {
    parse_test(
        "
만약 1 2 < {
    '1은 2보다 작다'@
} 혹은 2 3 < {
    '2는 3보다 작다'@
}
'3 + 4 = ' 3 4 + @
",
        &[
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Less),
            Instruction::GotoIfNot(10),
            Instruction::StartBlock,
            Instruction::LoadStr("1은 2보다 작다"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Goto(18),
            Instruction::LoadInt(2),
            Instruction::LoadInt(3),
            Instruction::Operator(Operator::Less),
            Instruction::GotoIfNot(18),
            Instruction::StartBlock,
            Instruction::LoadStr("2는 3보다 작다"),
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
만약 1 2 < {
    '1은 2보다 작다'@
} 혹은 2 2 == {
    '2와 2는 같다'@
} 혹은 1 2 > {
    '1은 2보다 크다'@
} 그외 {
    '1은 2와 같다'@
}
'foo'@
",
        &[
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Less),
            Instruction::GotoIfNot(10),
            Instruction::StartBlock,
            Instruction::LoadStr("1은 2보다 작다"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Goto(18),
            Instruction::LoadInt(2),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(19),
            Instruction::StartBlock,
            Instruction::LoadStr("2와 2는 같다"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Goto(27),
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Greater),
            Instruction::GotoIfNot(28),
            Instruction::StartBlock,
            Instruction::LoadStr("1은 2보다 크다"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Goto(32),
            Instruction::StartBlock,
            Instruction::LoadStr("1은 2와 같다"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::LoadStr("foo"),
            Instruction::PrintLine,
        ],
    );
}

#[test]
fn parse_double_if() {
    parse_test(
        "만약 1 ~ { '2'@ } 그외 { '3'@ } 만약 0 { '3'@ } 그외 {  '4'@ }",
        &[
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::Operator(Operator::Not),
            Instruction::GotoIfNot(9),
            Instruction::StartBlock,
            Instruction::LoadStr("2"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Goto(13),
            Instruction::StartBlock,
            Instruction::LoadStr("3"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::LoadInt(0),
            Instruction::GotoIfNot(20),
            Instruction::StartBlock,
            Instruction::LoadStr("3"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Goto(24),
            Instruction::StartBlock,
            Instruction::LoadStr("4"),
            Instruction::PrintLine,
            Instruction::EndBlock,
        ],
    );
}

#[test]
fn parse_select_else() {
    parse_test(
        "
선택 1 {
    _ {
        ''@
    }
}
''@
",
        &[
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::StartBlock,
            Instruction::LoadStr(""),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Pop,
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
    _ {
        'other'@
    }
}
'foo'@
",
        &[
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
            Instruction::EndBlock,
            Instruction::Goto(21),
            Instruction::Duplicate,
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(22),
            Instruction::StartBlock,
            Instruction::LoadStr("2"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Goto(30),
            Instruction::Duplicate,
            Instruction::LoadInt(1),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(31),
            Instruction::StartBlock,
            Instruction::LoadStr("1"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Goto(35),
            Instruction::StartBlock,
            Instruction::LoadStr("other"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Pop,
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
            Instruction::LoadInt(1),
            Instruction::Duplicate,
            Instruction::LoadInt(1),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(10),
            Instruction::StartBlock,
            Instruction::LoadInt(2),
            Instruction::EndBlock,
            Instruction::Goto(17),
            Instruction::Duplicate,
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(18),
            Instruction::StartBlock,
            Instruction::LoadInt(3),
            Instruction::EndBlock,
            Instruction::Pop,
        ],
    );
}

#[test]
fn parse_call_in_if() {
    parse_test(
        "만약 호출 더하기 { 1 2 } { 1 } 그외 { 0 } 2",
        &[
            Instruction::StartBlock,
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::CallBuiltin("더하기"),
            Instruction::GotoIfNot(10),
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::EndBlock,
            Instruction::Goto(13),
            Instruction::StartBlock,
            Instruction::LoadInt(0),
            Instruction::EndBlock,
            Instruction::LoadInt(2),
        ],
    );
}

#[test]
fn parse_call_in_else_if() {
    parse_test(
        "만약 1 { 1 } 혹은 호출 더하기 { 1 2 } { 2 } 그외 { 3 } 4",
        &[
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::GotoIfNot(7),
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::EndBlock,
            Instruction::Goto(15),
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::CallBuiltin("더하기"),
            Instruction::GotoIfNot(16),
            Instruction::StartBlock,
            Instruction::LoadInt(2),
            Instruction::EndBlock,
            Instruction::Goto(19),
            Instruction::StartBlock,
            Instruction::LoadInt(3),
            Instruction::EndBlock,
            Instruction::LoadInt(4),
        ],
    );
}

#[test]
fn parse_call_in_loop() {
    parse_test(
        "반복 호출 더하기 { 1 2 } { 1 } 2",
        &[
            Instruction::StartBlock,
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::CallBuiltin("더하기"),
            Instruction::GotoIfNot(10),
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::EndBlock,
            Instruction::Goto(1),
            Instruction::LoadInt(2),
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
            Instruction::LoadVar("0"),
            Instruction::LoadInt(3),
            Instruction::Operator(Operator::Less),
            Instruction::GotoIfNot(14),
            Instruction::StartBlock,
            Instruction::LoadVar("0"),
            Instruction::LoadInt(1),
            Instruction::Operator(Operator::Add),
            Instruction::StoreVar("0"),
            Instruction::EndBlock,
            Instruction::Goto(3),
        ],
    );
}

#[test]
fn parse_call() {
    parse_test(
        "
호출 더하기 {
    1 2 +
    4
}
5 +
",
        &[
            Instruction::StartBlock,
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Add),
            Instruction::LoadInt(4),
            Instruction::CallBuiltin("더하기"),
            Instruction::LoadInt(5),
            Instruction::Operator(Operator::Add),
        ],
    );
}

// Issue #1
#[test]
fn parse_nested_block_with_loop() {
    parse_test(
        "
반복 0 {
    만약 1 2 + 3 == {
        '4'
    } 그외 {
        '5'
    }
}
    ",
        &[
            Instruction::StartBlock,
            Instruction::LoadInt(0),
            Instruction::GotoIfNot(19),
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Add),
            Instruction::LoadInt(3),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(14),
            Instruction::StartBlock,
            Instruction::LoadStr("4"),
            Instruction::EndBlock,
            Instruction::Goto(17),
            Instruction::StartBlock,
            Instruction::LoadStr("5"),
            Instruction::EndBlock,
            Instruction::EndBlock,
            Instruction::Goto(1),
        ],
    );
}
