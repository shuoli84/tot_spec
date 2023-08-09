use super::scope::*;
use crate::Value;

/// A function call frame, it is the state for one function
#[derive(Debug)]
pub struct Frame {
    // scope stack, last is the top
    scopes: Vec<Scope>,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            scopes: vec![Scope::default()],
        }
    }
}

impl Frame {
    /// Returns the depth for current scope
    pub fn depth(&self) -> usize {
        self.scopes.len()
    }

    /// push a new scope to the frame
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    /// pop the scope
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Store to current scope
    pub fn store(&mut self, name: impl Into<String>, value: Value) {
        self.current_scope_mut().insert(name.into(), value);
    }

    /// Assign, if not found, return error
    pub fn assign(&mut self, name: &str, value: Value) -> anyhow::Result<()> {
        for scope in self.scopes.iter_mut().rev() {
            match scope.vars.get_mut(name) {
                None => continue,
                Some(v) => {
                    *v = value;
                    return Ok(());
                }
            }
        }
        anyhow::bail!("{name} not found");
    }

    /// assign value to subfield of name
    pub fn assign_by_path(
        &mut self,
        name: &str,
        path: &[PathComponent],
        value: Value,
    ) -> anyhow::Result<()> {
        for scope in self.scopes.iter_mut().rev() {
            match scope.vars.get_mut(name) {
                None => continue,
                Some(v) => {
                    return assign_by_path(v, path, value);
                }
            }
        }

        anyhow::bail!("{name} not found");
    }

    /// load value for name recursively, it looks up parent scopes
    pub fn load(&self, name: &str) -> Option<&Value> {
        for s in self.scopes.iter().rev() {
            if let Some(v) = s.vars.get(name) {
                return Some(v);
            }
        }

        None
    }

    /// load the name, if it doesn't exist, then raise an error
    pub fn load_required(&self, name: &str) -> anyhow::Result<&serde_json::Value> {
        self.load(name)
            .ok_or_else(|| anyhow::anyhow!("local var not found: {name}"))
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }
}

pub enum PathComponent<'a> {
    /// Field, both for struct field and json object key
    Field(&'a str),
    Index(usize),
}

fn assign_by_path(target: &mut Value, path: &[PathComponent], value: Value) -> anyhow::Result<()> {
    match path.split_first() {
        Some((first_component, rest_paths)) => match first_component {
            PathComponent::Field(field_name) => match target {
                Value::Object(obj) => match obj.get_mut(*field_name) {
                    None => {
                        anyhow::bail!("object missing key {field_name}")
                    }
                    Some(target) => {
                        return Ok(assign_by_path(target, rest_paths, value)?);
                    }
                },
                _ => {
                    anyhow::bail!("only object and array supports paths")
                }
            },
            PathComponent::Index(idx) => match target {
                Value::Array(arr) => {
                    if arr.len() <= *idx {
                        anyhow::bail!("array out of index");
                    }

                    return Ok(assign_by_path(
                        arr.get_mut(*idx).expect("checked"),
                        rest_paths,
                        value,
                    )?);
                }
                _ => anyhow::bail!("only array supports index"),
            },
        },
        None => {
            *target = value;
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign_by_path() {
        let mut value = serde_json::json!({
            "foo": "bar",
            "list": [
                "field_1", "field_2"
            ],
            "nested_obj": {
                "nest_foo": "bar",
            }
        });

        assign_by_path(
            &mut value,
            &[PathComponent::Field("foo")],
            Value::String("bar bar".to_string()),
        )
        .unwrap();

        assert_eq!(value.get("foo").unwrap().as_str().unwrap(), "bar bar");

        assign_by_path(
            &mut value,
            &[PathComponent::Field("list"), PathComponent::Index(1)],
            Value::String("bar bar".to_string()),
        )
        .unwrap();

        assert_eq!(
            value
                .get("list")
                .unwrap()
                .as_array()
                .unwrap()
                .get(1)
                .unwrap(),
            "bar bar",
        );

        assign_by_path(
            &mut value,
            &[
                PathComponent::Field("nested_obj"),
                PathComponent::Field("nest_foo"),
            ],
            Value::String("bar bar".to_string()),
        )
        .unwrap();

        assert_eq!(
            value
                .get("nested_obj")
                .unwrap()
                .as_object()
                .unwrap()
                .get("nest_foo")
                .unwrap(),
            "bar bar",
        );
    }
}
