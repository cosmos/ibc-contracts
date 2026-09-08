// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::idl::{Idl, IdlFieldType, IdlTypeDef, IdlVariantField};
use crate::util::{sanitize_ident, write_file_header, NameMap};

/// Generates `types.rs` containing non-account, non-event types from the IDL.
///
/// Account types (in `idl.accounts`) go to `accounts.rs` and event types
/// (in `idl.events`) go to `events.rs`.
pub fn generate_types(program: &str, idl: &Idl, program_dir: &Path, names: &NameMap) -> bool {
    let event_names: BTreeSet<&str> = idl.events.iter().map(|e| e.name.as_str()).collect();
    let account_names: BTreeSet<&str> = idl
        .accounts
        .iter()
        .filter(|a| a.name.starts_with(&format!("{program}::")))
        .map(|a| a.name.as_str())
        .collect();

    let mut regular_types: Vec<&IdlTypeDef> = idl
        .types
        .iter()
        .filter(|t| {
            !event_names.contains(t.name.as_str()) && !account_names.contains(t.name.as_str())
        })
        .collect();

    if regular_types.is_empty() {
        return false;
    }

    regular_types.sort_by_key(|t| resolve_name(names, &t.name));

    let mut output = String::new();
    write_file_header(&mut output);
    writeln!(
        output,
        "use anchor_lang::prelude::*;\n\
         use anchor_lang::solana_program::pubkey::Pubkey;\n"
    )
    .unwrap();

    for type_def in &regular_types {
        generate_type_def(&mut output, type_def, false, names);
    }

    let path = program_dir.join("types.rs");
    fs::write(&path, output).unwrap_or_else(|e| panic!("Failed to write {}: {e}", path.display()));

    true
}

/// Generates `events.rs` with event structs marked `#[event]`.
///
/// Events are sorted by resolved name for deterministic output.
pub fn generate_events(
    idl: &Idl,
    program_dir: &Path,
    has_types: bool,
    has_accounts: bool,
    names: &NameMap,
) -> bool {
    if idl.events.is_empty() {
        return false;
    }

    let type_map: HashMap<&str, &IdlTypeDef> =
        idl.types.iter().map(|t| (t.name.as_str(), t)).collect();

    let mut output = String::new();
    write_file_header(&mut output);
    writeln!(output, "use anchor_lang::prelude::*;").unwrap();
    if has_types {
        writeln!(output, "use super::types::*;").unwrap();
    }
    if has_accounts {
        writeln!(output, "use super::accounts::*;").unwrap();
    }
    writeln!(output).unwrap();

    let mut sorted_events: Vec<&_> = idl.events.iter().collect();
    sorted_events.sort_by_key(|e| resolve_name(names, &e.name));

    for event in sorted_events {
        if let Some(type_def) = type_map.get(event.name.as_str()) {
            generate_type_def(&mut output, type_def, true, names);
        }
    }

    let events_path = program_dir.join("events.rs");
    fs::write(&events_path, output)
        .unwrap_or_else(|e| panic!("Failed to write {}: {e}", events_path.display()));

    true
}

/// Generates `accounts.rs` containing account state types with discriminator impls.
///
/// Account types are identified by the `idl.accounts` section. Each account
/// gets its struct definition (from `idl.types`) followed by a discriminator
/// `impl` block. Sorted by resolved name for deterministic output.
pub fn generate_accounts(
    program: &str,
    idl: &Idl,
    program_dir: &Path,
    has_types: bool,
    names: &NameMap,
) -> bool {
    let mut owned: Vec<_> = idl
        .accounts
        .iter()
        .filter(|a| a.name.starts_with(&format!("{program}::")))
        .collect();

    if owned.is_empty() {
        return false;
    }

    owned.sort_by_key(|a| resolve_name(names, &a.name));

    let type_map: HashMap<&str, &IdlTypeDef> =
        idl.types.iter().map(|t| (t.name.as_str(), t)).collect();

    let mut output = String::new();
    write_file_header(&mut output);
    writeln!(
        output,
        "use anchor_lang::prelude::*;\n\
         use anchor_lang::solana_program::pubkey::Pubkey;\n"
    )
    .unwrap();
    if has_types {
        writeln!(output, "use super::types::*;\n").unwrap();
    }

    for acc in &owned {
        if let Some(type_def) = type_map.get(acc.name.as_str()) {
            generate_type_def(&mut output, type_def, false, names);
        }

        let type_name = resolve_name(names, &acc.name);
        let disc_bytes: Vec<String> = acc.discriminator.iter().map(ToString::to_string).collect();
        writeln!(output, "impl {type_name} {{").unwrap();
        writeln!(
            output,
            "    pub const DISCRIMINATOR: [u8; 8] = [{}];",
            disc_bytes.join(", ")
        )
        .unwrap();
        writeln!(output, "}}\n").unwrap();
    }

    let path = program_dir.join("accounts.rs");
    fs::write(&path, output).unwrap_or_else(|e| panic!("Failed to write {}: {e}", path.display()));

    true
}

