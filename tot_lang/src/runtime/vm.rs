use super::frame::Frame;
use crate::program::{Op, Program, ReferenceOrValue};
use serde_json::Value;

/// The virtual machine, which stores all runtime state for one run
#[derive(Default, Debug)]
pub struct Vm {
    frame: Frame,
    register: Option<Value>,
}

impl Vm {
    pub async fn eval(&mut self, code: &str) -> anyhow::Result<()> {
        let program = Program::from_statement(code)?;
        self.execute(&program).await?;
        Ok(())
    }

    /// Execute the ast
    pub async fn execute(&mut self, program: &Program) -> anyhow::Result<()> {
        for op in program.operations() {
            match op {
                Op::Declare { name, .. } => {
                    // store a null value, so the store op will modify the one for current scope
                    self.frame.store(name.to_string(), Value::Null);
                }
                Op::Store { name } => self
                    .frame
                    .store(name.to_string(), self.register.take().unwrap()),
                Op::Load(ReferenceOrValue::Value(value)) => {
                    self.register = Some(value.clone());
                }
                Op::Load(ReferenceOrValue::Reference(reference)) => {
                    let value = self.frame.load_required(reference)?;
                    self.register = Some(value.clone());
                }
                Op::EnterScope => {
                    self.frame.push_scope();
                }
                Op::ExitScope => {
                    self.frame.pop_scope();
                }
                Op::Call { path, params } => {
                    let mut loaded_params = vec![];
                    for param in params {
                        match param {
                            ReferenceOrValue::Reference(reference) => {
                                loaded_params.push(self.frame.load_required(reference)?.clone());
                            }
                            ReferenceOrValue::Value(value) => {
                                loaded_params.push(value.clone());
                            }
                        }
                    }

                    if path.eq("debug") {
                        for param in loaded_params {
                            print!("{} ", param);
                        }
                        println!();
                    } else {
                        println!("calling {path} with params: {loaded_params:?}");
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute() {
        let mut vm = Vm::default();
        vm.eval("let i: i32 = 1;").await.unwrap();
        vm.eval("let j: i32 = 2;").await.unwrap();
        vm.eval("let k: i32 = 3;").await.unwrap();
        vm.eval("debug(\"hello\", i, j, k);").await.unwrap();
    }
}
