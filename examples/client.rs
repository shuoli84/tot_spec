use std::collections::{BTreeMap, HashMap};

mod spec;
use spec::example_spec::*;

fn main() {
    let simple_struct = SimpleStruct {
        bool_value: false,
        i8_value: 8,
        i64_value: 32.into(),
        string_value: Some("foo".into()),
        bytes_value: Some(vec![0u8, 1u8].into()),
        i8_to_string: Some(BTreeMap::from([
            (1i8, "foo".to_string()),
            (2i8, "bar".to_string()),
        ])),
        key_values: Some(KeyValue(HashMap::from([
            ("foo_key".to_string(), b"foo_value".to_vec()),
            ("bar_key".to_string(), b"bar_value".to_vec()),
        ]))),
        children: Some(vec![]),
        children_container: None,
    };
    println!("dbg output:\n{:#?}", simple_struct);
    println!(
        "yaml output:\n{}",
        serde_yaml::to_string(&simple_struct).unwrap()
    );
    let json_output = serde_json::to_string_pretty(&simple_struct).unwrap();
    println!("json output:\n{json_output}",);
    let value_back = serde_json::from_str::<SimpleStruct>(&json_output).unwrap();
    println!("value back: {:?}", value_back);
}
