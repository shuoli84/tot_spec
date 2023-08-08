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
