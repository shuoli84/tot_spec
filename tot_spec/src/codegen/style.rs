use crate::{FieldDef, ModelDef, ModelType};
use convert_case::{Boundary, Case as ConvertCase, Casing};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Style {
    #[serde(default)]
    field_name_case: Case,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub enum Case {
    #[serde(rename = "snake")]
    Snake,
    #[serde(rename = "camel")]
    Camel,
    #[default]
    Unspecified,
}

impl Case {
    fn is_case(&self, name: &str) -> bool {
        self.convert(name).eq(name)
    }

    fn convert(&self, name: &str) -> String {
        let convert_case = match self {
            Case::Snake => ConvertCase::Snake,
            Case::Camel => ConvertCase::Camel,
            Case::Unspecified => return name.to_string(),
        };

        name.with_boundaries(&[
            Boundary::DigitUpper,
            Boundary::DigitLower,
            Boundary::Space,
            Boundary::Acronym,
            Boundary::Hyphen,
            Boundary::LowerUpper,
        ])
        .to_case(convert_case)
    }
}

impl Style {
    pub fn validate_model(&self, model: &ModelDef) -> Vec<String> {
        match &model.type_ {
            ModelType::Struct(st_) => self.validate_field_name(&st_.fields),
            ModelType::Virtual(st_) => self.validate_field_name(&st_.fields),
            _ => vec![],
        }
    }

    fn validate_field_name(&self, fields: &[FieldDef]) -> Vec<String> {
        let mut violations = vec![];
        for field in fields {
            if !self.field_name_case.is_case(&field.name) {
                violations.push(format!(
                    "field {} is not case {:?} should be {}",
                    field.name,
                    self.field_name_case,
                    self.field_name_case.convert(&field.name)
                ));
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case() {
        assert!(Case::Snake.is_case("i8_value"));
        assert!(Case::Snake.is_case("i64_value"));
        assert!(Case::Snake.is_case("http_request"));

        assert_eq!(Case::Snake.convert("i8Value"), "i8_value");
        assert_eq!(Case::Snake.convert("i64Value"), "i64_value");
        assert_eq!(Case::Snake.convert("HTTPRequest"), "http_request");
    }
}
