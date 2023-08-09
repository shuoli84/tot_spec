async fn test_assign_to_field(i: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let mut v: spec::TestStruct = {
        let s = i.clone();
        serde_json::from_value(s)?
    };
    v.foo = "bar bar".to_string();
    Ok({
        let s = v.clone();
        serde_json::to_value(&s)?
    })
}
