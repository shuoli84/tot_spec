use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(data: &ibig::IBig, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = data.to_string();
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<ibig::IBig, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    ibig::IBig::from_str(&s).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use ibig::ibig;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_ibig() {
        #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
        struct V {
            #[serde(with = "crate::ibig_serde_str")]
            value: ibig::IBig,
        }

        let v = V {
            value: ibig!(-123123213213213123123123123123123213),
        };

        let json_str = serde_json::to_string(&v).unwrap();
        assert_eq!(
            "{\"value\":\"-123123213213213123123123123123123213\"}",
            json_str
        );
        let v_back = serde_json::from_str::<V>(&json_str).unwrap();
        assert_eq!(v, v_back);
    }
}