pub fn generate_type_def(
    output: &mut String,
    type_def: &IdlTypeDef,
    is_event: bool,
    names: &NameMap,
) {
    for doc in &type_def.docs {
        for line in doc.lines() {
            writeln!(output, "/// {line}").unwrap();
        }
    }

    if is_event {
        writeln!(output, "#[derive(Clone, Debug)]").unwrap();
        writeln!(output, "#[event]").unwrap();
    } else {
        writeln!(
            output,
            "#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]"
        )
        .unwrap();
    }

    let type_name = resolve_name(names, &type_def.name);

    match type_def.type_def.kind.as_str() {
        "struct" => {
            writeln!(output, "pub struct {type_name} {{").unwrap();
            for field in &type_def.type_def.fields {
                for doc in &field.docs {
                    for line in doc.lines() {
                        writeln!(output, "    /// {line}").unwrap();
                    }
                }
                let rust_type = idl_type_to_rust(&field.field_type, names);
                let ident = sanitize_ident(field.name.as_deref().unwrap_or("_unnamed"));
                writeln!(output, "    pub {ident}: {rust_type},").unwrap();
            }
            writeln!(output, "}}\n").unwrap();
        }
        "enum" => {
            writeln!(output, "pub enum {type_name} {{").unwrap();
            for variant in &type_def.type_def.variants {
                if variant.fields.is_empty() {
                    writeln!(output, "    {},", variant.name).unwrap();
                } else {
                    let all_named = variant
                        .fields
                        .iter()
                        .all(|f| matches!(f, IdlVariantField::Named(_)));

                    if all_named {
                        writeln!(output, "    {} {{", variant.name).unwrap();
                        for vf in &variant.fields {
                            if let IdlVariantField::Named(field) = vf {
                                let rust_type = idl_type_to_rust(&field.field_type, names);
                                let ident =
                                    sanitize_ident(field.name.as_deref().unwrap_or("_unnamed"));
                                writeln!(output, "        {ident}: {rust_type},").unwrap();
                            }
                        }
                        writeln!(output, "    }},").unwrap();
                    } else {
                        let types: Vec<String> = variant
                            .fields
                            .iter()
                            .map(|vf| idl_type_to_rust(vf.field_type(), names))
                            .collect();
                        writeln!(output, "    {}({}),", variant.name, types.join(", ")).unwrap();
                    }
                }
            }
            writeln!(output, "}}\n").unwrap();
        }
        other => panic!("Unsupported IDL type kind: {other}"),
    }
}

/// Converts an IDL field type to its corresponding Rust type string.
///
/// Used by both `type_gen` for struct fields and `instruction_gen` for args structs.
pub fn idl_type_to_rust(field_type: &IdlFieldType, names: &NameMap) -> String {
    match field_type {
        IdlFieldType::Primitive(p) => match p.as_str() {
            "string" => "String".to_string(),
            "pubkey" => "Pubkey".to_string(),
            "bytes" => "Vec<u8>".to_string(),
            "bool" | "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64"
            | "i128" => p.clone(),
            other => panic!("Unknown IDL primitive type: {other}"),
        },
        IdlFieldType::Vec { vec } => {
            let inner = idl_type_to_rust(vec, names);
            format!("Vec<{inner}>")
        }
        IdlFieldType::Option { option } => {
            let inner = idl_type_to_rust(option, names);
            format!("Option<{inner}>")
        }
        IdlFieldType::Array {
            array: (inner, size),
        } => {
            let inner_type = idl_type_to_rust(inner, names);
            format!("[{inner_type}; {size}]")
        }
        IdlFieldType::Defined { defined } => resolve_name(names, &defined.name),
    }
}

