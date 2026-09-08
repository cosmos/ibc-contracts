// SPDX-License-Identifier: Apache-2.0

//! Programs for `sp1-ics07-tendermint`.

use sp1_sdk::{Elf, SP1VerifyingKey};
use std::sync::OnceLock;

/// Trait for SP1 ICS07 Tendermint programs.
pub trait SP1Program {
    /// Get the ELF bytes for the program.
    fn elf(&self) -> &[u8];

    /// Get the verifying key for the program.
    #[must_use]
    fn get_vkey(&self) -> SP1VerifyingKey;
}

/// Compute the verifying key from ELF bytes using a blocking mock prover.
///
/// Spawns a dedicated OS thread to avoid panicking when called from
/// within an existing tokio runtime (the SP1 blocking API creates its
/// own runtime internally).
fn compute_vkey(elf: &[u8]) -> SP1VerifyingKey {
    let elf = elf.to_vec();
    std::thread::spawn(move || {
        use sp1_sdk::blocking::{Prover, ProverClient};
        use sp1_sdk::ProvingKey;
        let mock = ProverClient::builder().mock().build();
        let pk = mock.setup(Elf::from(elf)).expect("setup failed");
        pk.verifying_key().clone()
    })
    .join()
    .expect("compute_vkey thread panicked")
}

/// SP1 ICS07 Tendermint programs.
pub struct SP1ICS07TendermintPrograms {
    /// The update client program.
    pub update_client: UpdateClientProgram,
    /// The membership program.
    pub membership: MembershipProgram,
    /// The update client and membership program.
    pub update_client_and_membership: UpdateClientAndMembershipProgram,
    /// The misbehaviour program.
    pub misbehaviour: MisbehaviourProgram,
}

/// SP1 ICS07 Tendermint update client program.
pub struct UpdateClientProgram {
    elf: Vec<u8>,
    vkey: OnceLock<SP1VerifyingKey>,
}

/// SP1 ICS07 Tendermint verify (non)membership program.
pub struct MembershipProgram {
    elf: Vec<u8>,
    vkey: OnceLock<SP1VerifyingKey>,
}

/// SP1 ICS07 Tendermint update client and verify (non)membership program.
pub struct UpdateClientAndMembershipProgram {
    elf: Vec<u8>,
    vkey: OnceLock<SP1VerifyingKey>,
}

/// SP1 ICS07 Tendermint misbehaviour program.
pub struct MisbehaviourProgram {
    elf: Vec<u8>,
    vkey: OnceLock<SP1VerifyingKey>,
}

impl UpdateClientProgram {
    /// Create a new [`UpdateClientProgram`] from the given ELF bytes.
    #[must_use]
    pub const fn new(elf: Vec<u8>) -> Self {
        Self {
            elf,
            vkey: OnceLock::new(),
        }
    }
}

impl MembershipProgram {
    /// Create a new [`MembershipProgram`] from the given ELF bytes.
    #[must_use]
    pub const fn new(elf: Vec<u8>) -> Self {
        Self {
            elf,
            vkey: OnceLock::new(),
        }
    }
}

impl UpdateClientAndMembershipProgram {
    /// Create a new [`UpdateClientAndMembershipProgram`] from the given ELF bytes.
    #[must_use]
    pub const fn new(elf: Vec<u8>) -> Self {
        Self {
            elf,
            vkey: OnceLock::new(),
        }
    }
}

impl MisbehaviourProgram {
    /// Create a new [`MisbehaviourProgram`] from the given ELF bytes.
    #[must_use]
    pub const fn new(elf: Vec<u8>) -> Self {
        Self {
            elf,
            vkey: OnceLock::new(),
        }
    }
}

impl SP1Program for UpdateClientProgram {
    fn elf(&self) -> &[u8] {
        &self.elf
    }

    fn get_vkey(&self) -> SP1VerifyingKey {
        self.vkey.get_or_init(|| compute_vkey(&self.elf)).clone()
    }
}

impl SP1Program for MembershipProgram {
    fn elf(&self) -> &[u8] {
        &self.elf
    }

    fn get_vkey(&self) -> SP1VerifyingKey {
        self.vkey.get_or_init(|| compute_vkey(&self.elf)).clone()
    }
}

impl SP1Program for UpdateClientAndMembershipProgram {
    fn elf(&self) -> &[u8] {
        &self.elf
    }

    fn get_vkey(&self) -> SP1VerifyingKey {
        self.vkey.get_or_init(|| compute_vkey(&self.elf)).clone()
    }
}

impl SP1Program for MisbehaviourProgram {
    fn elf(&self) -> &[u8] {
        &self.elf
    }

    fn get_vkey(&self) -> SP1VerifyingKey {
        self.vkey.get_or_init(|| compute_vkey(&self.elf)).clone()
    }
}
