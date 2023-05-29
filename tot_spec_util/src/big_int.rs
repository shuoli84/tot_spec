use std::{ops::Deref, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A wrapper struct which can be used to serialize/deserialize ibig::IBig to/from string
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct BigInt(ibig::IBig);

impl BigInt {
    /// Create a new BigInt instance from ibig::IBig
    pub fn new(value: ibig::IBig) -> Self {
        Self(value)
    }

    /// Get a reference to inner value
    pub fn inner(&self) -> &ibig::IBig {
        &self.0
    }
}

impl From<ibig::IBig> for BigInt {
    fn from(value: ibig::IBig) -> Self {
        Self(value)
    }
}

impl Deref for BigInt {
    type Target = ibig::IBig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for BigInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for BigInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(deserialize(deserializer)?))
    }
}

fn serialize<S>(data: &ibig::IBig, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = data.to_string();
    serializer.serialize_str(&s)
}

fn deserialize<'de, D>(deserializer: D) -> Result<ibig::IBig, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    ibig::IBig::from_str(&s).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ibig::ibig;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_big_int() {
        #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
        struct V {
            value: BigInt,
        }

        let v = V {
            value: BigInt(ibig!(-123123213213213123123123123123123213)),
        };

        let json_str = serde_json::to_string(&v).unwrap();
        assert_eq!(
            "{\"value\":\"-123123213213213123123123123123123213\"}",
            json_str
        );
        let v_back = serde_json::from_str::<V>(&json_str).unwrap();
        assert_eq!(v, v_back);
    }

    #[test]
    fn test_big_int_optional() {
        #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
        struct V {
            value: Option<BigInt>,
        }

        let v = V {
            value: Some(BigInt(ibig!(-123123213213213123123123123123123213))),
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
