// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;

use crate::idl::{IdlFieldType, IdlInstructionAccount, IdlInstructionArg, IdlPdaSeed, IdlTypeDef};
use crate::util::sanitize_ident;

/// A resolved PDA pattern with typed parameters.
pub struct ResolvedPda {
    pub const_seeds: Vec<Vec<u8>>,
    pub params: Vec<PdaParam>,
    /// Fixed outer program for `find_program_address` (from `pda.program` in IDL).
    ///
    /// When `Some`, the generated helper uses this constant instead of a
    /// `program_id` parameter. This is needed for PDAs owned by a different
    /// program (e.g. `program_data` is owned by `bpf_loader_upgradeable`).
    pub outer_program: Option<Vec<u8>>,
}

pub struct PdaParam {
    pub name: String,
    pub rust_type: String,
    pub seed_expr: String,
}

/// Resolves a single instruction account's PDA definition.
///
/// Returns `None` for accounts without PDA seeds or with no const seeds.
/// Cross-program PDAs and dot-path account seeds are resolved normally —
/// the caller decides whether to auto-derive in `build()` or just emit
/// a standalone helper.
pub fn resolve_account_pda(
    acc: &IdlInstructionAccount,
    ix_args: &[IdlInstructionArg],
    type_map: &HashMap<&str, &IdlTypeDef>,
) -> Option<ResolvedPda> {
    let pda = acc.pda.as_ref()?;

    if pda.seeds.is_empty() {
        return None;
    }

    let const_seeds: Vec<Vec<u8>> = pda
        .seeds
        .iter()
        .filter(|s| s.kind == "const")
        .filter_map(|s| s.value.clone())
        .collect();

    if const_seeds.is_empty() {
        return None;
    }

    let params = resolve_pda_params(&pda.seeds, ix_args, type_map);
    let outer_program = extract_const_program(pda.program.as_ref());

    Some(ResolvedPda {
        const_seeds,
        params,
        outer_program,
    })
}

/// Extracts a const program address from the IDL `pda.program` field.
///
/// Returns `Some(bytes)` when `program` is `{"kind": "const", "value": [...]}`,
/// `None` for dynamic programs or absent fields.
fn extract_const_program(program: Option<&serde_json::Value>) -> Option<Vec<u8>> {
    let val = program?;
    if val.get("kind")?.as_str()? != "const" {
        return None;
    }
    val.get("value")?
        .as_array()?
        .iter()
        .map(|v| v.as_u64().map(|n| n as u8))
        .collect()
}

/// BPF Loader Upgradeable program ID bytes.
const BPF_LOADER_UPGRADEABLE_BYTES: [u8; 32] = [
    2, 168, 246, 145, 78, 136, 161, 176, 226, 16, 21, 62, 247, 99, 174, 43, 0, 194, 185, 61, 22,
    193, 36, 210, 192, 83, 122, 16, 4, 128, 0, 0,
];

/// Maps const program bytes to a Rust expression for `find_program_address`.
fn const_program_expr(bytes: &[u8]) -> String {
    if bytes == BPF_LOADER_UPGRADEABLE_BYTES {
        "anchor_lang::solana_program::bpf_loader_upgradeable::ID".to_string()
    } else {
        let b: Vec<String> = bytes.iter().map(ToString::to_string).collect();
        format!("Pubkey::new_from_array([{}])", b.join(", "))
    }
}

