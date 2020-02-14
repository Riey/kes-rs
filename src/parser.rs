use crate::instruction::Instruction;
use crate::lexer::Lexer;
use crate::operator::Operator;
use crate::token::Token;
use bumpalo::collections::Vec;
use bumpalo::Bump;
use std::vec::Vec as StdVec;

use crate::error::{ParserResult as Result, ParserError};

#[derive(Clone, Copy)]
enum State {
    Empty,
    If(usize),
    Else(usize),
    ElseIf(usize, usize),
    Loop(usize, usize),
    Select,
    SelectElse(usize),
    SelectArm(usize, usize),
}

struct Parser<'s, 'b> {
    bump: &'b Bump,
    lexer: Lexer<'s>,
    state: State,
    stack: StdVec<State>,
    ret: Vec<'b, Instruction<'b>>,
}

impl<'s, 'b> Parser<'s, 'b> {
    fn new(bump: &'b Bump, source: &'s str) -> Self {
        Self {
            bump,
            lexer: Lexer::new(source),
            state: State::Empty,
            stack: StdVec::with_capacity(20),
            ret: Vec::with_capacity_in(1000, bump),
        }
    }

    #[inline(always)]
    fn push(&mut self, instruction: Instruction<'b>) {
        self.ret.push(instruction);
    }

    #[inline(always)]
    fn backup_state(&mut self) {
        self.stack.push(self.state);
    }

    #[inline(always)]
    fn restore_state(&mut self) {
        self.state = self.stack.pop().unwrap();
    }

    #[inline(always)]
    fn set_state(&mut self, state: State) {
        self.state = state;
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
        self.lexer.next().ok_or(ParserError::UnexpectedEndOfToken)?
    }

