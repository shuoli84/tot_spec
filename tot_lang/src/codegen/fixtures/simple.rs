async fn hello(i: String) -> anyhow::Result<String> {
    let mut j: String = i.clone();
    let mut k: String = {
        if true {
            "foo".to_string()
        } else {
            "bar".to_string()
        }
    };
    if true {
        return Ok("foo".to_string());
    } else {
        return Ok("bar".to_string());
    };
    println!("{}", k.clone());
    let mut sync_call_result: String = my_crate::a::b::sync_func(k.clone())?;
    let mut async_call_result: String =
        my_crate::a::b::async_func(sync_call_result.clone()).await?;
    Ok(k.clone())
}
