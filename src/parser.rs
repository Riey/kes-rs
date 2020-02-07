use crate::token::{OperatorToken, Token};

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
}

#[derive(Clone, Copy)]
enum State {
    Empty,
    If(usize),
    Else(usize),
    ElseIf(usize, usize),
    Select(usize),
    Arm(usize, usize),
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

    fn step(&mut self, token: Token<'s>) {
        match token {
            Token::IntLit(num) => self.push(Instruction::PushInt(num)),
            Token::StrLit(text) => self.push(Instruction::PushStr(text)),
            Token::Variable(ident) => self.push(Instruction::PushVar(ident)),
            Token::Operator(op) => self.push(Instruction::Operator(op)),
            Token::Colon => self.push(Instruction::Print),
            Token::At => self.push(Instruction::PrintL),
            Token::Sharp => self.push(Instruction::PrintW),
            Token::OpenBrace => {
                let pos = self.next_pos();

                match self.state {
                    State::Select(..) => todo!(),
                    State::Arm(..) => todo!(),
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
                    State::Empty => panic!("Unexpected block close"),
                    State::ElseIf(if_end, top) => {
                        self.ret[if_end] = Instruction::Goto(pos + 1);
                        self.set_state(State::If(top));
                        self.step(Token::CloseBrace);
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

                                        loop {
                                            match self.tokens.next().unwrap() {
                                                Token::OpenBrace => {
                                                    self.set_state(State::ElseIf(
                                                        pos,
                                                        self.ret.len(),
                                                    ));
                                                    self.push(Instruction::Nop);
                                                    break;
                                                }
                                                token => self.step(token),
                                            }
                                        }
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
