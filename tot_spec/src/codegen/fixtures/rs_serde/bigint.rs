/// struct for bigint field
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestBigInt {
    pub value: std::option::Option<tot_spec_util::big_int::BigInt>,
}
