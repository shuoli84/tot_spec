async fn test_it(i: serde_json::Value) -> anyhow::Result<base::SecondResponse> {
    let mut i: serde_json::Value = i.clone();
    let mut j: base::FirstResponse = my_crate::a::b::first({
        let s = i.clone();
        serde_json::from_value(s)?
    })
    .await?;
    let mut k: base::SecondResponse = my_crate::a::b::second({
        let s = j.clone();
        base::SecondRequest { foo: s.foo.clone() }
    })
    .await?;
    Ok(k.clone())
}

async fn test_number_to_string() -> anyhow::Result<String> {
    let mut i: i8 = 12;
    Ok({
        let s = i.clone();
        s.to_string()
    })
}

async fn test_float_to_string() -> anyhow::Result<String> {
    let mut f: f64 = 12.0;
    Ok({
        let s = f.clone();
        s.to_string()
    })
}

async fn test_integer_to_float() -> anyhow::Result<f64> {
    let mut f: i8 = 12;
    Ok({
        let s = f.clone();
        s as f64
    })
}

async fn test_bool_to_string() -> anyhow::Result<String> {
    let mut v: bool = true;
    Ok({
        let s = v.clone();
        s.to_string()
    })
}
