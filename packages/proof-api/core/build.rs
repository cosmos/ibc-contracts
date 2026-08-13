// SPDX-License-Identifier: Apache-2.0

use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let descriptor_path = out_dir.join("proofapi_descriptor.bin");
    tonic_build::configure()
        .file_descriptor_set_path(&descriptor_path)
        .build_server(true)
        .compile_protos(
            &["../../../proto/proofapi/proofapi.proto"],
            &["../../../proto/proofapi"],
        )?;
    Ok(())
}
