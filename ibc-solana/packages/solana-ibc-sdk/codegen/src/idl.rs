// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Idl {
    pub instructions: Vec<IdlInstruction>,
    #[serde(default)]
    pub events: Vec<IdlEvent>,
    #[serde(default)]
    pub types: Vec<IdlTypeDef>,
    #[serde(default)]
    pub accounts: Vec<IdlAccountDef>,
}

#[derive(Deserialize)]
pub struct IdlAccountDef {
    pub name: String,
    pub discriminator: Vec<u8>,
}

#[derive(Deserialize)]
pub struct IdlInstruction {
    pub name: String,
    pub discriminator: Vec<u8>,
    pub accounts: Vec<IdlInstructionAccount>,
    #[serde(default)]
    pub args: Vec<IdlInstructionArg>,
}

#[derive(Deserialize)]
pub struct IdlInstructionArg {
    pub name: String,
    #[serde(rename = "type")]
    pub arg_type: IdlFieldType,
}

#[derive(Deserialize)]
pub struct IdlInstructionAccount {
    pub name: String,
    #[serde(default)]
    pub writable: bool,
    #[serde(default)]
    pub signer: bool,
    pub address: Option<String>,
    #[serde(default)]
    pub pda: Option<IdlPda>,
}

#[derive(Deserialize)]
pub struct IdlPda {
    pub seeds: Vec<IdlPdaSeed>,
    #[serde(default)]
    pub program: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct IdlPdaSeed {
    pub kind: String,
    #[serde(default)]
    pub value: Option<Vec<u8>>,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct IdlEvent {
    pub name: String,
    #[allow(dead_code)]
    pub discriminator: Vec<u8>,
}

#[derive(Deserialize, Clone)]
pub struct IdlTypeDef {
    pub name: String,
    #[serde(default)]
    pub docs: Vec<String>,
    #[serde(rename = "type")]
    pub type_def: IdlTypeDefBody,
}

#[derive(Deserialize, Clone)]
pub struct IdlTypeDefBody {
    pub kind: String,
    #[serde(default)]
    pub fields: Vec<IdlField>,
    #[serde(default)]
    pub variants: Vec<IdlEnumVariant>,
}

#[derive(Deserialize, Clone)]
pub struct IdlField {
    pub name: Option<String>,
    #[serde(default)]
    pub docs: Vec<String>,
    #[serde(rename = "type")]
    pub field_type: IdlFieldType,
}

/// Enum variant fields can be either named (`{name, type}`) or unnamed (just a type).
/// Anchor IDL uses unnamed fields for tuple variants like `Ack([u8; 32])`.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum IdlVariantField {
    Named(IdlField),
    Unnamed(IdlFieldType),
}

impl IdlVariantField {
    pub const fn field_type(&self) -> &IdlFieldType {
        match self {
            Self::Named(f) => &f.field_type,
            Self::Unnamed(t) => t,
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum IdlFieldType {
    Primitive(String),
    Vec { vec: Box<Self> },
    Option { option: Box<Self> },
    Array { array: (Box<Self>, usize) },
    Defined { defined: IdlDefinedRef },
}

#[derive(Deserialize, Clone)]
pub struct IdlDefinedRef {
    pub name: String,
}

#[derive(Deserialize, Clone)]
pub struct IdlEnumVariant {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<IdlVariantField>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_minimal_idl() {
        let json = r#"{
            "instructions": [],
            "address": "test"
        }"#;
        let idl: Idl = serde_json::from_str(json).unwrap();
        assert!(idl.instructions.is_empty());
        assert!(idl.events.is_empty());
        assert!(idl.types.is_empty());
        assert!(idl.accounts.is_empty());
    }

    #[test]
    fn deserialize_instruction_with_accounts() {
        let json = r#"{
            "instructions": [{
                "name": "initialize",
                "discriminator": [1, 2, 3, 4, 5, 6, 7, 8],
                "accounts": [
                    {
                        "name": "payer",
                        "writable": true,
                        "signer": true
                    },
                    {
                        "name": "system_program",
                        "address": "11111111111111111111111111111111"
                    }
                ],
                "args": [
                    {
                        "name": "amount",
                        "type": "u64"
                    }
                ]
            }]
        }"#;

        let idl: Idl = serde_json::from_str(json).unwrap();
        assert_eq!(idl.instructions.len(), 1);

        let ix = &idl.instructions[0];
        assert_eq!(ix.name, "initialize");
        assert_eq!(ix.discriminator, vec![1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(ix.accounts.len(), 2);

        assert!(ix.accounts[0].writable);
        assert!(ix.accounts[0].signer);
        assert!(ix.accounts[0].address.is_none());

        assert!(!ix.accounts[1].writable);
        assert_eq!(
            ix.accounts[1].address.as_deref(),
            Some("11111111111111111111111111111111")
        );

        assert_eq!(ix.args.len(), 1);
        assert_eq!(ix.args[0].name, "amount");
        assert!(matches!(ix.args[0].arg_type, IdlFieldType::Primitive(ref p) if p == "u64"));
    }

    #[test]
    fn deserialize_pda_seeds() {
        let json = r#"{
            "instructions": [{
                "name": "test",
                "discriminator": [0,0,0,0,0,0,0,0],
                "accounts": [{
                    "name": "state",
                    "pda": {
                        "seeds": [
                            {"kind": "const", "value": [115, 116, 97, 116, 101]},
                            {"kind": "account", "path": "owner"},
                            {"kind": "arg", "path": "id"}
                        ]
                    }
                }]
            }]
        }"#;

        let idl: Idl = serde_json::from_str(json).unwrap();
        let pda = idl.instructions[0].accounts[0].pda.as_ref().unwrap();
        assert_eq!(pda.seeds.len(), 3);
        assert_eq!(pda.seeds[0].kind, "const");
        assert_eq!(pda.seeds[0].value.as_ref().unwrap(), b"state");
        assert_eq!(pda.seeds[1].kind, "account");
        assert_eq!(pda.seeds[1].path.as_deref(), Some("owner"));
        assert_eq!(pda.seeds[2].kind, "arg");
        assert_eq!(pda.seeds[2].path.as_deref(), Some("id"));
    }

    #[test]
    fn deserialize_field_types() {
        let json = r#"{
            "instructions": [{
                "name": "test",
                "discriminator": [0,0,0,0,0,0,0,0],
                "accounts": [],
                "args": [
                    {"name": "a", "type": "u64"},
                    {"name": "b", "type": {"vec": "u8"}},
                    {"name": "c", "type": {"option": "string"}},
                    {"name": "d", "type": {"array": ["u8", 32]}},
                    {"name": "e", "type": {"defined": {"name": "MyType"}}}
                ]
            }]
        }"#;

        let idl: Idl = serde_json::from_str(json).unwrap();
        let args = &idl.instructions[0].args;

        assert!(matches!(&args[0].arg_type, IdlFieldType::Primitive(p) if p == "u64"));
        assert!(matches!(&args[1].arg_type, IdlFieldType::Vec { .. }));
        assert!(matches!(&args[2].arg_type, IdlFieldType::Option { .. }));
        assert!(matches!(&args[3].arg_type, IdlFieldType::Array { .. }));
        assert!(
            matches!(&args[4].arg_type, IdlFieldType::Defined { defined } if defined.name == "MyType")
        );
    }

    #[test]
    fn deserialize_types_and_events() {
        let json = r#"{
            "instructions": [],
            "types": [{
                "name": "mod::MyStruct",
                "type": {
                    "kind": "struct",
                    "fields": [
                        {"name": "x", "type": "u64"}
                    ]
                }
            }],
            "events": [{
                "name": "mod::MyStruct",
                "discriminator": [1, 2, 3, 4, 5, 6, 7, 8]
            }],
            "accounts": [{
                "name": "mod::MyAccount",
                "discriminator": [10, 20, 30, 40, 50, 60, 70, 80]
            }]
        }"#;

        let idl: Idl = serde_json::from_str(json).unwrap();
        assert_eq!(idl.types.len(), 1);
        assert_eq!(idl.types[0].name, "mod::MyStruct");
        assert_eq!(idl.types[0].type_def.kind, "struct");
        assert_eq!(idl.types[0].type_def.fields.len(), 1);

        assert_eq!(idl.events.len(), 1);
        assert_eq!(idl.events[0].name, "mod::MyStruct");

        assert_eq!(idl.accounts.len(), 1);
        assert_eq!(idl.accounts[0].name, "mod::MyAccount");
        assert_eq!(
            idl.accounts[0].discriminator,
            vec![10, 20, 30, 40, 50, 60, 70, 80]
        );
    }

