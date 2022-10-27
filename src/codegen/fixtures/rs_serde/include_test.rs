use base;
use base as base_dup;

/// TestBase
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TestBase {
    /// use base's BaseId as the id
    pub id: base::BaseId,

    /// use base_dup's BaseId as the id_2, this is just demo
    pub id_2: base_dup::BaseId,
}
