use super::frame::Frame;
use crate::program::{Op, Program, ReferenceOrValue};
use crate::type_repository::{ModelOrType, TypeRepository};
use crate::VmBehavior;
use anyhow::{anyhow, bail};
use serde_json::{Number, Value};
use std::default::Default;
use std::sync::Arc;
use tot_spec::{ModelType, StructDef, Type};

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
        let mut operations = program.operations().into_iter();

        while let Some(op) = operations.next() {
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
                Op::Convert { target_path } => {
                    let value = self
                        .register
                        .take()
                        .ok_or_else(|| anyhow!("register didn't set"))?;

                    let target_type = self.type_repository.type_for_path(target_path)?;
                    let converted_value = match target_type {
                        ModelOrType::Type(ty) => self.convert_value(value, ty)?,
                        ModelOrType::ModelType(model_type) => {
                            self.convert_value_to_model(value, &model_type.clone())?
                        }
                    };
                    self.register = Some(converted_value);
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

    fn convert_value(&mut self, value: Value, ty: Type) -> anyhow::Result<Value> {
        Ok(match ty {
            Type::Bool => Value::Bool(bool_for_value(&Some(value))),
            Type::I8 | Type::I16 | Type::I32 | Type::I64 => match value {
                Value::Number(number) => {
                    if let Some(i64_value) = number.as_i64() {
                        match ty {
                            Type::I8 => {
                                if i64_value > i8::MAX as i64 {
                                    bail!("overflow");
                                } else {
                                    Value::Number(number)
                                }
                            }
                            Type::I16 => {
                                if i64_value > i16::MAX as i64 {
                                    bail!("overflow");
                                } else {
                                    Value::Number(number)
                                }
                            }
                            Type::I32 => {
                                if i64_value > i32::MAX as i64 {
                                    bail!("overflow");
                                } else {
                                    Value::Number(Number::from(i64_value))
                                }
                            }
                            Type::I64 => Value::Number(number),
                            _ => {
                                unreachable!()
                            }
                        }
                    } else if let Some(u64_value) = number.as_u64() {
                        match ty {
                            Type::I8 => {
                                if u64_value > i8::MAX as u64 {
                                    bail!("overflow");
                                } else {
                                    Value::Number(number)
                                }
                            }
                            Type::I16 => {
                                if u64_value > i16::MAX as u64 {
                                    bail!("overflow");
                                } else {
                                    Value::Number(number)
                                }
                            }
                            Type::I32 => {
                                if u64_value > u32::MAX as u64 {
                                    bail!("overflow");
                                } else {
                                    Value::Number(number)
                                }
                            }
                            Type::I64 => Value::Number(number),
                            _ => {
                                unreachable!()
                            }
                        }
                    } else {
                        bail!("not able to convert")
                    }
                }
                _ => {
                    bail!("not able to convert value from {value} to {ty:?}");
                }
            },
            Type::F64 => {
                todo!()
            }
            Type::Decimal => {
                todo!()
            }
            Type::BigInt => {
                todo!()
            }
            Type::Bytes => {
                todo!()
            }
            Type::String => match ty {
                Type::String => value,
                _ => {
                    bail!("cant' convert from string to {ty:?}");
                }
            },
            Type::List { .. } => {
                todo!()
            }
            Type::Map { .. } => {
                todo!()
            }
            Type::Reference(user_type) => {
                dbg!(user_type);
                todo!()
            }
            Type::Json => value,
        })
    }

    /// convert value to model, the returned value is also a Value
    fn convert_value_to_model(
        &mut self,
        value: Value,
        model_type: &ModelType,
    ) -> anyhow::Result<Value> {
        let Value::Object(obj)  = value else {
            bail!("only object supported");
        };

        match model_type {
            ModelType::Struct(st) => return self.convert_json_map_to_struct(obj, st),
            _ => {
                bail!("not supported")
            }
        }
    }

    fn convert_json_map_to_struct(
        &mut self,
        value: serde_json::Map<String, Value>,
        st: &StructDef,
    ) -> anyhow::Result<Value> {
        // todo: support extend
        assert!(st.extend.is_none());

        let mut result = serde_json::Map::<String, Value>::new();

        for field in st.fields.iter() {
            let field_name = &field.name;

            match value.get(field_name) {
                None => {
                    if !field.required {
                        // ignore optional field
                        continue;
                    }

                    bail!("missing key for required field [{field_name}]")
                }
                Some(value) => {
                    let field_ty = field.type_.inner();
                    let field_value = self.convert_value(value.clone(), field_ty.clone())?;
                    result.insert(field_name.to_string(), field_value);
                }
            }
        }

        Ok(Value::Object(result))
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
            params: &[Value],
        ) -> anyhow::Result<Value> {
            match method {
                "json" => {
                    let param = &params[0];
                    let str_val = param.as_str().unwrap();
                    return Ok(serde_json::from_str::<Value>(str_val)?);
                }
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
        vm.eval("let f: string = foo();").await.unwrap();
        vm.eval("let g: string = bar();").await.unwrap();
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
            let i: string = "hello";
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

    #[tokio::test]
    async fn test_execute_convert_to_number_i8() {
        let mut vm = test_vm();
        vm.eval(
            r#"{
            let i: i64 = 4;
            i as i8
        };"#,
        )
        .await
        .unwrap();
        let result = vm.into_value();
        assert_eq!(result.unwrap().as_i64().unwrap(), 4);
    }

    #[tokio::test]
    async fn test_execute_convert_number_to_json() {
        let mut vm = test_vm();
        vm.eval(
            r#"{
            let i: i64 = 4;
            i as json
        };"#,
        )
        .await
        .unwrap();
        let result = vm.into_value();
        assert_eq!(result.unwrap().as_i64().unwrap(), 4);
    }

    #[tokio::test]
    async fn test_execute_convert_json_to_request() {
        let mut vm = test_vm();
        vm.eval(
            r#"{
            let i: json = json("{\"foo\": \"bar\", \"ignore\": true}");
            i as base::FirstRequest
        };"#,
        )
        .await
        .unwrap();
        let result = vm.into_value().unwrap();
        assert_eq!(
            result,
            serde_json::json!({
                "foo": "bar"
            })
        );
    }

    #[tokio::test]
    async fn test_execute_convert_json_to_struct_nested() {
        let mut vm = test_vm();
        vm.eval(
            r#"{
            let i: json = json("{\"foo\": 123, \"ignore\": true, \"nested_base_info\": {  }}");
            i as spec::TestStruct
        };"#,
        )
        .await
        .unwrap();
        let result = vm.into_value().unwrap();
        assert_eq!(
            result,
            serde_json::json!({
                "foo": "bar"
            })
        );
    }
}
