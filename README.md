# Introduction

tot_spec is a language argonostic model definition util. It is mainly used to define json models cross boundaries, e.g, api server and client, sdk rust and swift etc.

# Examples

## A nested struct

```yaml
models:
- name: SimpleStruct
  type:
    name: struct
    fields:
    # basic types including bool, i8, i64, f64 and string
    - name: bool_value
      type:
        name: bool
      required: true
    - name: i8_value
      type:
        name: i8
      required: true
    - name: i64_value
      type:
        name: i64
    - name: string_value
      type:
        name: string
    - name: bytes_value
      type:
        name: bytes

    # also container types including Map and List
    - name: i8_to_string
      type:
        name: map
        key_type:
          name: i8
        value_type:
          name: string
      attributes:
        # use rs_type attibute to mark underlying type as BTreeMap
        rs_type: std::collections::BTreeMap::<i8, std::string::String>
    - name: key_values
      type:
        # reference other types defined in spec
        name: ref
        target: KeyValue
    - name: children
      type:
        name: list
        item_type:
          name: ref
          target: SimpleStruct

```

## New type

```yaml
- name: Container
  type:
    # type name as "new_type", which maps to rust's new type pattern
    # for other languages, this can be a no op
    name: new_type
    inner_type:
      name: list
      item_type:
        name: ref
        target: SimpleStruct
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
      type:
        name: string

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
pub trait Base {
    fn request_id(&self) -> &std::string::String;
}
```