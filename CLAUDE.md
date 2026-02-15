# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
# Build all workspace crates
cargo build

# Run all tests (IMPORTANT: use --workspace due to default-members config)
cargo test --workspace

# Run tests for specific crate
cargo test -p tot_spec
cargo test -p tot_spec_util
cargo test -p tot_spec_cli

# Update test fixtures (instead of asserting, writes expected output)
cargo test --workspace --features test_update_spec

# Run the codegen binary
cargo run -- -i <spec_folder> -c <generator> -o <output_path>

# Run with default generator (rs_serde)
cargo run -- -i examples/spec -o output/
```

## Project Architecture

**tot_spec** is a language-agnostic model definition utility that generates code from YAML specifications. It uses a Cargo workspace with three crates:

- **tot_spec_cli/**: CLI binary entry point, contains language-specific code generators
- **tot_spec/**: Core library with spec parsing, model definitions, and type resolution
- **tot_spec_util/**: Utility crate for special types (big integers, decimals)

### Core Types (tot_spec/src/models.rs)

- `Definition`: Root spec structure (models, includes, metadata)
- `ModelDef`: Model definitions with types: `struct`, `enum`, `virtual`, `new_type`, `const`
- `Type`: Supported types (primitives, list, map, ref, json)
- `FieldDef`: Field with type, description, and attributes

### Code Generators (tot_spec/src/codegen/)

Each generator implements the `Codegen` trait. Available generators:
- `rs_serde`: Rust serde structs/enums
- `java_jackson`: Java Jackson classes
- `swift_codable`: Swift Codable structures
- `py_dataclass`: Python dataclasses
- `swagger`: OpenAPI/Swagger spec

### Spec Structure

YAML specs contain:
- `meta`: Package names, descriptions for codegen
- `includes`: Import other specs with namespaces
- `models`: Array of model definitions

### Key Patterns

- **Virtual types**: Define shared fields that map to traits (Rust) or interfaces (Java/Swift)
- **New types**: Wrapper types for domain modeling
- **Const**: Enum-like constant definitions with explicit values
- **Includes**: Compose specs with namespace prefixes for type references

### Field Attributes

Language-specific customization via attributes:
```yaml
attributes:
  rs_type: std::collections::BTreeMap<String, String>
  rs_extra_derive: Hash, PartialEq
```