    #[test]
    fn deserialize_enum_variants() {
        let json = r#"{
            "instructions": [],
            "types": [{
                "name": "Status",
                "type": {
                    "kind": "enum",
                    "variants": [
                        {"name": "Active"},
                        {"name": "Data", "fields": [{"name": "value", "type": "u64"}]},
                        {"name": "Hash", "fields": [{"array": ["u8", 32]}]}
                    ]
                }
            }]
        }"#;

        let idl: Idl = serde_json::from_str(json).unwrap();
        let variants = &idl.types[0].type_def.variants;
        assert_eq!(variants.len(), 3);

        assert!(variants[0].fields.is_empty());
        assert!(matches!(&variants[1].fields[0], IdlVariantField::Named(_)));
        assert!(matches!(
            &variants[2].fields[0],
            IdlVariantField::Unnamed(_)
        ));
    }

    #[test]
    fn variant_field_type_accessor() {
        let named = IdlVariantField::Named(IdlField {
            name: Some("x".to_string()),
            docs: vec![],
            field_type: IdlFieldType::Primitive("u64".to_string()),
        });
        assert!(matches!(named.field_type(), IdlFieldType::Primitive(p) if p == "u64"));

        let unnamed = IdlVariantField::Unnamed(IdlFieldType::Primitive("bool".to_string()));
        assert!(matches!(unnamed.field_type(), IdlFieldType::Primitive(p) if p == "bool"));
    }
}
