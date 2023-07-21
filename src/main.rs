use rustyline::{error::ReadlineError, DefaultEditor};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

struct Machine {
    stack: Vec<i64>,
    definitions: HashMap<String, Definition>,
}

#[derive(Clone)]
enum Definition {
    Native(fn(&mut Vec<i64>) -> Result<(), ForthError>),
    Tokens(String),
}

impl std::fmt::Debug for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Definition::Native(_) => write!(f, "Native"),
            Definition::Tokens(s) => write!(f, "Tokens({})", s),
        }
    }
}

#[derive(Debug, Clone)]
enum Keyword {
    If,
    Else,
    Then,
    Do,
}

impl TryFrom<&str> for Keyword {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "if" => Ok(Keyword::If),
            "else" => Ok(Keyword::Else),
            "then" => Ok(Keyword::Then),
            "do" => Ok(Keyword::Do),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
enum Token {
    Number(i64),
    Op(Definition),
    Keyword(Keyword),
}

#[derive(Debug, Error)]
enum ForthError {
    #[error("Found an undefined word")]
    WordNotDefined(String),
    #[error("Not enough values on the stack to exec op")]
    StackUnderflow,
    #[error("Attempted to divide by zero")]
    DivByZero,
    #[error("New 'if' keyword before previous conditional completed.")]
    UnbalancedIf,
}

fn add(stack: &mut Vec<i64>) -> Result<(), ForthError> {
    let lhs = stack.pop();
    let rhs = stack.pop();
    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        stack.push(lhs + rhs);
        Ok(())
    } else {
        Err(ForthError::StackUnderflow)
    }
}

fn sub(stack: &mut Vec<i64>) -> Result<(), ForthError> {
    let rhs = stack.pop();
    let lhs = stack.pop();
    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        stack.push(lhs - rhs);
        Ok(())
    } else {
        Err(ForthError::StackUnderflow)
    }
}

fn mul(stack: &mut Vec<i64>) -> Result<(), ForthError> {
    let lhs = stack.pop();
    let rhs = stack.pop();
    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        stack.push(lhs * rhs);
        Ok(())
    } else {
        Err(ForthError::StackUnderflow)
    }
}

fn div(stack: &mut Vec<i64>) -> Result<(), ForthError> {
    let rhs = stack.pop();
    let lhs = stack.pop();
    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        let res = lhs.checked_div(rhs);
        match res {
            Some(n) => stack.push(n),
            None => return Err(ForthError::DivByZero),
        }
        Ok(())
    } else {
        Err(ForthError::StackUnderflow)
    }
}

fn print(stack: &mut Vec<i64>) -> Result<(), ForthError> {
    match stack.last() {
        Some(n) => {
            println!("{}", n);
            Ok(())
        }
        None => Err(ForthError::StackUnderflow),
    }
}

fn dup(stack: &mut Vec<i64>) -> Result<(), ForthError> {
    match stack.last() {
        Some(n) => {
            stack.push(n.clone());
            Ok(())
        }
        None => Err(ForthError::StackUnderflow),
    }
}

