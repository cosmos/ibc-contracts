// SPDX-License-Identifier: Apache-2.0

use std::io::Result;

fn main() -> Result<()> {
    // Configure prost-build
    let mut config = prost_build::Config::new();

    // Add Eq derive for message types
    config.type_attribute(".ibc.applications.gmp.v1.Acknowledgement", "#[derive(Eq)]");
    config.type_attribute(".ibc.applications.gmp.v1.GMPPacketData", "#[derive(Eq)]");
    config.type_attribute(".solana.GMPSolanaPayload", "#[derive(Eq)]");
    config.type_attribute(".solana.SolanaAccountMeta", "#[derive(Eq)]");

    // Compile proto files
    config.compile_protos(
        &[
            "../../../proto/ibc/applications/gmp/v1/packet.proto",
            "../../../proto/solana/gmp_solana_payload.proto",
        ],
        &["../../../proto"],
    )?;

    println!("cargo:rerun-if-changed=../../../proto/ibc/applications/gmp/v1/packet.proto");
    println!("cargo:rerun-if-changed=../../../proto/solana/gmp_solana_payload.proto");

    Ok(())
}
