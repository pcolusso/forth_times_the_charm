use std::collections::HashMap;

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

#[derive(Debug)]
enum ForthError {
    WordNotDefined(String),
    StackUnderflow
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

impl Machine {
    pub fn new() -> Self {
        let stack = vec![];
        let mut definitions = HashMap::new();
        definitions.insert("+".to_owned(), Definition::Native(add));
        definitions.insert("-".to_owned(), Definition::Native(sub));

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

    pub fn peek(&self) -> &i64 {
        self.stack.last().unwrap()
    }
}

fn main() -> Result<(), ForthError> {
    let mut machine = Machine::new();

    let toks = machine.lex("10 20 +")?;

    machine.exec(toks)?;

    println!("{:?}", machine.peek());

    Ok(())
}
