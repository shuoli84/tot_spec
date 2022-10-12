use std::fmt::Write;

use crate::Definition;

pub fn render(def: &Definition) -> anyhow::Result<String> {
    let mut result = String::new();

    for model in def.models.iter() {
        writeln!(&mut result, "\n/// {}", model.name)?;

        match &model.type_ {
            crate::ModelType::Enum { variants } => {
                writeln!(
                    &mut result,
                    "#[derive(Debug, serde::Serialize, serde::Deserialize)]"
                )?;
                writeln!(&mut result, "pub enum {} {{", &model.name)?;

                for variant in variants {
                    writeln!(
                        &mut result,
                        "  {}({}),",
                        variant.name,
                        variant.playload_type.rs_type()
                    )?;
                }

                writeln!(&mut result, "}}")?;
            }
            crate::ModelType::Struct(struct_def) => {
                writeln!(
                    &mut result,
                    "#[derive(Debug, serde::Serialize, serde::Deserialize)]"
                )?;
                writeln!(&mut result, "pub struct {} {{", &model.name)?;
                for field in struct_def.fields.iter() {
                    writeln!(&mut result, "  pub {}: {},", field.name, field.rs_type())?;
                }
                writeln!(&mut result, "}}")?;
            }
            crate::ModelType::NewType { inner_type } => {
                writeln!(
                    &mut result,
                    "#[derive(Debug, serde::Serialize, serde::Deserialize)]"
                )?;
                writeln!(
                    &mut result,
                    "pub struct {} ({});",
                    &model.name,
                    inner_type.rs_type()
                )?;
            }
        }
    }

    Ok(result)
}
