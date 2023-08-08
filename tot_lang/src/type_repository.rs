use anyhow::anyhow;
use std::borrow::Cow;
use std::ops::Add;
use std::path::PathBuf;
use tot_spec::codegen::context::{Context, SpecId};
use tot_spec::{ModelDef, ModelType, Type, TypeReference};

/// Provides type info for both codegen and vm
#[derive(Debug)]
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

        let spec_id = self
            .context
            .spec_for_path(&spec_path)?
            .ok_or_else(|| anyhow!("not able to get spec_id for path {spec_path:?}"))?;
        let def = self.context.get_definition(spec_id)?;
        let model = def
            .get_model(type_name)
            .ok_or_else(|| anyhow!("not able to find type: {type_name}"))?;
        Ok(ModelOrType::ModelType(Cow::Borrowed(&model), spec_id))
    }

    pub fn type_for_type_reference(
        &self,
        type_reference: &TypeReference,
        spec_id: SpecId,
    ) -> anyhow::Result<ModelOrType> {
        let model = self
            .context
            .get_model_def_for_reference(type_reference, spec_id)?;
        Ok(ModelOrType::ModelType(Cow::Borrowed(&model), spec_id))
    }
}

#[derive(Debug)]
pub enum ModelOrType<'a> {
    /// ModelType, it also holds the spec_id to resolve included type_reference
    ModelType(Cow<'a, ModelDef>, SpecId),
    /// Parsed type, it also holds the source spec_id resolve included type_reference
    Type(Type),
}

impl ModelOrType<'_> {
    pub fn to_owned(self) -> ModelOrType<'static> {
        match self {
            ModelOrType::ModelType(model_def, spec_id) => {
                ModelOrType::ModelType(Cow::Owned(model_def.as_ref().clone()), spec_id)
            }
            ModelOrType::Type(ty) => ModelOrType::Type(ty),
        }
    }
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
