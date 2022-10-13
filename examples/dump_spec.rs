use tot_spec::*;

fn main() {
    let def = Definition {
        models: vec![
            ModelDef {
                name: "SimpleStruct".to_string(),
                type_: ModelType::Struct(StructDef {
                    extend: None,
                    fields: vec![
                        FieldDef::new("bool_value", Type::Bool).with_required(true),
                        FieldDef::new("i8_value", Type::I8).with_required(true),
                        FieldDef::new("i64_value", Type::I64),
                        FieldDef::new("string_value", Type::String),
                        FieldDef::new("bytes_value", Type::Bytes),
                        FieldDef::new("string_map", Type::map(Type::String)).with_attribute(
                            "rs_type",
                            "std::collections::BTreeMap::<i8, std::string::String>",
                        ),
                        FieldDef::new("key_values", Type::reference("KeyValue")),
                        FieldDef::new("children", Type::list(Type::reference("SimpleStruct"))),
                    ],
                }),
            },
            ModelDef {
                name: "KeyValue".into(),
                type_: ModelType::new_type(Type::map(Type::Bytes)),
            },
            ModelDef {
                name: "Container".into(),
                type_: ModelType::new_type(Type::list(Type::reference("SimpleStruct"))),
            },
            ModelDef {
                name: "Base".into(),
                type_: ModelType::Virtual(StructDef {
                    extend: None,
                    fields: vec![FieldDef::new("request_id", Type::String)],
                }),
            },
            ModelDef {
                name: "Number".into(),
                type_: ModelType::Enum {
                    variants: vec![
                        VariantDef {
                            name: "I64".into(),
                            playload_type: Type::I64,
                        },
                        VariantDef {
                            name: "F64".into(),
                            playload_type: Type::F64,
                        },
                    ],
                },
            },
            ModelDef {
                name: "AddRequest".into(),
                type_: ModelType::Struct(StructDef {
                    extend: Some("Base".into()),
                    fields: vec![FieldDef::new(
                        "numbers",
                        Type::list(Type::reference("Number")),
                    )],
                }),
            },
            ModelDef {
                name: "DeleteRequest".into(),
                type_: ModelType::Struct(StructDef {
                    extend: Some("Base".into()),
                    fields: vec![FieldDef::new(
                        "numbers",
                        Type::list(Type::reference("Number")),
                    )],
                }),
            },
        ],
    };
    println!("{}", serde_yaml::to_string(&def).unwrap());
}
