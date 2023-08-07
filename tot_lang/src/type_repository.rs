use anyhow::anyhow;
use std::ops::Add;
use tot_spec::codegen::context::Context;
use tot_spec::{ModelType, Type};

/// Provides type info for both codegen and vm
pub struct TypeRepository {
    /// the context which holds all loaded type info
    context: Context,
}

impl TypeRepository {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    /// Query the type for path.
    /// This resolves type reference to ModelType
    pub fn type_for_path(&self, type_path: &str) -> anyhow::Result<ModelOrType> {
        assert!(!type_path.is_empty());
        let mut components = type_path.split("::").collect::<Vec<_>>();
        let type_name = components.pop().unwrap();

        if components.is_empty() {
            // type is built in
            return Ok(ModelOrType::Type(Type::try_parse(type_name)?));
        }

        // get spec path from type_path
        // e.g: a::b::c => a/b/c.yaml (todo: remove the yaml part in future)
        let spec_path = components.join("/") + ".yaml";

        let def = self.context.get_definition(&spec_path)?;
        let model = def
            .get_model(type_name)
            .ok_or_else(|| anyhow!("not able to find type: {type_name}"))?;
        Ok(ModelOrType::ModelType(&model.type_))
    }
}

#[derive(Debug)]
pub enum ModelOrType<'a> {
    ModelType(&'a ModelType),
    Type(Type),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_type_repo() {
        let context = Context::new_from_folder(&PathBuf::from("src/codegen/fixtures")).unwrap();
        let type_repo = TypeRepository::new(context);

        assert!(dbg!(type_repo.type_for_path("i32")).is_ok());
        assert!(dbg!(type_repo.type_for_path("nested_base::nest_base::NestBaseInfo")).is_ok());
        assert!(dbg!(type_repo.type_for_path("base::BaseInfo")).is_ok());
    }
}
