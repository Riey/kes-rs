use crate::instruction::Instruction;
use crate::lexer::Lexer;
use crate::operator::Operator;
use crate::token::Token;
use bumpalo::collections::Vec;
use bumpalo::Bump;

use crate::error::{ParserError as Error, ParserResult as Result};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum State {
    If,
    ElseIf,
    Else,
    Loop,
    Select,
    Call,
    Underscore,
    OpenBrace,
    CloseBrace,
}

#[derive(Clone, Copy)]
enum BlockState {
    IfBlock(usize),
    ElseBlock(usize),
    SelectBlock,
    SelectArmBlock(usize, usize),
    SelectElseBlock(usize),
    CallBlock,
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
    fn peek_next_token(&self) -> Result<Option<Token<'s>>> {
        self.lexer.clone().next().transpose()
    }

    #[inline]
    fn expect_next_token(&mut self) -> Result<Token<'s>> {
        self.lexer.next().ok_or(Error::UnexpectedEndOfToken)?
    }

    fn make_unexpected_token_err(&self, tok: Token) -> Error {
        Error::UnexpectedToken(format!("{:?}", tok), self.lexer.line())
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

    fn expect_next_close_brace(&mut self) -> Result<()> {
        match self.expect_next_token()? {
            Token::CloseBrace => Ok(()),
            token => Err(Error::UnexpectedToken(
                format!("{:?}가 아니라 }}가 와야합니다", token),
                self.lexer.line(),
            )),
        }
    }

    fn process_token(&mut self, token: Token<'s>) -> Option<State> {
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
            Token::If => return Some(State::If),
            Token::Else => return Some(State::Else),
            Token::ElseIf => return Some(State::ElseIf),
            Token::Underscore => return Some(State::Underscore),
            Token::OpenBrace => return Some(State::OpenBrace),
            Token::CloseBrace => return Some(State::CloseBrace),
            Token::Loop => return Some(State::Loop),
            Token::Select => return Some(State::Select),
            Token::Call => return Some(State::Call),
        }

        None
    }

    fn parse(mut self) -> Result<Vec<'b, Instruction<'s>>> {
        use std::vec::Vec as StdVec;
        let mut wait_block_stack = StdVec::with_capacity(100);
        let mut block_stack = StdVec::with_capacity(100);

        while let Some(token) = self.next_token()? {
            match self.process_token(token) {
                Some(state) => match state {
                    State::OpenBrace => {
                        let wait_block = wait_block_stack.pop().unwrap();

                        match wait_block {
                            State::If => {
                                block_stack.push(BlockState::IfBlock(self.next_pos()));
                                self.push(Instruction::Nop);
                                self.push(Instruction::StartBlock);
                            }
                            State::OpenBrace | State::CloseBrace => unsafe {
                                std::hint::unreachable_unchecked()
                            },
                            _ => unimplemented!(),
                        }
                    }
                    State::CloseBrace => {
                        let block = block_stack.pop().unwrap();

                        match block {
                            BlockState::IfBlock(top) => {
                                self.push(Instruction::EndBlock);
                                self.ret[top] = Instruction::GotoIfNot(self.next_pos());
                            }
                            _ => unimplemented!(),
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
fn parse_if_else_test() {
    parse_test(
        "
만약 1 2 < {
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
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Less),
            Instruction::GotoIfNot(10),
            Instruction::StartBlock,
            Instruction::LoadStr("1은 2보다 작다"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Goto(32),
            Instruction::LoadInt(2),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(19),
            Instruction::StartBlock,
            Instruction::LoadStr("2와 2는 같다"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Goto(32),
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
    그외 {
        ''@
    }
}
''@
",
        &[
            Instruction::StartBlock,
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::StartBlock,
            Instruction::LoadStr(""),
            Instruction::PrintLine,
            Instruction::EndBlock,
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
            Instruction::GotoIfNot(14),
            Instruction::StartBlock,
            Instruction::LoadStr("3"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Goto(36),
            Instruction::Duplicate,
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(23),
            Instruction::StartBlock,
            Instruction::LoadStr("2"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Goto(36),
            Instruction::Duplicate,
            Instruction::LoadInt(1),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(32),
            Instruction::StartBlock,
            Instruction::LoadStr("1"),
            Instruction::PrintLine,
            Instruction::EndBlock,
            Instruction::Goto(36),
            Instruction::StartBlock,
            Instruction::LoadStr("other"),
            Instruction::PrintLine,
            Instruction::EndBlock,
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
            Instruction::Goto(19),
            Instruction::Duplicate,
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(19),
            Instruction::StartBlock,
            Instruction::LoadInt(3),
            Instruction::EndBlock,
            Instruction::Goto(19),
            Instruction::EndBlock,
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
        "만약 1 { 1 } 그외 호출 더하기 { 1 2 } { 2 } 그외 { 3 } 4",
        &[
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::GotoIfNot(7),
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::EndBlock,
            Instruction::Goto(19),
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
            Instruction::StartBlock,
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::CallBuiltin("더하기"),
            Instruction::GotoIfNot(9),
            Instruction::LoadInt(1),
            Instruction::Goto(2),
            Instruction::EndBlock,
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
            Instruction::StartBlock,
            Instruction::LoadInt(0),
            Instruction::GotoIfNot(18),
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
            Instruction::Goto(2),
            Instruction::EndBlock,
        ],
    );
}
