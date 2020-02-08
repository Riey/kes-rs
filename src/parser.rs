use crate::instruction::Instruction;
use crate::operator::Operator;
use crate::token::Token;
use bumpalo::Bump;
use bumpalo::collections::Vec;

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

struct Parser<'s, 'b, I: Iterator<Item = Token<'s>>> {
    bump: &'b Bump,
    tokens: I,
    state: State,
    stack: Vec<'b, State>,
    ret: Vec<'b, Instruction<'b>>,
}

impl<'s, 'b, I: Iterator<Item = Token<'s>>> Parser<'s, 'b, I> {
    fn new(bump: &'b Bump, tokens: I) -> Self {
        Self {
            bump,
            tokens,
            state: State::Empty,
            stack: Vec::with_capacity_in(20, bump),
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

    fn intern(&self, s: &str) -> &'b str {
        self.bump.alloc_str(s)
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
                self.push(Instruction::Operator(Operator::Equal));
                self.set_state(State::SelectArm(prev_end, self.ret.len()));
                self.push(Instruction::Nop);
            }
            Token::Else => {
                self.set_state(State::SelectElse(prev_end));
            }
            token => panic!("Unexpected token {:?}", token),
        }

        assert_eq!(self.tokens.next(), Some(Token::OpenBrace));
    }

    fn step(&mut self, token: Token<'s>) {
        match token {
            Token::IntLit(num) => self.push(Instruction::LoadInt(num)),
            Token::StrLit(text) => self.push(Instruction::LoadStr(self.intern(text))),
            Token::Variable(ident) => self.push(Instruction::LoadVar(self.intern(ident))),
            Token::Builtin(ident) => self.push(Instruction::CallBuiltin(self.intern(ident))),
            Token::Assign => {
                if let Some(Token::Variable(ident)) = self.tokens.next() {
                    self.push(Instruction::StoreVar(self.intern(ident)));
                } else {
                    panic!("Expected variable");
                }
            }
            Token::Else => unreachable!(),
            Token::Operator(op) => self.push(Instruction::Operator(op)),
            Token::At => self.push(Instruction::NewLine),
            Token::Sharp => self.push(Instruction::Wait),
            Token::Question => todo!(),
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
                        self.push(Instruction::Pop);
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
                        if top != 0 {
                            self.ret[top] = Instruction::Goto(pos);
                        }
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
                };
            }
        }
    }

    fn parse(mut self) -> Vec<'b, Instruction<'b>> {
        while let Some(token) = self.tokens.next() {
            self.step(token);
        }

        self.ret
    }
}

pub fn parse<'b, 's, I: Iterator<Item = Token<'s>>>(bump: &'b Bump, tokens: I) -> Vec<'b, Instruction<'b>> {
    Parser::new(bump, tokens).parse()
}

#[test]
fn parse_assign() {
    use crate::lexer::lex;
    use crate::operator::Operator;
    use pretty_assertions::assert_eq;

    let bump = Bump::new();

    let instructions = parse(&bump, lex("
1 2 + -> $1
"));

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
    use crate::lexer::lex;
    use pretty_assertions::assert_eq;

    let bump = Bump::new();

    let instructions = parse(&bump, lex("
1 2 < {
    '1은 2보다 작다'@
}
'3 + 4 = ' 3 4 + @
"));

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
    use crate::lexer::lex;
    use pretty_assertions::assert_eq;

    let bump = Bump::new();

    let instructions = parse(&bump, lex("
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
    use crate::lexer::lex;
    use pretty_assertions::assert_eq;

    let bump = Bump::new();

    let instructions = parse(&bump, lex("
선택 1 {
    그외 {
        ''@
    }
}
''@
"));

    assert_eq!(
        instructions,
        &[
            Instruction::LoadInt(1),
            Instruction::LoadStr(""),
            Instruction::NewLine,
            Instruction::Pop,
            Instruction::LoadStr(""),
            Instruction::NewLine,
        ]
    );
}

#[test]
fn parse_select() {
    use crate::lexer::lex;
    use pretty_assertions::assert_eq;

    let bump = Bump::new();

    let instructions = parse(&bump, lex("
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
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Add),
            Instruction::Duplicate,
            Instruction::LoadInt(3),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(10),
            Instruction::LoadStr("3"),
            Instruction::NewLine,
            Instruction::Goto(16),
            Instruction::Duplicate,
            Instruction::LoadInt(2),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(17),
            Instruction::LoadStr("2"),
            Instruction::NewLine,
            Instruction::Goto(23),
            Instruction::Duplicate,
            Instruction::LoadInt(1),
            Instruction::Operator(Operator::Equal),
            Instruction::GotoIfNot(24),
            Instruction::LoadStr("1"),
            Instruction::NewLine,
            Instruction::Goto(26),
            Instruction::LoadStr("other"),
            Instruction::NewLine,
            Instruction::Pop,
            Instruction::LoadStr("foo"),
            Instruction::NewLine,
        ]
    );
}
