/// struct for bigint field
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TestBigInt {
    pub value: std::option::Option<ibig::BigInt>,
}