    #[inline(always)]
    fn intern(&self, s: &str) -> &'b str {
        self.bump.alloc_str(s)
    }

    fn move_next_open_brace(&mut self) -> Result<()> {
        loop {
            match self.expect_next_token()? {
                Token::OpenBrace => {
                    break Ok(());
                }
                token => self.step(token)?,
            }
        }
    }

    fn make_unexpected_token_err(&self, tok: Token) -> ParserError {
        ParserError::UnexpectedToken(format!("{:?}", tok), self.lexer.line())
    }

    fn expect_next_open_brace(&mut self) -> Result<()> {
        match self.expect_next_token()? {
            Token::OpenBrace => Ok(()),
            token => Err(ParserError::UnexpectedToken(format!("{:?}가 아니라 {{가 와야합니다", token), self.lexer.line())),
        }
    }

    fn read_select_arm(&mut self, prev_end: usize) -> Result<()> {
        match self.expect_next_token()? {
            token @ Token::IntLit(_) | token @ Token::StrLit(_) => {
                self.backup_state();
                self.push(Instruction::Duplicate);
                self.step(token)?;
                self.push(Instruction::Operator(Operator::Equal));
                self.set_state(State::SelectArm(prev_end, self.next_pos()));
                self.push(Instruction::Nop);
                self.expect_next_open_brace()?;
            }
            Token::Else => {
                self.backup_state();
                self.set_state(State::SelectElse(prev_end));
                self.expect_next_open_brace()?;
            }
            Token::CloseBrace => {
                // Remove duplicate
                if let State::SelectArm(_, top) = self.state {
                    self.ret[top - 3] = Instruction::Nop;
                }
                // Select ended without else
                self.step(Token::CloseBrace)?;
            }
            token => {
                return Err(self.make_unexpected_token_err(token));
            }
        }

        Ok(())
    }

    fn step(&mut self, token: Token<'s>) -> Result<()> {
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
            Token::Else => unreachable!(),
            Token::Operator(op) => self.push(Instruction::Operator(op)),
            Token::At => self.push(Instruction::NewLine),
            Token::Sharp => self.push(Instruction::Wait),
            Token::Loop => {
                self.backup_state();
                let start = self.next_pos();
                self.move_next_open_brace()?;
                let end = self.next_pos();
                self.push(Instruction::Nop);
                self.set_state(State::Loop(start, end));
            }
            Token::Select => {
                self.backup_state();
                self.set_state(State::Select);
                self.move_next_open_brace()?;
                self.push(Instruction::MarkStack);
                self.read_select_arm(0)?;
            }
            Token::OpenBrace => {
                let pos = self.next_pos();

                match self.state {
                    State::Select | State::Loop(..) => {
                        return Err(self.make_unexpected_token_err(Token::OpenBrace));
                    }
                    _ => {
                        self.backup_state();
                        self.push(Instruction::Nop);
                        self.set_state(State::If(pos));
                    }
                };
            }
            Token::CloseBrace => {
                let pos = self.next_pos();
                match self.state {
                    State::Empty => {
                        return Err(self.make_unexpected_token_err(Token::CloseBrace));
                    }
                    State::ElseIf(if_end, top) => {
                        self.ret[if_end] = Instruction::Goto(pos + 1);
                        self.set_state(State::If(top));
                        self.step(Token::CloseBrace)?;
                    }
                    State::Loop(start, end) => {
                        self.ret[end] = Instruction::GotoIfNot(pos + 1);
                        self.push(Instruction::Goto(start));
                        self.restore_state();
                    }
                    State::Select => {
                        self.push(Instruction::RemoveMarked);
                        self.restore_state();
                    }
                    State::SelectArm(prev_end, top) => {
                        self.restore_state();
                        self.ret[top] = Instruction::GotoIfNot(pos + 1);
                        self.push(Instruction::Nop);
                        if prev_end != 0 {
                            self.ret[prev_end] = Instruction::Goto(pos);
                        }
                        self.read_select_arm(pos)?;
                    }
                    State::SelectElse(top) => {
                        self.restore_state();
                        if top != 0 {
                            self.ret[top] = Instruction::Goto(pos);
                        }
                    }
                    State::If(top) => {
                        match self.next_token()? {
                            Some(Token::Else) => {
                                match self.expect_next_token()? {
                                    // Else
                                    Token::OpenBrace => {
                                        self.ret[top] = Instruction::GotoIfNot(pos + 1);
                                        self.push(Instruction::Nop);
                                        self.set_state(State::Else(pos));
                                    }
                                    // Else if
                                    token => {
                                        self.ret[top] = Instruction::GotoIfNot(pos + 1);
                                        self.push(Instruction::Nop);
                                        self.step(token)?;
                                        self.move_next_open_brace()?;
                                        self.set_state(State::ElseIf(pos, self.ret.len()));
                                        self.push(Instruction::Nop);
                                    }
                                }
                            }
                            // end if
                            Some(token) => {
                                self.ret[top] = Instruction::GotoIfNot(pos);
                                self.restore_state();
                                self.step(token)?;
                            }
                            None => {
                                self.ret[top] = Instruction::GotoIfNot(pos);
                                self.restore_state();
                                self.push(Instruction::Nop);
                            }
                        }
                    }
                    State::Else(if_end) => {
                        self.restore_state();
                        self.ret[if_end] = Instruction::Goto(pos);
                    }
                };
            }
        }

        Ok(())
    }

    fn parse(mut self) -> Result<Vec<'b, Instruction<'b>>> {
        while let Some(token) = self.next_token()? {
            self.step(token)?;
        }

        Ok(self.ret)
    }
}

pub fn parse<'b>(bump: &'b Bump, source: &str) -> Result<Vec<'b, Instruction<'b>>> {
    Parser::new(bump, source).parse()
}

#[test]
fn parse_condition() {
    use pretty_assertions::assert_eq;

    let bump = Bump::new();

    let instructions = parse(
        &bump,
        "
5 [$0]
$0 2 % '$0은 짝수' '$0은 홀수' [?]
",
    ).unwrap();

    assert_eq!(
        instructions,
        &[
            Instruction::LoadInt(5),
            Instruction::StoreVar("0"),
            Instruction::LoadVar("0"),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Rem),
            Instruction::LoadStr("$0은 짝수"),
            Instruction::LoadStr("$0은 홀수"),
            Instruction::Conditional,
        ]
    );
}

#[test]
fn parse_assign() {
    use pretty_assertions::assert_eq;

    let bump = Bump::new();

    let instructions = parse(
        &bump,
        "
1 2 + [$1]
",
    ).unwrap();

    assert_eq!(
        instructions,
        &[
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Add),
            Instruction::StoreVar("1"),
        ]
    );
}

#[test]
fn parse_if_test() {
    use pretty_assertions::assert_eq;

    let bump = Bump::new();

    let instructions = parse(
        &bump,
        "
1 2 < {
    '1은 2보다 작다'@
}
'3 + 4 = ' 3 4 + @
",
    ).unwrap();

    assert_eq!(
        &instructions,
        &[
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Less),
            Instruction::GotoIfNot(6),
            Instruction::LoadStr("1은 2보다 작다"),
            Instruction::NewLine,
            Instruction::LoadStr("3 + 4 = "),
            Instruction::LoadInt(3),
            Instruction::LoadInt(4),
            Instruction::Operator(Operator::Add),
            Instruction::NewLine,
        ]
    );
}

