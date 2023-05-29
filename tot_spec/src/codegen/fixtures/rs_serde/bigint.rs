/// struct for bigint field
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TestBigInt {
    #[serde(with = "tot_spec_util::ibig_serde_str")]
    pub value: std::option::Option<ibig::BigInt>,
}