/// Resolves a type name using the map, falling back to the raw name.
fn resolve_name(names: &NameMap, fq_name: &str) -> String {
    names
        .get(fq_name)
        .cloned()
        .unwrap_or_else(|| fq_name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idl::*;
    use crate::util::build_name_map;

    fn names_from(types: &[&str]) -> NameMap {
        let idl = Idl {
            instructions: vec![],
            events: vec![],
            types: types
                .iter()
                .map(|name| IdlTypeDef {
                    name: (*name).to_string(),
                    docs: vec![],
                    type_def: IdlTypeDefBody {
                        kind: "struct".into(),
                        fields: vec![],
                        variants: vec![],
                    },
                })
                .collect(),
            accounts: vec![],
        };
        build_name_map(&idl)
    }

    #[test]
    fn primitives() {
        let names = NameMap::new();
        assert_eq!(
            idl_type_to_rust(&IdlFieldType::Primitive("u64".to_string()), &names),
            "u64"
        );
        assert_eq!(
            idl_type_to_rust(&IdlFieldType::Primitive("string".to_string()), &names),
            "String"
        );
        assert_eq!(
            idl_type_to_rust(&IdlFieldType::Primitive("pubkey".to_string()), &names),
            "Pubkey"
        );
        assert_eq!(
            idl_type_to_rust(&IdlFieldType::Primitive("bytes".to_string()), &names),
            "Vec<u8>"
        );
    }

    #[test]
    fn all_integer_types() {
        let names = NameMap::new();
        for ty in [
            "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128",
        ] {
            assert_eq!(
                idl_type_to_rust(&IdlFieldType::Primitive(ty.to_string()), &names),
                ty
            );
        }
    }

    #[test]
    fn bool_type() {
        let names = NameMap::new();
        assert_eq!(
            idl_type_to_rust(&IdlFieldType::Primitive("bool".to_string()), &names),
            "bool"
        );
    }

    #[test]
    fn containers() {
        let names = NameMap::new();
        let vec_type = IdlFieldType::Vec {
            vec: Box::new(IdlFieldType::Primitive("bytes".to_string())),
        };
        assert_eq!(idl_type_to_rust(&vec_type, &names), "Vec<Vec<u8>>");

        let option_type = IdlFieldType::Option {
            option: Box::new(IdlFieldType::Primitive("string".to_string())),
        };
        assert_eq!(idl_type_to_rust(&option_type, &names), "Option<String>");

        let array_type = IdlFieldType::Array {
            array: (Box::new(IdlFieldType::Primitive("u8".to_string())), 32),
        };
        assert_eq!(idl_type_to_rust(&array_type, &names), "[u8; 32]");
    }

    #[test]
    fn nested_containers() {
        let names = NameMap::new();
        let nested = IdlFieldType::Vec {
            vec: Box::new(IdlFieldType::Option {
                option: Box::new(IdlFieldType::Array {
                    array: (Box::new(IdlFieldType::Primitive("u8".to_string())), 20),
                }),
            }),
        };
        assert_eq!(idl_type_to_rust(&nested, &names), "Vec<Option<[u8; 20]>>");
    }

    #[test]
    fn defined_type_uses_short_name() {
        let names = names_from(&["solana_ibc_types::router::Packet"]);
        let defined_type = IdlFieldType::Defined {
            defined: IdlDefinedRef {
                name: "solana_ibc_types::router::Packet".to_string(),
            },
        };
        assert_eq!(idl_type_to_rust(&defined_type, &names), "Packet");
    }

    #[test]
    fn defined_type_uses_fq_name_on_collision() {
        let names = names_from(&["mod_a::Shared", "mod_b::Shared"]);
        let defined_a = IdlFieldType::Defined {
            defined: IdlDefinedRef {
                name: "mod_a::Shared".to_string(),
            },
        };
        let defined_b = IdlFieldType::Defined {
            defined: IdlDefinedRef {
                name: "mod_b::Shared".to_string(),
            },
        };
        assert_eq!(idl_type_to_rust(&defined_a, &names), "ModA_Shared");
        assert_eq!(idl_type_to_rust(&defined_b, &names), "ModB_Shared");
    }

    #[test]
    #[should_panic(expected = "Unknown IDL primitive type: float64")]
    fn unknown_primitive_panics() {
        let names = NameMap::new();
        idl_type_to_rust(&IdlFieldType::Primitive("float64".to_string()), &names);
    }

    fn make_struct_type_def(name: &str, fields: Vec<(&str, IdlFieldType)>) -> IdlTypeDef {
        IdlTypeDef {
            name: name.to_string(),
            docs: vec![],
            type_def: IdlTypeDefBody {
                kind: "struct".to_string(),
                fields: fields
                    .into_iter()
                    .map(|(n, t)| IdlField {
                        name: Some(n.to_string()),
                        docs: vec![],
                        field_type: t,
                    })
                    .collect(),
                variants: vec![],
            },
        }
    }

    #[test]
    fn generate_struct_type() {
        let td = make_struct_type_def(
            "my_mod::MyStruct",
            vec![
                ("field_a", IdlFieldType::Primitive("u64".to_string())),
                ("field_b", IdlFieldType::Primitive("string".to_string())),
            ],
        );
        let names = names_from(&["my_mod::MyStruct"]);

        let mut output = String::new();
        generate_type_def(&mut output, &td, false, &names);

        assert!(output.contains("#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]"));
        assert!(output.contains("pub struct MyStruct {"));
        assert!(output.contains("pub field_a: u64,"));
        assert!(output.contains("pub field_b: String,"));
    }

    #[test]
    fn generate_event_type() {
        let td = make_struct_type_def(
            "events::MyEvent",
            vec![("data", IdlFieldType::Primitive("bytes".to_string()))],
        );
        let names = names_from(&["events::MyEvent"]);

        let mut output = String::new();
        generate_type_def(&mut output, &td, true, &names);

        assert!(output.contains("#[derive(Clone, Debug)]"));
        assert!(output.contains("#[event]"));
        assert!(output.contains("pub struct MyEvent {"));
        assert!(!output.contains("AnchorSerialize"));
    }

    #[test]
    fn generate_enum_with_unit_variants() {
        let td = IdlTypeDef {
            name: "Status".to_string(),
            docs: vec![],
            type_def: IdlTypeDefBody {
                kind: "enum".to_string(),
                fields: vec![],
                variants: vec![
                    IdlEnumVariant {
                        name: "Active".to_string(),
                        fields: vec![],
                    },
                    IdlEnumVariant {
                        name: "Frozen".to_string(),
                        fields: vec![],
                    },
                ],
            },
        };
        let names = names_from(&["Status"]);

        let mut output = String::new();
        generate_type_def(&mut output, &td, false, &names);

        assert!(output.contains("pub enum Status {"));
        assert!(output.contains("    Active,"));
        assert!(output.contains("    Frozen,"));
    }

    #[test]
    fn generate_enum_with_tuple_variant() {
        let td = IdlTypeDef {
            name: "Commitment".to_string(),
            docs: vec![],
            type_def: IdlTypeDefBody {
                kind: "enum".to_string(),
                fields: vec![],
                variants: vec![IdlEnumVariant {
                    name: "Ack".to_string(),
                    fields: vec![IdlVariantField::Unnamed(IdlFieldType::Array {
                        array: (Box::new(IdlFieldType::Primitive("u8".to_string())), 32),
                    })],
                }],
            },
        };
        let names = names_from(&["Commitment"]);

        let mut output = String::new();
        generate_type_def(&mut output, &td, false, &names);

        assert!(output.contains("    Ack([u8; 32]),"));
    }

    #[test]
    fn generate_enum_with_named_variant() {
        let td = IdlTypeDef {
            name: "Action".to_string(),
            docs: vec![],
            type_def: IdlTypeDefBody {
                kind: "enum".to_string(),
                fields: vec![],
                variants: vec![IdlEnumVariant {
                    name: "Transfer".to_string(),
                    fields: vec![IdlVariantField::Named(IdlField {
                        name: Some("amount".to_string()),
                        docs: vec![],
                        field_type: IdlFieldType::Primitive("u64".to_string()),
                    })],
                }],
            },
        };
        let names = names_from(&["Action"]);

        let mut output = String::new();
        generate_type_def(&mut output, &td, false, &names);

        assert!(output.contains("    Transfer {"));
        assert!(output.contains("        amount: u64,"));
    }

    #[test]
    fn generate_struct_with_docs() {
        let td = IdlTypeDef {
            name: "Documented".to_string(),
            docs: vec!["A documented struct.".to_string()],
            type_def: IdlTypeDefBody {
                kind: "struct".to_string(),
                fields: vec![IdlField {
                    name: Some("x".to_string()),
                    docs: vec!["The x field.".to_string()],
                    field_type: IdlFieldType::Primitive("u32".to_string()),
                }],
                variants: vec![],
            },
        };
        let names = names_from(&["Documented"]);

        let mut output = String::new();
        generate_type_def(&mut output, &td, false, &names);

        assert!(output.contains("/// A documented struct."));
        assert!(output.contains("/// The x field."));
    }

    #[test]
    fn multiline_docs_get_proper_prefix() {
        let td = IdlTypeDef {
            name: "MultiDoc".to_string(),
            docs: vec!["First line.\nSecond line.\nThird line.".to_string()],
            type_def: IdlTypeDefBody {
                kind: "struct".to_string(),
                fields: vec![IdlField {
                    name: Some("x".to_string()),
                    docs: vec!["Field first.\nField second.".to_string()],
                    field_type: IdlFieldType::Primitive("u8".to_string()),
                }],
                variants: vec![],
            },
        };
        let names = names_from(&["MultiDoc"]);

        let mut output = String::new();
        generate_type_def(&mut output, &td, false, &names);

        assert!(output.contains("/// First line.\n/// Second line.\n/// Third line.\n"));
        assert!(output.contains("    /// Field first.\n    /// Field second.\n"));
    }

    #[test]
    #[should_panic(expected = "Unsupported IDL type kind: union")]
    fn unsupported_type_kind_panics() {
        let td = IdlTypeDef {
            name: "Bad".to_string(),
            docs: vec![],
            type_def: IdlTypeDefBody {
                kind: "union".to_string(),
                fields: vec![],
                variants: vec![],
            },
        };
        let names = names_from(&["Bad"]);
        let mut output = String::new();
        generate_type_def(&mut output, &td, false, &names);
    }

    #[test]
    fn keyword_field_name_sanitized() {
        let td = make_struct_type_def(
            "HasKeyword",
            vec![("type", IdlFieldType::Primitive("string".to_string()))],
        );
        let names = names_from(&["HasKeyword"]);

        let mut output = String::new();
        generate_type_def(&mut output, &td, false, &names);
        assert!(output.contains("pub r#type: String,"));
    }

    #[test]
    fn keyword_enum_variant_field_sanitized() {
        let td = IdlTypeDef {
            name: "EnumWithKeyword".to_string(),
            docs: vec![],
            type_def: IdlTypeDefBody {
                kind: "enum".to_string(),
                fields: vec![],
                variants: vec![IdlEnumVariant {
                    name: "Variant".to_string(),
                    fields: vec![IdlVariantField::Named(IdlField {
                        name: Some("type".to_string()),
                        docs: vec![],
                        field_type: IdlFieldType::Primitive("u8".to_string()),
                    })],
                }],
            },
        };
        let names = names_from(&["EnumWithKeyword"]);

        let mut output = String::new();
        generate_type_def(&mut output, &td, false, &names);
        assert!(output.contains("r#type: u8,"));
    }

    #[test]
    fn unnamed_field_fallback() {
        let td = IdlTypeDef {
            name: "Anon".to_string(),
            docs: vec![],
            type_def: IdlTypeDefBody {
                kind: "struct".to_string(),
                fields: vec![IdlField {
                    name: None,
                    docs: vec![],
                    field_type: IdlFieldType::Primitive("u8".to_string()),
                }],
                variants: vec![],
            },
        };
        let names = names_from(&["Anon"]);

        let mut output = String::new();
        generate_type_def(&mut output, &td, false, &names);
        assert!(output.contains("pub _unnamed: u8,"));
    }

    #[test]
    fn generate_types_uses_fq_names_on_collision() {
        let dir = std::env::temp_dir().join("codegen_test_collision_names");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let idl = Idl {
            instructions: vec![],
            events: vec![],
            types: vec![
                make_struct_type_def(
                    "mod_a::Shared",
                    vec![("x", IdlFieldType::Primitive("u64".to_string()))],
                ),
                make_struct_type_def(
                    "mod_b::Shared",
                    vec![("y", IdlFieldType::Primitive("u32".to_string()))],
                ),
            ],
            accounts: vec![],
        };
        let names = build_name_map(&idl);

        assert!(generate_types("test", &idl, &dir, &names));
        let content = std::fs::read_to_string(dir.join("types.rs")).unwrap();

        assert!(content.contains("pub struct ModA_Shared {"));
        assert!(content.contains("pub struct ModB_Shared {"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn generate_types_uses_short_names_when_unique() {
        let dir = std::env::temp_dir().join("codegen_test_short_names");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let idl = Idl {
            instructions: vec![],
            events: vec![],
            types: vec![
                make_struct_type_def(
                    "mod_a::Foo",
                    vec![("x", IdlFieldType::Primitive("u64".to_string()))],
                ),
                make_struct_type_def(
                    "mod_b::Bar",
                    vec![("y", IdlFieldType::Primitive("u32".to_string()))],
                ),
            ],
            accounts: vec![],
        };
        let names = build_name_map(&idl);

        assert!(generate_types("test", &idl, &dir, &names));
        let content = std::fs::read_to_string(dir.join("types.rs")).unwrap();

        assert!(content.contains("pub struct Foo {"));
        assert!(content.contains("pub struct Bar {"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn generate_types_returns_false_for_empty() {
        let dir = std::env::temp_dir().join("codegen_test_empty_types");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let idl = Idl {
            instructions: vec![],
            events: vec![],
            types: vec![],
            accounts: vec![],
        };
        let names = build_name_map(&idl);

        assert!(!generate_types("test", &idl, &dir, &names));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn generate_types_excludes_events_and_accounts() {
        let dir = std::env::temp_dir().join("codegen_test_exclude");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let idl = Idl {
            instructions: vec![],
            events: vec![IdlEvent {
                name: "prog::MyEvent".to_string(),
                discriminator: vec![0; 8],
            }],
            types: vec![
                make_struct_type_def(
                    "prog::MyEvent",
                    vec![("data", IdlFieldType::Primitive("u64".to_string()))],
                ),
                make_struct_type_def(
                    "prog::MyAccount",
                    vec![("value", IdlFieldType::Primitive("u32".to_string()))],
                ),
                make_struct_type_def(
                    "prog::RegularType",
                    vec![("x", IdlFieldType::Primitive("u8".to_string()))],
                ),
            ],
            accounts: vec![IdlAccountDef {
                name: "prog::MyAccount".to_string(),
                discriminator: vec![1; 8],
            }],
        };
        let names = build_name_map(&idl);

        assert!(generate_types("prog", &idl, &dir, &names));
        let content = std::fs::read_to_string(dir.join("types.rs")).unwrap();

        assert!(content.contains("pub struct RegularType {"));
        assert!(!content.contains("MyEvent"));
        assert!(!content.contains("MyAccount"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn generate_types_sorted_by_name() {
        let dir = std::env::temp_dir().join("codegen_test_sorted");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let idl = Idl {
            instructions: vec![],
            events: vec![],
            types: vec![
                make_struct_type_def(
                    "Zebra",
                    vec![("z", IdlFieldType::Primitive("u8".to_string()))],
                ),
                make_struct_type_def(
                    "Apple",
                    vec![("a", IdlFieldType::Primitive("u8".to_string()))],
                ),
            ],
            accounts: vec![],
        };
        let names = build_name_map(&idl);

        assert!(generate_types("test", &idl, &dir, &names));
        let content = std::fs::read_to_string(dir.join("types.rs")).unwrap();

        let apple_pos = content.find("pub struct Apple {").unwrap();
        let zebra_pos = content.find("pub struct Zebra {").unwrap();
        assert!(
            apple_pos < zebra_pos,
            "Types should be sorted alphabetically"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn generate_accounts_with_discriminators() {
        let dir = std::env::temp_dir().join("codegen_test_accounts");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let idl = Idl {
            instructions: vec![],
            events: vec![],
            types: vec![make_struct_type_def(
                "prog::MyState",
                vec![("value", IdlFieldType::Primitive("u64".to_string()))],
            )],
            accounts: vec![IdlAccountDef {
                name: "prog::MyState".to_string(),
                discriminator: vec![10, 20, 30, 40, 50, 60, 70, 80],
            }],
        };
        let names = build_name_map(&idl);

        assert!(generate_accounts("prog", &idl, &dir, false, &names));
        let content = std::fs::read_to_string(dir.join("accounts.rs")).unwrap();

        assert!(content.contains("pub struct MyState {"));
        assert!(content.contains("pub value: u64,"));
        assert!(content.contains("impl MyState {"));
        assert!(content
            .contains("pub const DISCRIMINATOR: [u8; 8] = [10, 20, 30, 40, 50, 60, 70, 80];"));
        assert!(!content.contains("use super::types::*;"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn generate_accounts_imports_types_when_available() {
        let dir = std::env::temp_dir().join("codegen_test_accounts_import");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let idl = Idl {
            instructions: vec![],
            events: vec![],
            types: vec![make_struct_type_def(
                "prog::MyState",
                vec![("v", IdlFieldType::Primitive("u8".to_string()))],
            )],
            accounts: vec![IdlAccountDef {
                name: "prog::MyState".to_string(),
                discriminator: vec![0; 8],
            }],
        };
        let names = build_name_map(&idl);

        assert!(generate_accounts("prog", &idl, &dir, true, &names));
        let content = std::fs::read_to_string(dir.join("accounts.rs")).unwrap();
        assert!(content.contains("use super::types::*;"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn generate_accounts_returns_false_for_empty() {
        let dir = std::env::temp_dir().join("codegen_test_empty_accounts");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let idl = Idl {
            instructions: vec![],
            events: vec![],
            types: vec![],
            accounts: vec![],
        };
        let names = build_name_map(&idl);

        assert!(!generate_accounts("test", &idl, &dir, false, &names));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn generate_events_returns_false_for_empty() {
        let dir = std::env::temp_dir().join("codegen_test_empty_events");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let idl = Idl {
            instructions: vec![],
            events: vec![],
            types: vec![],
            accounts: vec![],
        };
        let names = build_name_map(&idl);

        assert!(!generate_events(&idl, &dir, false, false, &names));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn generate_events_skips_unmatched_event() {
        let dir = std::env::temp_dir().join("codegen_test_unmatched_event");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let idl = Idl {
            instructions: vec![],
            events: vec![IdlEvent {
                name: "NonExistentType".to_string(),
                discriminator: vec![0; 8],
            }],
            types: vec![make_struct_type_def(
                "SomeOther",
                vec![("x", IdlFieldType::Primitive("u8".to_string()))],
            )],
            accounts: vec![],
        };
        let names = build_name_map(&idl);

        assert!(generate_events(&idl, &dir, true, false, &names));
        let content = std::fs::read_to_string(dir.join("events.rs")).unwrap();
        assert!(!content.contains("pub struct"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn generate_events_not_in_types() {
        let dir = std::env::temp_dir().join("codegen_test_events_not_in_types");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let idl = Idl {
            instructions: vec![],
            events: vec![IdlEvent {
                name: "prog::MyEvent".to_string(),
                discriminator: vec![0; 8],
            }],
            types: vec![
                make_struct_type_def(
                    "prog::MyEvent",
                    vec![("data", IdlFieldType::Primitive("u64".to_string()))],
                ),
                make_struct_type_def(
                    "prog::RegularType",
                    vec![("x", IdlFieldType::Primitive("u8".to_string()))],
                ),
            ],
            accounts: vec![],
        };
        let names = build_name_map(&idl);

        assert!(generate_events(&idl, &dir, true, false, &names));
        let events_content = std::fs::read_to_string(dir.join("events.rs")).unwrap();
        assert!(events_content.contains("#[event]"));
        assert!(events_content.contains("pub struct MyEvent {"));
        assert!(!events_content.contains("RegularType"));

        assert!(generate_types("prog", &idl, &dir, &names));
        let types_content = std::fs::read_to_string(dir.join("types.rs")).unwrap();
        assert!(types_content.contains("pub struct RegularType {"));
        assert!(!types_content.contains("MyEvent"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