#[test]
fn parse_if_else_test() {
    use pretty_assertions::assert_eq;

    let bump = Bump::new();

    let instructions = parse(
        &bump,
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
    ).unwrap();

    assert_eq!(
        &instructions,
        &[
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Less),
            Instruction::GotoIfNot(7),
            Instruction::LoadStr("1은 2보다 작다"),
            Instruction::NewLine,
            Instruction::Goto(14),
            Instruction::LoadInt(2),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(14),
            Instruction::LoadStr("2와 2는 같다"),
            Instruction::NewLine,
            Instruction::Goto(21),
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Greater),
            Instruction::GotoIfNot(21),
            Instruction::LoadStr("1은 2보다 크다"),
            Instruction::NewLine,
            Instruction::Goto(23),
            Instruction::LoadStr("1은 2와 같다"),
            Instruction::NewLine,
            Instruction::LoadStr("foo"),
            Instruction::NewLine,
        ]
    );
}

#[test]
fn parse_select_else() {
    use pretty_assertions::assert_eq;

    let bump = Bump::new();

    let instructions = parse(
        &bump,
        "
선택 1 {
    그외 {
        ''@
    }
}
''@
",
    ).unwrap();

    assert_eq!(
        instructions,
        &[
            Instruction::LoadInt(1),
            Instruction::MarkStack,
            Instruction::LoadStr(""),
            Instruction::NewLine,
            Instruction::RemoveMarked,
            Instruction::LoadStr(""),
            Instruction::NewLine,
        ]
    );
}

#[test]
fn parse_select() {
    use pretty_assertions::assert_eq;

    let bump = Bump::new();

    let instructions = parse(
        &bump,
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
    ).unwrap();

    assert_eq!(
        &instructions,
        &[
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Add),
            Instruction::MarkStack,
            Instruction::Duplicate,
            Instruction::LoadInt(3),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(11),
            Instruction::LoadStr("3"),
            Instruction::NewLine,
            Instruction::Goto(17),
            Instruction::Duplicate,
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(18),
            Instruction::LoadStr("2"),
            Instruction::NewLine,
            Instruction::Goto(24),
            Instruction::Duplicate,
            Instruction::LoadInt(1),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(25),
            Instruction::LoadStr("1"),
            Instruction::NewLine,
            Instruction::Goto(27),
            Instruction::LoadStr("other"),
            Instruction::NewLine,
            Instruction::RemoveMarked,
            Instruction::LoadStr("foo"),
            Instruction::NewLine,
        ]
    );
}

#[test]
fn parse_select_without_else() {
    use pretty_assertions::assert_eq;

    let bump = Bump::with_capacity(8196);
    let instructions = parse(
        &bump,
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
    ).unwrap();

    assert_eq!(
        &instructions,
        &[
            Instruction::LoadInt(1),
            Instruction::MarkStack,
            Instruction::Duplicate,
            Instruction::LoadInt(1),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(8),
            Instruction::LoadInt(2),
            Instruction::Goto(13),
            Instruction::Duplicate,
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(14),
            Instruction::LoadInt(3),
            Instruction::Nop,
            Instruction::RemoveMarked,
        ]
    );
}

#[test]
fn parse_loop_test() {
    let bump = Bump::with_capacity(8196);
    let instructions = parse(&bump, "0 [$0] 반복 $0 3 < { $0 1 + [$0] }").unwrap();
    assert_eq!(
        instructions,
        &[
            Instruction::LoadInt(0),
            Instruction::StoreVar("0"),
            Instruction::LoadVar("0"),
            Instruction::LoadInt(3),
            Instruction::Operator(Operator::Less),
            Instruction::GotoIfNot(11),
            Instruction::LoadVar("0"),
            Instruction::LoadInt(1),
            Instruction::Operator(Operator::Add),
            Instruction::StoreVar("0"),
            Instruction::Goto(2),
        ]
    );
}

// Issue #1
#[test]
fn parse_nested_block_with_loop() {
    let bump = Bump::with_capacity(8196);
    let instructions = parse(&bump, "
반복 1 {
    1 2 + 3 == {
        '4'
    } 그외 {
        '5'
    }
}
    ").unwrap();

    assert_eq!(instructions, &[
    ]);
}
