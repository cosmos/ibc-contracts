use serde::{Deserialize, Serialize};

/// Generated build identity shared by verification key generation and prepared-witness validation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProgramIdentity {
    pub elf_sha256: String,
    pub program_v_key: String,
}

/// Operator-approved identities. These must come from deployment configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovedPrograms {
    #[serde(rename = "update-client")]
    pub update: ProgramIdentity,
    pub membership: ProgramIdentity,
    #[serde(rename = "non-membership")]
    pub non_membership: ProgramIdentity,
}

impl ApprovedPrograms {
    pub fn validate(&self) -> Result<(), crate::ToolError> {
        for (name, identity) in self.entries() {
            validate_hash(name, "elfSha256", &identity.elf_sha256)?;
            validate_hash(name, "programVKey", &identity.program_v_key)?;
        }
        Ok(())
    }

    /// Compare values returned by the deployed contract getters with configuration.
    pub fn validate_deployed_vkeys(
        &self,
        update: [u8; 32],
        membership: [u8; 32],
        non_membership: [u8; 32],
    ) -> Result<(), crate::ToolError> {
        self.validate()?;
        for ((name, identity), actual) in
            self.entries()
                .into_iter()
                .zip([update, membership, non_membership])
        {
            crate::prepared::validate_vkey_identity(
                name,
                &format!("0x{}", hex::encode(actual)),
                identity,
            )?;
        }
        Ok(())
    }

    pub(crate) fn entries(&self) -> [(&'static str, &ProgramIdentity); 3] {
        [
            ("update-client", &self.update),
            ("membership", &self.membership),
            ("non-membership", &self.non_membership),
        ]
    }
}

fn validate_hash(program: &str, field: &str, value: &str) -> Result<(), crate::ToolError> {
    let bytes = value.strip_prefix("0x").filter(|payload| {
        payload.len() == 64
            && payload.bytes().all(|byte| byte.is_ascii_hexdigit())
            && payload.bytes().any(|byte| byte != b'0')
    });
    if bytes.is_none() {
        return Err(crate::ToolError::InvalidProgramIdentity(format!(
            "{program}.{field} must be a nonzero 0x-prefixed 32-byte hex value"
        )));
    }
    Ok(())
}
