use crate::token::{BooleanOperatorToken, OperatorToken, Token};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instruction<'s> {
    Nop,
    PushInt(u32),
    PushStr(&'s str),
    PushVar(&'s str),
    CallBuiltin(&'s str),
    Print,
    PrintL,
    PrintW,
    Operator(OperatorToken),
    Goto(usize),
    GotoIfNot(usize),
    EndBlock,
    Duplicate,
}

#[derive(Clone, Copy)]
enum State {
    Empty,
    If(usize),
    Else(usize),
    ElseIf(usize, usize),
    Select,
    SelectElse(usize),
    SelectArm(usize, usize),
}

struct Parser<'s, I: Iterator<Item = Token<'s>>> {
    tokens: I,
    state: State,
    stack: Vec<State>,
    ret: Vec<Instruction<'s>>,
}

impl<'s, I: Iterator<Item = Token<'s>>> Parser<'s, I> {
    fn new(tokens: I) -> Self {
        Self {
            tokens,
            state: State::Empty,
            stack: Vec::with_capacity(20),
            ret: Vec::with_capacity(1000),
        }
    }

    #[inline(always)]
    fn push(&mut self, instruction: Instruction<'s>) {
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

    fn move_next_open_brace(&mut self) {
        loop {
            match self.tokens.next().unwrap() {
                Token::OpenBrace => {
                    break;
                }
                token => self.step(token),
            }
        }
    }

    fn read_select_arm(&mut self, prev_end: usize) {
        self.backup_state();
        match self.tokens.next().unwrap() {
            token @ Token::IntLit(_) | token @ Token::StrLit(_) => {
                self.push(Instruction::Duplicate);
                self.step(token);
                self.push(Instruction::Operator(OperatorToken::Boolean(
                    BooleanOperatorToken::Equal,
                )));
                self.set_state(State::SelectArm(prev_end, self.ret.len()));
                self.push(Instruction::Nop);
            }
            Token::Else => {
                self.set_state(State::SelectElse(self.ret.len() - 1));
            }
            token => panic!("Unexpected token {:?}", token),
        }

        assert_eq!(self.tokens.next(), Some(Token::OpenBrace));
    }

    fn step(&mut self, token: Token<'s>) {
        match token {
            Token::IntLit(num) => self.push(Instruction::PushInt(num)),
            Token::StrLit(text) => self.push(Instruction::PushStr(text)),
            Token::Variable(ident) => self.push(Instruction::PushVar(ident)),
            Token::Operator(op) => self.push(Instruction::Operator(op)),
            Token::Colon => self.push(Instruction::Print),
            Token::At => self.push(Instruction::PrintL),
            Token::Sharp => self.push(Instruction::PrintW),
            Token::Select => {
                self.backup_state();
                self.set_state(State::Select);
                self.move_next_open_brace();
                self.read_select_arm(0);
            }
            Token::OpenBrace => {
                let pos = self.next_pos();

                match self.state {
                    State::Select => unreachable!(),
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
                    State::Empty => panic!("Unexpected block close, {:?}", self.ret),
                    State::ElseIf(if_end, top) => {
                        self.ret[if_end] = Instruction::Goto(pos + 1);
                        self.set_state(State::If(top));
                        self.step(Token::CloseBrace);
                    }
                    State::Select => {
                        self.restore_state();
                    }
                    State::SelectArm(prev_end, top) => {
                        self.restore_state();
                        self.ret[top] = Instruction::GotoIfNot(pos + 1);
                        self.push(Instruction::Nop);
                        if prev_end != 0 {
                            self.ret[prev_end] = Instruction::Goto(pos);
                        }
                        self.read_select_arm(pos);
                    }
                    State::SelectElse(top) => {
                        self.restore_state();
                        self.ret[top] = Instruction::Goto(pos);
                    }
                    State::If(top) => {
                        match self.tokens.next() {
                            Some(Token::Else) => {
                                match self.tokens.next().unwrap() {
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
                                        self.step(token);
                                        self.move_next_open_brace();
                                        self.set_state(State::ElseIf(pos, self.ret.len()));
                                        self.push(Instruction::Nop);
                                    }
                                }
                            }
                            // end if
                            Some(token) => {
                                self.ret[top] = Instruction::GotoIfNot(pos);
                                self.restore_state();
                                self.step(token);
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
                    _ => todo!(),
                };
            }
            _ => todo!(),
        }
    }

    fn parse(mut self) -> Vec<Instruction<'s>> {
        while let Some(token) = self.tokens.next() {
            self.step(token);
        }

        self.ret
    }
}

pub fn parse<'s, I: Iterator<Item = Token<'s>>>(tokens: I) -> Vec<Instruction<'s>> {
    Parser::new(tokens).parse()
}

#[test]
fn parse_if_test() {
    use crate::lexer::lex;
    use crate::token::{BooleanOperatorToken, SimpleOperatorToken};
    use pretty_assertions::assert_eq;

    let instructions = parse(lex("
1 2 < {
    '1은 2보다 작다'@
}
'3 + 4 = ' 3 4 + @
"));

    assert_eq!(
        &instructions,
        &[
            Instruction::PushInt(1),
            Instruction::PushInt(2),
            Instruction::Operator(OperatorToken::Boolean(BooleanOperatorToken::Less)),
            Instruction::GotoIfNot(6),
            Instruction::PushStr("1은 2보다 작다"),
            Instruction::PrintL,
            Instruction::PushStr("3 + 4 = "),
            Instruction::PushInt(3),
            Instruction::PushInt(4),
            Instruction::Operator(OperatorToken::Simple(SimpleOperatorToken::Add)),
            Instruction::PrintL,
        ]
    );
}

#[test]
fn parse_if_else_test() {
    use crate::lexer::lex;
    use crate::token::{BooleanOperatorToken, OperatorToken};
    use pretty_assertions::assert_eq;

    let instructions = parse(lex("
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
"));

    assert_eq!(
        &instructions,
        &[
            Instruction::PushInt(1),
            Instruction::PushInt(2),
            Instruction::Operator(OperatorToken::Boolean(BooleanOperatorToken::Less)),
            Instruction::GotoIfNot(7),
            Instruction::PushStr("1은 2보다 작다"),
            Instruction::PrintL,
            Instruction::Goto(14),
            Instruction::PushInt(2),
            Instruction::PushInt(2),
            Instruction::Operator(OperatorToken::Boolean(BooleanOperatorToken::Equal)),
            Instruction::GotoIfNot(14),
            Instruction::PushStr("2와 2는 같다"),
            Instruction::PrintL,
            Instruction::Goto(21),
            Instruction::PushInt(1),
            Instruction::PushInt(2),
            Instruction::Operator(OperatorToken::Boolean(BooleanOperatorToken::Greater)),
            Instruction::GotoIfNot(21),
            Instruction::PushStr("1은 2보다 크다"),
            Instruction::PrintL,
            Instruction::Goto(23),
            Instruction::PushStr("1은 2와 같다"),
            Instruction::PrintL,
            Instruction::PushStr("foo"),
            Instruction::PrintL,
        ]
    );
}

#[test]
fn parse_select_else() {
    
    use crate::lexer::lex;
    use crate::token::{BooleanOperatorToken, SimpleOperatorToken};
    use pretty_assertions::assert_eq;

    let instructions = parse(lex("
선택 1 {
    그외 {
        ''@
    }
}
''@
"));

    assert_eq!(instructions, &[
        Instruction::Goto(3),
        Instruction::PushInt(1),
    ]);


}

#[test]
fn parse_select() {
    use crate::lexer::lex;
    use crate::token::{BooleanOperatorToken, SimpleOperatorToken};
    use pretty_assertions::assert_eq;

    let instructions = parse(lex("
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
"));

    assert_eq!(
        &instructions,
        &[
            Instruction::PushInt(1),
            Instruction::PushInt(2),
            Instruction::Operator(OperatorToken::Simple(SimpleOperatorToken::Add)),
            Instruction::Duplicate,
            Instruction::PushInt(3),
            Instruction::Operator(OperatorToken::Boolean(BooleanOperatorToken::Equal)),
            Instruction::GotoIfNot(10),
            Instruction::PushStr("3"),
            Instruction::PrintL,
            Instruction::Goto(16),
            Instruction::Duplicate,
            Instruction::PushInt(2),
            Instruction::Operator(OperatorToken::Boolean(BooleanOperatorToken::Equal)),
            Instruction::GotoIfNot(17),
            Instruction::PushStr("2"),
            Instruction::PrintL,
            Instruction::Goto(23),
            Instruction::Duplicate,
            Instruction::PushInt(1),
            Instruction::Operator(OperatorToken::Boolean(BooleanOperatorToken::Equal)),
            Instruction::GotoIfNot(24),
            Instruction::PushStr("1"),
            Instruction::PrintL,
            Instruction::Goto(26),
            Instruction::PushStr("other"),
            Instruction::PrintL,
            Instruction::PushStr("foo"),
            Instruction::PrintL,
        ]
    );
}
