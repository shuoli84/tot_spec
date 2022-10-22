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
                ..Default::default()
            },
            ModelDef {
                name: "KeyValue".into(),
                type_: ModelType::new_type(Type::map(Type::Bytes)),
                ..Default::default()
            },
            ModelDef {
                name: "Container".into(),
                type_: ModelType::new_type(Type::list(Type::reference("SimpleStruct"))),
                ..Default::default()
            },
            ModelDef {
                name: "Base".into(),
                type_: ModelType::Virtual(StructDef {
                    extend: None,
                    fields: vec![FieldDef::new("request_id", Type::String)],
                }),
                ..Default::default()
            },
            ModelDef {
                name: "Number".into(),
                type_: ModelType::Enum {
                    variants: vec![
                        VariantDef {
                            name: "I64".into(),
                            desc: None,
                            payload_type: Some(Type::I64.into()),
                        },
                        VariantDef {
                            name: "F64".into(),
                            desc: None,
                            payload_type: Some(Type::F64.into()),
                        },
                    ],
                },
                ..Default::default()
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
                ..Default::default()
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
                ..Default::default()
            },
        ],
        meta: Default::default(),
    };
    println!("{}", serde_yaml::to_string(&def).unwrap());
}
