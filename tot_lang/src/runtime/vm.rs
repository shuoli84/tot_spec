use super::frame::Frame;
use crate::program::{Op, Program, ReferenceOrValue};
use crate::type_repository::{ModelOrType, TypeRepository};
use crate::VmBehavior;
use anyhow::{anyhow, bail};
use serde_json::{Number, Value};
use std::default::Default;
use std::sync::Arc;
use tot_spec::{ModelDef, ModelType, SpecId, StructDef, Type};

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
                Op::Load(ReferenceOrValue::Reference { var_name, path }) => {
                    assert!(path.is_empty(), "not supported yet");
                    let value = self.frame.load_required(var_name)?;
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
                            ReferenceOrValue::Reference { var_name, path } => {
                                assert!(path.is_empty(), "not supported yet");
                                loaded_params.push(self.frame.load_required(var_name)?.clone());
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

                    let target_type = self.type_repository.type_for_path(target_path)?.to_owned();
                    self.register = Some(self.convert_value_to_model_or_type(value, target_type)?);
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

    /// if from_spec is Some, then this type should be resolved in that spec's context
    fn convert_value(
        &self,
        value: Value,
        target_type: &Type,
        from_spec: Option<SpecId>,
    ) -> anyhow::Result<Value> {
        Ok(match target_type {
            // todo: verify that convert to bool codegen
            Type::Bool => Value::Bool(bool_for_value(&Some(value))),
            Type::I8 | Type::I16 | Type::I32 | Type::I64 => match value {
                Value::Number(number) => {
                    if let Some(i64_value) = number.as_i64() {
                        match target_type {
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
                        match target_type {
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
                                if u64_value > i32::MAX as u64 {
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
                    } else if let Some(f64_value) = number.as_f64() {
                        match target_type {
                            Type::I8 => Value::Number(Number::from(f64_value as i8)),
                            Type::I16 => Value::Number(Number::from(f64_value as i16)),
                            Type::I32 => Value::Number(Number::from(f64_value as i32)),
                            Type::I64 => Value::Number(Number::from(f64_value as i64)),
                            _ => {
                                unreachable!()
                            }
                        }
                    } else {
                        bail!("not able to convert")
                    }
                }
                _ => {
                    bail!("not able to convert value from {value} to {target_type:?}");
                }
            },
            Type::F64 => match value {
                Value::Number(number) => {
                    let val = if let Some(val) = number.as_i64() {
                        val as f64
                    } else if let Some(val) = number.as_u64() {
                        val as f64
                    } else if let Some(val) = number.as_f64() {
                        val
                    } else {
                        bail!("value is not number")
                    };

                    Value::Number(
                        Number::from_f64(val).ok_or_else(|| anyhow!("f64 value out of range"))?,
                    )
                }
                _ => {
                    bail!("not able to convert {value:?} to f64");
                }
            },
            Type::Decimal => {
                todo!()
            }
            Type::BigInt => {
                todo!()
            }
            Type::String => {
                if value.is_string() {
                    value
                } else if value.is_number() {
                    Value::String(value.to_string())
                } else {
                    bail!("cant' convert from {value:?} to string");
                }
            }
            Type::Bytes => {
                todo!()
            }
            Type::List { .. } => {
                todo!()
            }
            Type::Map { .. } => {
                todo!()
            }
            Type::Reference(user_type) => {
                let Some(from_spec) = from_spec else {
                    bail!("type_reference should has relative spec set");
                };

                let model_or_type = self
                    .type_repository
                    .type_for_type_reference(&user_type, from_spec)?
                    .to_owned();

                self.convert_value_to_model_or_type(value, model_or_type)?
            }
            Type::Json => value,
        })
    }

    fn convert_value_to_model_or_type(
        &self,
        value: Value,
        model_or_type: ModelOrType<'static>,
    ) -> anyhow::Result<Value> {
        Ok(match model_or_type {
            ModelOrType::Type(ty) => self.convert_value(value, &ty, None)?,
            ModelOrType::ModelType(model_type, spec_id) => {
                self.convert_value_to_model(value, &model_type.clone(), spec_id)?
            }
        })
    }

    /// convert value to model, the returned value is also a Value
    fn convert_value_to_model(
        &self,
        value: Value,
        model_def: &ModelDef,
        spec_id: SpecId,
    ) -> anyhow::Result<Value> {
        match &model_def.type_ {
            ModelType::Struct(st) => {
                let Value::Object(obj)  = value else {
                    bail!("only object supported");
                };

                return self.convert_json_map_to_struct(obj, st, spec_id);
            }
            ModelType::NewType { inner_type } => {
                self.convert_value(value, inner_type, Some(spec_id))
            }
            _ => {
                bail!("not supported")
            }
        }
    }

    fn convert_json_map_to_struct(
        &self,
        value: serde_json::Map<String, Value>,
        st: &StructDef,
        spec_id: SpecId,
    ) -> anyhow::Result<Value> {
        let mut fields = if let Some(base_type) = st.extend.as_ref() {
            let base_type = base_type.inner();
            let base_type_model = self
                .type_repository
                .type_for_type_reference(base_type, spec_id)?;
            match base_type_model {
                ModelOrType::ModelType(model_type, _) => match &model_type.type_ {
                    ModelType::Virtual(ref virtual_model) => virtual_model.fields.clone(),
                    _ => {
                        unimplemented!();
                    }
                },
                ModelOrType::Type(_) => {
                    unimplemented!();
                }
            }
        } else {
            vec![]
        };
        fields.extend_from_slice(&st.fields);

        let mut result = serde_json::Map::<String, Value>::new();

        for field in fields.iter() {
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
                    let field_value = self.convert_value(value.clone(), field_ty, Some(spec_id))?;
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
    use tot_spec::Context;

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
    async fn test_execute_convert_to_number_f64() {
        let mut vm = test_vm();
        vm.eval(
            r#"{
            let i: i64 = 4;
            i as f64
        };"#,
        )
        .await
        .unwrap();
        let result = vm.into_value();
        assert_eq!(result.unwrap().as_f64().unwrap(), 4.0);
    }

    #[tokio::test]
    async fn test_execute_convert_string() {
        let mut vm = test_vm();
        vm.eval(
            r#"{
            let i: string = "hello";
            i as string
        };"#,
        )
        .await
        .unwrap();
        let result = vm.into_value();
        assert_eq!(result.unwrap().as_str().unwrap(), "hello");
    }

    #[tokio::test]
    async fn test_execute_convert_number_to_string() {
        let mut vm = test_vm();
        vm.eval(
            r#"{
            let i: i8 = 12;
            i as string
        };"#,
        )
        .await
        .unwrap();
        let result = vm.into_value();
        assert_eq!(result.unwrap().as_str().unwrap(), "12");
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
    async fn test_execute_convert_float_to_json() {
        let mut vm = test_vm();
        vm.eval(
            r#"{
            let f: f64 = 4.0;
            f as json
        };"#,
        )
        .await
        .unwrap();
        let result = vm.into_value();
        assert_eq!(result.unwrap().as_f64().unwrap(), 4.0);
    }

    #[tokio::test]
    async fn test_execute_convert_float_to_string() {
        let mut vm = test_vm();
        vm.eval(
            r#"{
            let f: f64 = 4.0;
            f as string
        };"#,
        )
        .await
        .unwrap();
        let result = vm.into_value();
        assert_eq!(result.unwrap().as_str().unwrap(), "4.0");
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
            r##"{
            let i: json = json("{
                \"foo\": 123, 
                \"ignore\": true, 
                \"nested_base_info\": { 
                    \"nested_base_field\": 12 
                }
            }");
            i as spec::TestStruct
        };"##,
        )
        .await
        .unwrap();
        let result = vm.into_value().unwrap();
        assert_eq!(
            result,
            serde_json::json!({
                "foo": 123,
                "nested_base_info": {
                    "nested_base_field": 12
                }
            })
        );
    }

    #[tokio::test]
    async fn test_execute_convert_json_to_struct_extend() {
        let mut vm = test_vm();
        vm.eval(
            r##"{
            let i: json = json("{
                \"common_i8_field\":  12 
            }");
            i as spec::TestExtend
        };"##,
        )
        .await
        .unwrap();
        let result = vm.into_value().unwrap();
        assert_eq!(
            result,
            serde_json::json!({
                "common_i8_field": 12
            })
        );
    }

    #[tokio::test]
    async fn test_execute_convert_json_to_new_type() {
        let mut vm = test_vm();
        vm.eval(
            r##"{
            let i: json = json("{
                \"foo\":  \"bar\" 
            }");
            i as spec::NewTypeStruct
        };"##,
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
    async fn test_execute_assign_to_field() {
        let mut vm = test_vm();
        vm.eval(
            r##"{
            let i: json = json("{
                \"foo\":  \"bar\" 
            }");
            let j: spec::NewTypeStruct = i as spec::NewTypeStruct;
            j.foo = "bar bar";
            j
        };"##,
        )
        .await
        .unwrap();
        let result = vm.into_value().unwrap();
        assert_eq!(
            result,
            serde_json::json!({
                "foo": "bar bar"
            })
        );
    }
}