impl Machine {
    pub fn new() -> Self {
        let stack = vec![];
        let mut definitions = HashMap::new();
        definitions.insert("+".to_owned(), Definition::Native(add));
        definitions.insert("-".to_owned(), Definition::Native(sub));
        definitions.insert("*".to_owned(), Definition::Native(mul));
        definitions.insert("/".to_owned(), Definition::Native(div));
        definitions.insert("dup".to_owned(), Definition::Native(dup));
        definitions.insert(".".to_owned(), Definition::Native(print));
        definitions.insert(
            "drop".to_owned(),
            Definition::Native(|stack| {
                stack.pop();
                Ok(())
            }),
        );
        definitions.insert(
            "swap".to_owned(),
            Definition::Native(|stack| {
                let rhs = stack.pop();
                let lhs = stack.pop();
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    stack.push(rhs);
                    stack.push(lhs);
                    Ok(())
                } else {
                    Err(ForthError::StackUnderflow)
                }
            }),
        );
        definitions.insert(
            "=".to_owned(),
            Definition::Native(|stack| {
                if let (Some(rhs), Some(lhs)) = (stack.pop(), stack.pop()) {
                    stack.push(if lhs == rhs { 1 } else { 0 });
                    Ok(())
                } else {
                    Err(ForthError::StackUnderflow)
                }
            }),
        );
        definitions.insert(
            "<>".to_owned(),
            Definition::Native(|stack| {
                if let (Some(rhs), Some(lhs)) = (stack.pop(), stack.pop()) {
                    stack.push(if lhs != rhs { 1 } else { 0 });
                    Ok(())
                } else {
                    Err(ForthError::StackUnderflow)
                }
            }),
        );
        definitions.insert(
            "<".to_owned(),
            Definition::Native(|stack| {
                if let (Some(rhs), Some(lhs)) = (stack.pop(), stack.pop()) {
                    stack.push(if lhs < rhs { 1 } else { 0 });
                    Ok(())
                } else {
                    Err(ForthError::StackUnderflow)
                }
            }),
        );
        definitions.insert(
            ">".to_owned(),
            Definition::Native(|stack| {
                if let (Some(rhs), Some(lhs)) = (stack.pop(), stack.pop()) {
                    stack.push(if lhs > rhs { 1 } else { 0 });
                    Ok(())
                } else {
                    Err(ForthError::StackUnderflow)
                }
            }),
        );
        Self { stack, definitions }
    }

    pub fn lex(&mut self, input: &str) -> Result<Vec<Token>, ForthError> {
        enum LexMode {
            Interpreting,
            Defining(VecDeque<String>),
        }
        let mut tokens = vec![];
        let mut mode = LexMode::Interpreting;

        for word in input.split_whitespace() {
            match &mut mode {
                LexMode::Interpreting => {
                    if let Ok(number) = word.parse::<i64>() {
                        tokens.push(Token::Number(number));
                        continue;
                    }
                    if let Some(def) = self.definitions.get(word) {
                        tokens.push(Token::Op(def.clone()));
                        continue;
                    }

                    if let Ok(keyword) = Keyword::try_from(word) {
                        tokens.push(Token::Keyword(keyword));
                        continue;
                    }

                    if word == ":" {
                        mode = LexMode::Defining(VecDeque::new());
                        continue;
                    }

                    return Err(ForthError::WordNotDefined(word.to_owned()));
                }
                LexMode::Defining(current) => {
                    if word == ";" {
                        let name = current.pop_front();
                        let spaced_back = itertools::join(current, " ");
                        let definition = Definition::Tokens(spaced_back);
                        self.definitions.insert(name.unwrap(), definition);
                        eprintln!("Defs: {:?}", &self.definitions);
                        mode = LexMode::Interpreting;
                    } else {
                        current.push_back(word.to_owned());
                    }
                }
            }
        }
        Ok(tokens)
    }

    pub fn exec(&mut self, tokens: Vec<Token>) -> Result<(), ForthError> {
        struct CaptureMode {
            pub tokens: Vec<Token>,
            pub capture: bool,
        }

        impl CaptureMode {
            fn push(&mut self, token: Token) {
                if self.capture {
                    self.tokens.push(token);
                }
            }
        }

        enum ExecMode {
            Normal,
            IfTrue(CaptureMode),
            IfFalse(CaptureMode),
        }

        let mut mode_stack = vec!();
        mode_stack.push(ExecMode::Normal);

        for token in tokens {
            let mut mode = mode_stack.last_mut().unwrap();
            match (mode, token) {
                (ExecMode::Normal, Token::Number(n)) => self.stack.push(n),
                (ExecMode::Normal, Token::Op(def)) => self.run(def)?,
                (ExecMode::Normal, Token::Keyword(kw)) => match kw {
                    Keyword::If => {
                        if let Some(condition) = self.stack.pop() {
                            if condition != 0 {
                                mode_stack.push(ExecMode::IfTrue(CaptureMode {
                                    tokens: vec![],
                                    capture: true,
                                }));
                            } else {
                                mode_stack.push(ExecMode::IfFalse(CaptureMode {
                                    tokens: vec![],
                                    capture: false,
                                }));
                            }
                        } else {
                            return Err(ForthError::StackUnderflow);
                        }
                    }
                    Keyword::Else | Keyword::Then => {
                        todo!(); // This is an error
                    }
                    Keyword::Do => {
                        todo!();
                    }
                },
                (ExecMode::IfTrue(tokens), Token::Number(n)) => tokens.push(Token::Number(n)),
                (ExecMode::IfTrue(tokens), Token::Op(op)) => tokens.push(Token::Op(op)),
                (ExecMode::IfFalse(tokens), Token::Number(n)) => tokens.push(Token::Number(n)),
                (ExecMode::IfFalse(tokens), Token::Op(op)) => tokens.push(Token::Op(op)),
                (ExecMode::IfFalse(tokens), Token::Keyword(kw)) => match kw {
                    Keyword::Else => tokens.capture = true,
                    Keyword::Then => {
                        // Cheeky swap the mode around
                        let old = mode_stack.pop();
                        // Force old into a IfFalse, we know thats what it is
                        if let Some(ExecMode::IfFalse(old)) = old {
                            self.exec(old.tokens)?;
                        } else {
                            unreachable!();
                        }
                    },
                    Keyword::If => {
                        if let Some(condition) = self.stack.pop() {
                            if condition != 0 {
                                mode_stack.push(ExecMode::IfTrue(CaptureMode {
                                    tokens: vec![],
                                    capture: true,
                                }));
                            } else {
                                mode_stack.push(ExecMode::IfFalse(CaptureMode {
                                    tokens: vec![],
                                    capture: false,
                                }));
                            }
                        } else {
                            return Err(ForthError::StackUnderflow);
                        }
                    }
                    Keyword::Do => {
                        todo!();
                    }
                },
                (ExecMode::IfTrue(tokens), Token::Keyword(kw)) => match kw {
                    Keyword::Else => tokens.capture = false,
                    Keyword::Then => {
                        let old = mode_stack.pop();
                        if let Some(ExecMode::IfTrue(old)) = old {
                            self.exec(old.tokens)?;
                        } else {
                            unreachable!();
                        }
                    }
                    Keyword::If => {
                        if let Some(condition) = self.stack.pop() {
                            if condition != 0 {
                                mode_stack.push(ExecMode::IfTrue(CaptureMode {
                                    tokens: vec![],
                                    capture: true,
                                }));
                            } else {
                                mode_stack.push(ExecMode::IfFalse(CaptureMode {
                                    tokens: vec![],
                                    capture: false,
                                }));
                            }
                        } else {
                            return Err(ForthError::StackUnderflow);
                        }
                    },
                    Keyword::Do => {
                        todo!();
                    }
                },
            }
        }

        Ok(())
    }

    pub fn run(&mut self, definition: Definition) -> Result<(), ForthError> {
        match definition {
            Definition::Native(func) => func(&mut self.stack)?,
            Definition::Tokens(toks) => {
                let toks = self.lex(&toks)?;
                self.exec(toks)?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), anyhow::Error> {
    let mut machine = Machine::new();
    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let toks = machine.lex(&line)?;
                machine.exec(toks)?;
                eprintln!("{:?}", machine.stack);
            }
            Err(ReadlineError::Interrupted) => {
                eprintln!("Terminated");
                break;
            }
            Err(ReadlineError::Eof) => {
                eprintln!("All done");
                break;
            }
            Err(err) => {
                eprintln!("Error {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
