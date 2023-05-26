# Introduction

tot_spec is a language agonostic model definition util. It is mainly used to define json models cross boundaries, e.g, api server and client, sdk rust and swift etc.

# Examples

## Run examples

```bash
# generate rust client code
cargo run --bin tot_spec -- -s "examples/spec/spec.yaml" -c "rs_serde" -o "examples/spec/example_spec.rs

# generate python client code
cargo run --bin tot_spec -- -s "examples/spec/spec.yaml" -c "py_dataclass" -o examples/spec/example_spec.py

# generate swift client code
cargo run --bin tot_spec -- -s "examples/spec/spec.yaml" -c "swift_codable" -o examples/swift_package/Sources/SpecModel/example_spec.swift
cd examples/swift_package && swift test

# generate java jackson
cargo run --bin tot_spec -- -s "examples/spec/spec.yaml" -c "java_jackson" -o "examples/java_jackson/example_app/src/main/java/"
```

## Nested struct

```yaml
models:
- name: SimpleStruct
  type:
    name: struct
    fields:
    # basic types including bool, i8, i64, f64 and string
    - name: bool_value
      type: bool
      required: true
    - name: i8_value
      type: i8
      required: true
    - name: i64_value
      type: i64
    - name: string_value
      type: string
    - name: bytes_value
      type: bytes

    # also container types including Map and List
    # now map keytype is restricted to string only
    - name: string_to_string
      type: map[string]
      attributes:
        # use rs_type attibute to mark underlying type as BTreeMap
        rs_type: std::collections::BTreeMap::<std::string::String, std::string::String>

    - name: key_values
      # reference other types defined in spec
      type: KeyValue

    # container list
    - name: children
      type: 
        name: list
        item_type: SimpleStruct

    # same list as above, string version
    - name: children_2
      type: list[SimpleStruct]
```

## New type

```yaml
- name: Container
  type:
    # type name as "new_type", which maps to rust's new type pattern
    # for other languages, this can be a no op
    name: new_type
    inner_type: list[SimpleStruct]

- name: UserId
  type:
    name: new_type
    inner_type: i64
  attributes:
    rs_extra_derive: Hash
```

## Virtual type

Use virtual type to define common part cross models.

```yaml
# Base contains some fields common cross all Request Models
- name: Base
  type:
    name: virtual
    fields:
    - name: request_id
      type: string

- name: AddRequest
  type:
    name: struct
    extend: Base
    fields:
    - name: numbers
      type:
        name: list
        item_type: i64

- name: ResetRequest
  type:
    name: struct
    extend: Base
```

In rust code, Base will be mapped to a trait as:

```rust
/// Base
pub trait Base {
    fn request_id(&self) -> &std::option::Option<std::string::String>;
}

/// AddRequest
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AddRequest {
    pub request_id: std::option::Option<std::string::String>,
    pub numbers: std::option::Option<std::vec::Vec<Number>>,
}

impl Base for AddRequest {
    fn request_id(&self) -> &std::option::Option<std::string::String> {
        &self.request_id
    }
}

/// ResetRequest
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ResetRequest {
    pub request_id: std::option::Option<std::string::String>,
}

impl Base for ResetRequest {
    fn request_id(&self) -> &std::option::Option<std::string::String> {
        &self.request_id
    }
}
```

## Const

Define integer or string consts

```yaml
models:
  - name: Code
    desc: Const def for i8
    type:
      name: const
      value_type: i8
      values:
        - name: Ok
          desc: Everything is ok
          value: 0
        - name: Error
          desc: Request is bad
          value: 1
    attributes:
      rs_extra_derive: Hash, PartialEq, Eq, PartialOrd, Ord
```

Rust side, will generate a NewType wraps i8, also with each values defined as const variables

```rust
pub struct Code(pub i8);

... from_value to_value

impl Code {
    /// Everything is ok
    pub const Ok: Code = Code(0);
    /// Request is bad
    pub const Error: Code = Code(1);
}
```
