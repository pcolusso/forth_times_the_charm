use std::collections::HashMap;
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

enum Token {
    Number(i64),
    Op(Definition),
}

#[derive(Debug, Error)]
enum ForthError {
    #[error("Found an undefined word")]
    WordNotDefined(String),
    #[error("Not enough values on the stack to exec op")]
    StackUnderflow,
    #[error("Attempted to divide by zero")]
    DivByZero
}

fn add(stack: &mut Vec<i64>) -> Result<(), ForthError>  {
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
    let lhs = stack.pop();
    let rhs = stack.pop();
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
    let lhs = stack.pop();
    let rhs = stack.pop();
    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
        let res = lhs.checked_div(rhs);
        match res {
            Some(n) => stack.push(n),
            None => return Err(ForthError::DivByZero)
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
        },
        None => Err(ForthError::StackUnderflow)
    }

}

fn dup(stack: &mut Vec<i64>) -> Result<(), ForthError> {
    match stack.last() {
        Some(n) => {
            stack.push(n.clone());
            Ok(())
        },
        None => Err(ForthError::StackUnderflow)
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

        Self { stack, definitions }
    }

    pub fn lex(&self, input: &str) -> Result<Vec<Token>, ForthError> {
        input
            .split_whitespace()
            .map(|w| {
                if let Ok(number) = w.parse::<i64>() {
                    return Ok(Token::Number(number));
                }

                if let Some(def) = self.definitions.get(w) {
                    return Ok(Token::Op(def.clone()));
                }

                return Err(ForthError::WordNotDefined(w.to_owned()));
            })
            .collect()
    }

    pub fn exec(&mut self, tokens: Vec<Token>) -> Result<(), ForthError> {
        for token in tokens {
            match token {
                Token::Number(n) => self.stack.push(n),
                Token::Op(def) => self.run(def)?,
            }
        }

        Ok(())
    }

    pub fn run(&mut self, definition: Definition) -> Result<(), ForthError> {
        match definition {
            Definition::Native(func) => func(&mut self.stack)?,
            Definition::Tokens(_toks) => {
                unimplemented!();
            }
        }
        Ok(())
    }

}

use rustyline::{DefaultEditor, error::ReadlineError};

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
            },
            Err(ReadlineError::Interrupted) => {
                eprintln!("Terminated");
                break
            },
            Err(ReadlineError::Eof) => {
                eprintln!("All done");
                break
            },
            Err(err) => {
                eprintln!("Error {:?}", err);
                break
            }
        }
    }

    Ok(())
}
