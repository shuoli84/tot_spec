# RPC Service Patterns

Common RPC service definitions using tot_spec YAML format.

## JSON-RPC Models

```yaml
meta:
  rs_serde:
    crate_name: jsonrpc

models:
# Base request with common fields
- name: JsonRpcRequest
  type:
    name: struct
    fields:
      - name: jsonrpc
        type: string
        required: true
      - name: method
        type: string
        required: true
      - name: params
        type: json
        required: false
      - name: id
        type: i64
        required: true

# Base response
- name: JsonRpcResponse
  type:
    name: struct
    fields:
      - name: jsonrpc
        type: string
        required: true
      - name: id
        type: i64
        required: true

# Error structure
- name: JsonRpcError
  type:
    name: struct
    fields:
      - name: code
        type: i32
        required: true
      - name: message
        type: string
        required: true
      - name: data
        type: json
        required: false

# Success response (result variant)
- name: JsonRpcSuccess
  type:
    name: struct
    extend: JsonRpcResponse
    fields:
      - name: result
        type: json
        required: true

# Error response wrapper
- name: JsonRpcErrorResponse
  type:
    name: struct
    extend: JsonRpcResponse
    fields:
      - name: error
        type:
          name: ref
          target: JsonRpcError
        required: true
```

## Error Codes (Const)

```yaml
- name: JsonRpcErrorCode
  type:
    name: const
    value_type: i32
    values:
      - name: ParseError
        value: -32700
      - name: InvalidRequest
        value: -32600
      - name: MethodNotFound
        value: -32601
      - name: InvalidParams
        value: -32602
      - name: InternalError
        value: -32603
```

## Example: User Service

```yaml
models:
# User entity
- name: User
  type:
    name: struct
    fields:
      - name: id
        type: string
        required: true
      - name: name
        type: string
        required: true
      - name: email
        type: string
        required: true
      - name: created_at
        type: i64
        required: true

# Request models
- name: GetUserRequest
  type:
    name: struct
    fields:
      - name: user_id
        type: string
        required: true

- name: CreateUserRequest
  type:
    name: struct
    fields:
      - name: name
        type: string
        required: true
      - name: email
        type: string
        required: true

- name: UpdateUserRequest
  type:
    name: struct
    extend: GetUserRequest
    fields:
      - name: name
        type: string
      - name: email
        type: string

- name: DeleteUserRequest
  type:
    name: struct
    extend: GetUserRequest

# Response models
- name: UserResponse
  type:
    name: struct
    fields:
      - name: user
        type:
          name: ref
          target: User
        required: true

- name: UserListResponse
  type:
    name: struct
    fields:
      - name: users
        type:
          name: list
          item_type: User
        required: true
      - name: total
        type: i64
        required: true

- name: EmptyResponse
  type:
    name: struct
    fields: []

# Service methods
methods:
  - name: GetUser
    request: GetUserRequest
    response: UserResponse

  - name: CreateUser
    request: CreateUserRequest
    response: UserResponse

  - name: UpdateUser
    request: UpdateUserRequest
    response: UserResponse

  - name: DeleteUser
    request: DeleteUserRequest
    response: EmptyResponse

  - name: ListUsers
    request: EmptyResponse
    response: UserListResponse
```

## Virtual Base for Extensibility

```yaml
# Base request with common fields
- name: BaseRequest
  type:
    name: virtual
    fields:
      - name: request_id
        type: string
      - name: trace_id
        type: string

# All requests extend base
- name: GetUserRequest
  type:
    name: struct
    extend: BaseRequest
    fields:
      - name: user_id
        type: string
        required: true
```

## Rust Handler Code Generation

Custom Codegen to generate RPC handlers from spec.

### Embedded Types

```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Definition {
    pub name: String,
    pub models: Vec<ModelDef>,
    pub methods: Vec<MethodDef>,
}

impl Definition {
    pub fn load_from_yaml(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let def = serde_yaml::from_str::<YamlDefinition>(&content)?;
        Ok(Definition {
            name: def.name,
            models: def.models,
            methods: def.methods,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDef {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: ModelType,
    #[serde(default)]
    pub desc: Option<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum ModelType {
    Struct { fields: Vec<FieldDef> },
    Enum { variants: Vec<VariantDef> },
    NewType { inner_type: String },
    Virtual { fields: Vec<FieldDef> },
    Const { value_type: String, values: Vec<ConstValue> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    #[serde(default)]
    pub type_: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantDef {
    pub name: String,
    #[serde(default)]
    pub payload_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstValue {
    pub name: String,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDef {
    pub name: String,
    #[serde(default)]
    pub desc: Option<String>,
    pub request: String,
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct YamlDefinition {
    name: String,
    #[serde(default)]
    models: Vec<ModelDef>,
    #[serde(default)]
    methods: Vec<MethodDef>,
}
```

### Codegen Implementation

```rust
struct RpcHandlerGen;

impl RpcHandlerGen {
    fn generate_for_folder(folder: &PathBuf, output: &PathBuf) -> anyhow::Result<()> {
        for entry in walkdir::WalkDir::new(folder)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "yaml").unwrap_or(false))
        {
            let def = Definition::load_from_yaml(entry.path())?;
            let content = Self::generate_handlers(entry.path(), &def);
            let output_path = output.join(
                entry.path()
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
                    + "_handlers.rs",
            );
            std::fs::write(output_path, content)?;
        }
        Ok(())
    }

    fn generate_handlers(path: &Path, def: &Definition) -> String {
        let mut code = String::new();

        code.push_str(&format!("// Auto-generated from {}\n\n", path.display()));
        code.push_str("use async_trait::async_trait;\n");
        code.push_str("use serde::{Deserialize, Serialize};\n\n");

        code.push_str("#[async_trait]\n");
        code.push_str(&format!("pub trait {}Service {{\n", to_camel_case(&def.name)));

        for method in &def.methods {
            code.push_str(&format!(
                "    async fn {}(&self, req: {}) -> anyhow::Result<{}>;\n",
                to_snake_case(&method.name),
                method.request,
                method.response
            ));
        }
        code.push_str("}\n\n");

        code.push_str(&format!("pub struct {}ServiceImpl<S> {{\n", to_camel_case(&def.name)));
        code.push_str("    _phantom: std::marker::PhantomData<S>,\n");
        code.push_str("}\n\n");

        code.push_str(&format!(
            "impl<S> {}ServiceImpl<S> {{\n",
            to_camel_case(&def.name)
        ));
        code.push_str("    pub fn new() -> Self {\n");
        code.push_str("        Self { _phantom: std::marker::PhantomData }\n");
        code.push_str("    }\n");
        code.push_str("}\n\n");

        code.push_str(&format!(
            "impl<S: {}Service + Send + Sync> {}Service for {}ServiceImpl<S> {{\n",
            to_camel_case(&def.name),
            to_camel_case(&def.name),
            to_camel_case(&def.name)
        ));

        for method in &def.methods {
            code.push_str(&format!(
                "    async fn {}(&self, req: {}) -> anyhow::Result<{}> {{\n",
                to_snake_case(&method.name),
                method.request,
                method.response
            ));
            code.push_str(&format!(
                "        unimplemented!(\"{}::{}\")\n",
                def.name, method.name
            ));
            code.push_str("    }\n");
        }
        code.push_str("}\n");

        code
    }
}

### Helper Functions

```rust
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.extend(c.to_lowercase());
    }
    result
}
```