/// Generates a PDA associated method on an instruction struct.
///
/// When the PDA has a fixed outer program (e.g. `bpf_loader_upgradeable`),
/// the generated method omits the `program_id` parameter and uses the
/// const program directly.
pub fn generate_pda_method(output: &mut String, account_name: &str, resolved: &ResolvedPda) {
    let mut param_strs: Vec<String> = resolved
        .params
        .iter()
        .map(|p| format!("{}: {}", sanitize_ident(&p.name), p.rust_type))
        .collect();

    if resolved.outer_program.is_none() {
        param_strs.push("program_id: &Pubkey".to_string());
    }

    let method_name = format!("{}_pda", sanitize_ident(account_name));
    writeln!(output, "    #[must_use]").unwrap();
    writeln!(
        output,
        "    pub fn {method_name}({}) -> (Pubkey, u8) {{",
        param_strs.join(", ")
    )
    .unwrap();
    writeln!(output, "        Pubkey::find_program_address(").unwrap();
    writeln!(output, "            &[").unwrap();

    for seed in &build_seed_exprs(resolved) {
        writeln!(output, "                {seed},").unwrap();
    }

    writeln!(output, "            ],").unwrap();
    if let Some(ref prog_bytes) = resolved.outer_program {
        let expr = const_program_expr(prog_bytes);
        writeln!(output, "            &{expr},").unwrap();
    } else {
        writeln!(output, "            program_id,").unwrap();
    }
    writeln!(output, "        )").unwrap();
    writeln!(output, "    }}\n").unwrap();
}

/// Resolves PDA seed parameters to typed Rust function parameters.
pub fn resolve_pda_params(
    seeds: &[IdlPdaSeed],
    ix_args: &[IdlInstructionArg],
    type_map: &HashMap<&str, &IdlTypeDef>,
) -> Vec<PdaParam> {
    let mut params = Vec::new();
    let mut seen_names = BTreeSet::new();

    for seed in seeds {
        if seed.kind == "const" {
            continue;
        }

        let path = seed.path.as_deref().unwrap_or("");
        let leaf = path.rsplit('.').next().unwrap_or(path);

        if !seen_names.insert(leaf.to_string()) {
            continue;
        }

        if seed.kind == "account" {
            params.push(PdaParam {
                name: leaf.to_string(),
                rust_type: "&Pubkey".to_string(),
                seed_expr: format!("{leaf}.as_ref()"),
            });
        } else if seed.kind == "arg" {
            let arg_type = resolve_arg_type(path, ix_args, type_map);
            let (rust_type, seed_expr) = arg_type_to_param(&arg_type, leaf);
            params.push(PdaParam {
                name: leaf.to_string(),
                rust_type,
                seed_expr,
            });
        }
    }

    params
}

/// Resolves the IDL field type of an arg path (handles nested paths like `msg.client_id`).
fn resolve_arg_type(
    path: &str,
    ix_args: &[IdlInstructionArg],
    type_map: &HashMap<&str, &IdlTypeDef>,
) -> IdlFieldType {
    let parts: Vec<&str> = path.split('.').collect();

    let root = parts[0];
    let Some(root_arg) = ix_args.iter().find(|a| a.name == root) else {
        return IdlFieldType::Primitive("bytes".to_string());
    };

    if parts.len() == 1 {
        return root_arg.arg_type.clone();
    }

    let mut current_type = &root_arg.arg_type;
    for &part in &parts[1..] {
        match current_type {
            IdlFieldType::Defined { defined } => {
                if let Some(type_def) = type_map.get(defined.name.as_str()) {
                    if let Some(field) = type_def
                        .type_def
                        .fields
                        .iter()
                        .find(|f| f.name.as_deref() == Some(part))
                    {
                        current_type = &field.field_type;
                    } else {
                        return IdlFieldType::Primitive("bytes".to_string());
                    }
                } else {
                    return IdlFieldType::Primitive("bytes".to_string());
                }
            }
            _ => return IdlFieldType::Primitive("bytes".to_string()),
        }
    }

    current_type.clone()
}

/// Maps an IDL arg type to a Rust function parameter type and seed expression.
fn arg_type_to_param(arg_type: &IdlFieldType, name: &str) -> (String, String) {
    match arg_type {
        IdlFieldType::Primitive(p) => match p.as_str() {
            "string" => ("&str".to_string(), format!("{name}.as_bytes()")),
            "pubkey" => ("&Pubkey".to_string(), format!("{name}.as_ref()")),
            "u64" => ("u64".to_string(), format!("&{name}.to_le_bytes()")),
            "u32" => ("u32".to_string(), format!("&{name}.to_le_bytes()")),
            "u16" => ("u16".to_string(), format!("&{name}.to_le_bytes()")),
            "u8" => ("u8".to_string(), format!("&[{name}]")),
            "i64" => ("i64".to_string(), format!("&{name}.to_le_bytes()")),
            "bool" => ("bool".to_string(), format!("&[u8::from({name})]")),
            _ => ("&[u8]".to_string(), name.to_string()),
        },
        _ => ("&[u8]".to_string(), name.to_string()),
    }
}

