use super::frame::Frame;
use crate::program::{Op, Program, ReferenceOrValue};
use crate::type_repository::TypeRepository;
use crate::VmBehavior;
use anyhow::anyhow;
use serde_json::Value;
use std::default::Default;
use std::sync::Arc;

/// The virtual machine, which stores all runtime state for one run
#[derive(Debug)]
pub struct Vm {
    frame: Frame,
    register: Option<Value>,
    behavior: Box<dyn VmBehavior>,
    type_repository: Arc<TypeRepository>,
}

impl Vm {
    pub fn new(behavior: Box<dyn VmBehavior>, type_repository: Arc<TypeRepository>) -> Self {
        Self {
            frame: Frame::default(),
            register: None,
            behavior,
            type_repository,
        }
    }
}

impl Vm {
    pub async fn eval(&mut self, code: &str) -> anyhow::Result<()> {
        let program = Program::from_statement(code)?;
        self.execute(&program).await?;
        Ok(())
    }

    /// Execute the ast
    pub async fn execute(&mut self, program: &Program) -> anyhow::Result<()> {
        let mut skip_count: usize = 0;
        for op in program.operations() {
            if skip_count > 0 {
                skip_count -= 1;
                continue;
            }

            match op {
                Op::Declare { name, .. } => {
                    // store a null value, so the store op will modify the one for current scope
                    self.frame.store(name.to_string(), Value::Null);
                }
                Op::Store { name } => self.frame.store(
                    name.to_string(),
                    self.register
                        .take()
                        .ok_or_else(|| anyhow!("register has no value"))?,
                ),
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
                        let result = self
                            .behavior
                            .runtime_call_method(path, &loaded_params)
                            .await?;
                        self.register = Some(result);
                    }
                }
                Op::Return => {
                    while self.frame.depth() > 0 {
                        self.frame.pop_scope();
                    }
                }
                Op::Jump(ops_to_skip) => skip_count = *ops_to_skip,
                Op::JumpIfFalse(ops_to_skip) => {
                    let value = self.register.take();
                    if !bool_for_value(&value) {
                        skip_count = *ops_to_skip
                    }
                }
            }
        }
        Ok(())
    }

    /// get current value at register, it is the way to get the result
    pub fn value(&self) -> Option<&Value> {
        self.register.as_ref()
    }

    /// consume self and return the value
    pub fn into_value(self) -> Option<Value> {
        self.register
    }
}

/// convert value to bool value
/// e.g, 0 -> false  null -> false  "" -> false
fn bool_for_value(value: &Option<Value>) -> bool {
    let Some(value) = value else { return false };

    match value {
        Value::Null => false,
        Value::Bool(v) => *v,
        Value::Number(num) => {
            if let Some(v) = num.as_i64() {
                v != 0
            } else if let Some(v) = num.as_u64() {
                v != 0
            } else if let Some(v) = num.as_f64() {
                v != 0.0
            } else {
                false
            }
        }
        Value::String(s) => !s.is_empty(),
        Value::Array(l) => !l.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::bail;
    use std::path::PathBuf;
    use tot_spec::codegen::context::Context;

    #[derive(Debug, Default)]
    struct TestBehavior {}

    #[async_trait::async_trait]
    impl VmBehavior for TestBehavior {
        async fn runtime_call_method(
            &mut self,
            method: &str,
            _params: &[Value],
        ) -> anyhow::Result<Value> {
            match method {
                "foo" => return Ok(Value::String("foo".into())),
                "bar" => return Ok(Value::String("bar".into())),
                _ => bail!("{method} not supported"),
            }
        }
    }

    fn test_vm() -> Vm {
        let context = Context::new_from_folder(&PathBuf::from("src/codegen/fixtures")).unwrap();
        let type_repository = Arc::new(TypeRepository::new(context));
        Vm::new(Box::new(TestBehavior::default()), type_repository)
    }

    #[tokio::test]
    async fn test_execute() {
        let mut vm = test_vm();
        vm.eval("let i: i32 = 1;").await.unwrap();
        vm.eval("let j: i32 = 2;").await.unwrap();
        vm.eval("let k: i32 = 3;").await.unwrap();
        vm.eval("debug(\"hello\", i, j, k);").await.unwrap();
        vm.eval("let f: String = foo();").await.unwrap();
        vm.eval("let g: String = bar();").await.unwrap();
        vm.eval("return g;").await.unwrap();
        let result = vm.into_value();
        assert!(result.is_some());
        assert!(result.unwrap().as_str().unwrap().eq("bar"));
    }

    #[tokio::test]
    async fn test_execute_return_from_inner() {
        let mut vm = test_vm();
        vm.eval(
            r#"{
            let i: String = "hello";
            {
                return i;
            };
        };"#,
        )
        .await
        .unwrap();
        let result = vm.into_value();
        assert!(result.unwrap().as_str().unwrap().eq("hello"));
    }

    #[tokio::test]
    async fn test_execute_if_else_true() {
        let mut vm = test_vm();
        vm.eval(
            r#"{
            if 1 {
                return 100;
            } else { 
                return 200;
            };
        };"#,
        )
        .await
        .unwrap();
        let result = vm.into_value();
        assert_eq!(result.unwrap().as_i64().unwrap(), 100);
    }

    #[tokio::test]
    async fn test_execute_if_else_false() {
        let mut vm = test_vm();
        vm.eval(
            r#"{
            if 0 {
                return 100;
            } else { 
                return 200;
            };
        };"#,
        )
        .await
        .unwrap();
        let result = vm.into_value();
        assert_eq!(result.unwrap().as_i64().unwrap(), 200);
    }

    #[tokio::test]
    async fn test_execute_if_no_else() {
        let mut vm = test_vm();
        vm.eval(
            r#"{
            if 0 {
                return 100;
            };
            200
        };"#,
        )
        .await
        .unwrap();
        let result = vm.into_value();
        assert_eq!(result.unwrap().as_i64().unwrap(), 200);
    }
}
