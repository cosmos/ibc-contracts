// SPDX-License-Identifier: Apache-2.0

use solana_ibc_macros::ibc_app;

// Module with no IBC callbacks at all
#[ibc_app]
pub mod my_app {
    pub fn some_other_function() -> Result<()> {
        Ok(())
    }
}

fn main() {}