/// Builds ordered seed expressions: const seeds first, then dynamic params.
pub fn build_seed_exprs(resolved: &ResolvedPda) -> Vec<String> {
    let mut exprs = Vec::new();

    for cs in &resolved.const_seeds {
        if let Ok(s) = std::str::from_utf8(cs) {
            exprs.push(format!("b\"{s}\""));
        } else {
            let bytes: Vec<String> = cs.iter().map(ToString::to_string).collect();
            exprs.push(format!("&[{}]", bytes.join(", ")));
        }
    }

    for p in &resolved.params {
        exprs.push(p.seed_expr.clone());
    }

    exprs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idl::*;

    fn const_seed(value: &[u8]) -> IdlPdaSeed {
        IdlPdaSeed {
            kind: "const".to_string(),
            value: Some(value.to_vec()),
            path: None,
        }
    }

    fn account_seed(path: &str) -> IdlPdaSeed {
        IdlPdaSeed {
            kind: "account".to_string(),
            value: None,
            path: Some(path.to_string()),
        }
    }

    fn arg_seed(path: &str) -> IdlPdaSeed {
        IdlPdaSeed {
            kind: "arg".to_string(),
            value: None,
            path: Some(path.to_string()),
        }
    }

    fn pda_account(name: &str, seeds: Vec<IdlPdaSeed>) -> IdlInstructionAccount {
        IdlInstructionAccount {
            name: name.to_string(),
            writable: false,
            signer: false,
            address: None,
            pda: Some(IdlPda {
                seeds,
                program: None,
            }),
        }
    }

    #[test]
    fn resolve_account_pda_simple_const() {
        let acc = pda_account("my_pda", vec![const_seed(b"access_manager")]);
        let type_map = HashMap::new();

        let resolved = resolve_account_pda(&acc, &[], &type_map).unwrap();
        assert_eq!(resolved.const_seeds.len(), 1);
        assert_eq!(resolved.params.len(), 0);
    }

    #[test]
    fn resolve_account_pda_with_account_and_arg_seeds() {
        let acc = pda_account(
            "channel",
            vec![
                const_seed(b"channel"),
                account_seed("owner"),
                arg_seed("channel_id"),
            ],
        );
        let ix_args = vec![IdlInstructionArg {
            name: "channel_id".to_string(),
            arg_type: IdlFieldType::Primitive("string".to_string()),
        }];
        let type_map = HashMap::new();

        let resolved = resolve_account_pda(&acc, &ix_args, &type_map).unwrap();
        assert_eq!(resolved.params.len(), 2);
        assert_eq!(resolved.params[0].name, "owner");
        assert_eq!(resolved.params[0].rust_type, "&Pubkey");
        assert_eq!(resolved.params[1].name, "channel_id");
        assert_eq!(resolved.params[1].rust_type, "&str");
        assert_eq!(resolved.params[1].seed_expr, "channel_id.as_bytes()");
    }

    #[test]
    fn resolve_account_pda_handles_cross_program() {
        let acc = IdlInstructionAccount {
            name: "external_pda".to_string(),
            writable: false,
            signer: false,
            address: None,
            pda: Some(IdlPda {
                seeds: vec![const_seed(b"gmp_result"), arg_seed("client_id")],
                program: Some(serde_json::json!({"kind": "account", "path": "external"})),
            }),
        };
        let ix_args = vec![IdlInstructionArg {
            name: "client_id".to_string(),
            arg_type: IdlFieldType::Primitive("string".to_string()),
        }];
        let type_map = HashMap::new();

        let resolved = resolve_account_pda(&acc, &ix_args, &type_map).unwrap();
        assert_eq!(resolved.const_seeds.len(), 1);
        assert_eq!(resolved.const_seeds[0], b"gmp_result");
        assert_eq!(resolved.params.len(), 1);
        assert_eq!(resolved.params[0].name, "client_id");
        assert_eq!(resolved.params[0].rust_type, "&str");
    }

    #[test]
    fn resolve_account_pda_returns_none_for_no_pda() {
        let acc = IdlInstructionAccount {
            name: "payer".to_string(),
            writable: true,
            signer: true,
            address: None,
            pda: None,
        };
        let type_map = HashMap::new();

        assert!(resolve_account_pda(&acc, &[], &type_map).is_none());
    }

    #[test]
    fn resolve_account_pda_returns_none_for_empty_seeds() {
        let acc = IdlInstructionAccount {
            name: "empty".to_string(),
            writable: false,
            signer: false,
            address: None,
            pda: Some(IdlPda {
                seeds: vec![],
                program: None,
            }),
        };
        let type_map = HashMap::new();

        assert!(resolve_account_pda(&acc, &[], &type_map).is_none());
    }

    #[test]
    fn resolve_account_pda_returns_none_for_no_const_seeds() {
        let acc = pda_account("dynamic", vec![account_seed("owner"), arg_seed("id")]);
        let type_map = HashMap::new();

        assert!(resolve_account_pda(&acc, &[], &type_map).is_none());
    }

    #[test]
    fn build_seed_exprs_utf8_and_binary() {
        let resolved = ResolvedPda {
            const_seeds: vec![b"channel".to_vec(), vec![0xFF, 0x01]],
            params: vec![PdaParam {
                name: "owner".to_string(),
                rust_type: "&Pubkey".to_string(),
                seed_expr: "owner.as_ref()".to_string(),
            }],
            outer_program: None,
        };

        let exprs = build_seed_exprs(&resolved);
        assert_eq!(exprs.len(), 3);
        assert_eq!(exprs[0], "b\"channel\"");
        assert_eq!(exprs[1], "&[255, 1]");
        assert_eq!(exprs[2], "owner.as_ref()");
    }

    #[test]
    fn generate_pda_method_output() {
        let resolved = ResolvedPda {
            const_seeds: vec![b"state".to_vec()],
            params: vec![PdaParam {
                name: "client_id".to_string(),
                rust_type: "&str".to_string(),
                seed_expr: "client_id.as_bytes()".to_string(),
            }],
            outer_program: None,
        };

        let mut output = String::new();
        generate_pda_method(&mut output, "my_state", &resolved);

        assert!(output
            .contains("pub fn my_state_pda(client_id: &str, program_id: &Pubkey) -> (Pubkey, u8)"));
        assert!(output.contains("b\"state\""));
        assert!(output.contains("client_id.as_bytes()"));
        assert!(output.contains("#[must_use]"));
    }

    #[test]
    fn generate_pda_method_no_params() {
        let resolved = ResolvedPda {
            const_seeds: vec![b"access_manager".to_vec()],
            params: vec![],
            outer_program: None,
        };

        let mut output = String::new();
        generate_pda_method(&mut output, "access_manager", &resolved);

        assert!(output.contains("pub fn access_manager_pda(program_id: &Pubkey) -> (Pubkey, u8)"));
        assert!(output.contains("b\"access_manager\""));
    }

    #[test]
    fn generate_pda_method_sanitizes_keywords() {
        let resolved = ResolvedPda {
            const_seeds: vec![b"type".to_vec()],
            params: vec![PdaParam {
                name: "match".to_string(),
                rust_type: "&str".to_string(),
                seed_expr: "match.as_bytes()".to_string(),
            }],
            outer_program: None,
        };

        let mut output = String::new();
        generate_pda_method(&mut output, "type", &resolved);

        assert!(output.contains("pub fn r#type_pda(r#match: &str, program_id: &Pubkey)"));
    }

    #[test]
    fn arg_type_to_param_all_primitives() {
        let cases = [
            ("string", "&str", "x.as_bytes()"),
            ("pubkey", "&Pubkey", "x.as_ref()"),
            ("u64", "u64", "&x.to_le_bytes()"),
            ("u32", "u32", "&x.to_le_bytes()"),
            ("u16", "u16", "&x.to_le_bytes()"),
            ("u8", "u8", "&[x]"),
            ("i64", "i64", "&x.to_le_bytes()"),
            ("bool", "bool", "&[u8::from(x)]"),
        ];

        for (idl_type, expected_rust, expected_expr) in cases {
            let ft = IdlFieldType::Primitive(idl_type.to_string());
            let (rust_type, seed_expr) = arg_type_to_param(&ft, "x");
            assert_eq!(rust_type, expected_rust, "rust_type for {idl_type}");
            assert_eq!(seed_expr, expected_expr, "seed_expr for {idl_type}");
        }
    }

    #[test]
    fn arg_type_to_param_non_primitive_fallback() {
        let ft = IdlFieldType::Defined {
            defined: IdlDefinedRef {
                name: "SomeStruct".to_string(),
            },
        };
        let (rust_type, seed_expr) = arg_type_to_param(&ft, "data");
        assert_eq!(rust_type, "&[u8]");
        assert_eq!(seed_expr, "data");
    }

    #[test]
    fn resolve_arg_type_nested_path() {
        let inner_type_def = IdlTypeDef {
            name: "MyMsg".to_string(),
            docs: vec![],
            type_def: IdlTypeDefBody {
                kind: "struct".to_string(),
                fields: vec![IdlField {
                    name: Some("client_id".to_string()),
                    docs: vec![],
                    field_type: IdlFieldType::Primitive("string".to_string()),
                }],
                variants: vec![],
            },
        };

        let ix_args = vec![IdlInstructionArg {
            name: "msg".to_string(),
            arg_type: IdlFieldType::Defined {
                defined: IdlDefinedRef {
                    name: "MyMsg".to_string(),
                },
            },
        }];

        let type_map: HashMap<&str, &IdlTypeDef> = [("MyMsg", &inner_type_def)].into();

        let result = resolve_arg_type("msg.client_id", &ix_args, &type_map);
        assert!(matches!(result, IdlFieldType::Primitive(ref p) if p == "string"));
    }

    #[test]
    fn resolve_arg_type_missing_path_returns_bytes() {
        let ix_args = vec![IdlInstructionArg {
            name: "msg".to_string(),
            arg_type: IdlFieldType::Primitive("u64".to_string()),
        }];
        let type_map: HashMap<&str, &IdlTypeDef> = HashMap::new();

        let result = resolve_arg_type("msg.nonexistent", &ix_args, &type_map);
        assert!(matches!(result, IdlFieldType::Primitive(ref p) if p == "bytes"));
    }

    #[test]
    fn resolve_arg_type_unknown_root_returns_bytes() {
        let ix_args = vec![];
        let type_map: HashMap<&str, &IdlTypeDef> = HashMap::new();

        let result = resolve_arg_type("unknown_arg", &ix_args, &type_map);
        assert!(matches!(result, IdlFieldType::Primitive(ref p) if p == "bytes"));
    }

    #[test]
    fn resolve_arg_type_defined_type_not_in_map_returns_bytes() {
        let ix_args = vec![IdlInstructionArg {
            name: "msg".to_string(),
            arg_type: IdlFieldType::Defined {
                defined: IdlDefinedRef {
                    name: "UnknownType".to_string(),
                },
            },
        }];
        let type_map: HashMap<&str, &IdlTypeDef> = HashMap::new();

        let result = resolve_arg_type("msg.field", &ix_args, &type_map);
        assert!(matches!(result, IdlFieldType::Primitive(ref p) if p == "bytes"));
    }

    #[test]
    fn resolve_pda_params_deduplicates_leaf_names() {
        let seeds = vec![
            const_seed(b"pfx"),
            account_seed("owner"),
            arg_seed("some.path.owner"),
        ];
        let ix_args = vec![];
        let type_map: HashMap<&str, &IdlTypeDef> = HashMap::new();

        let params = resolve_pda_params(&seeds, &ix_args, &type_map);
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "owner");
    }

    #[test]
    fn extract_const_program_returns_bytes_for_const_kind() {
        let program = Some(serde_json::json!({
            "kind": "const",
            "value": BPF_LOADER_UPGRADEABLE_BYTES.to_vec(),
        }));
        let result = extract_const_program(program.as_ref());
        assert_eq!(result, Some(BPF_LOADER_UPGRADEABLE_BYTES.to_vec()));
    }

    #[test]
    fn extract_const_program_returns_none_for_account_kind() {
        let program = Some(serde_json::json!({
            "kind": "account",
            "path": "external",
        }));
        assert!(extract_const_program(program.as_ref()).is_none());
    }

    #[test]
    fn extract_const_program_returns_none_for_absent() {
        assert!(extract_const_program(None).is_none());
    }

    #[test]
    fn resolve_account_pda_captures_const_outer_program() {
        let acc = IdlInstructionAccount {
            name: "program_data".to_string(),
            writable: false,
            signer: false,
            address: None,
            pda: Some(IdlPda {
                seeds: vec![const_seed(&[1; 32])],
                program: Some(serde_json::json!({
                    "kind": "const",
                    "value": BPF_LOADER_UPGRADEABLE_BYTES.to_vec(),
                })),
            }),
        };
        let type_map = HashMap::new();

        let resolved = resolve_account_pda(&acc, &[], &type_map).unwrap();
        assert_eq!(
            resolved.outer_program,
            Some(BPF_LOADER_UPGRADEABLE_BYTES.to_vec())
        );
    }

    #[test]
    fn resolve_account_pda_no_outer_program_for_normal_pda() {
        let acc = pda_account("state", vec![const_seed(b"state")]);
        let type_map = HashMap::new();

        let resolved = resolve_account_pda(&acc, &[], &type_map).unwrap();
        assert!(resolved.outer_program.is_none());
    }

    #[test]
    fn generate_pda_method_with_const_outer_program() {
        let resolved = ResolvedPda {
            const_seeds: vec![vec![1; 32]],
            params: vec![],
            outer_program: Some(BPF_LOADER_UPGRADEABLE_BYTES.to_vec()),
        };

        let mut output = String::new();
        generate_pda_method(&mut output, "program_data", &resolved);

        // No program_id parameter
        assert!(output.contains("pub fn program_data_pda() -> (Pubkey, u8)"));
        assert!(!output.contains("program_id"));
        // Uses bpf_loader_upgradeable::ID
        assert!(output.contains("bpf_loader_upgradeable::ID"));
    }

    #[test]
    fn generate_pda_method_with_dynamic_seeds_and_const_outer_program() {
        let resolved = ResolvedPda {
            const_seeds: vec![],
            params: vec![PdaParam {
                name: "program".to_string(),
                rust_type: "&Pubkey".to_string(),
                seed_expr: "program.as_ref()".to_string(),
            }],
            outer_program: Some(BPF_LOADER_UPGRADEABLE_BYTES.to_vec()),
        };

        let mut output = String::new();
        generate_pda_method(&mut output, "program_data", &resolved);

        // Has the dynamic param but no program_id
        assert!(output.contains("pub fn program_data_pda(program: &Pubkey) -> (Pubkey, u8)"));
        assert!(output.contains("bpf_loader_upgradeable::ID"));
    }

    #[test]
    fn const_program_expr_bpf_loader() {
        let expr = const_program_expr(&BPF_LOADER_UPGRADEABLE_BYTES);
        assert_eq!(
            expr,
            "anchor_lang::solana_program::bpf_loader_upgradeable::ID"
        );
    }

    #[test]
    fn const_program_expr_unknown_program() {
        let bytes = [42u8; 32];
        let expr = const_program_expr(&bytes);
        assert!(expr.starts_with("Pubkey::new_from_array("));
        assert!(expr.contains("42"));
    }
}
