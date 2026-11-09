use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    let output = PathBuf::from(env::var("OUT_DIR")?);
    let descriptor = output.join("prover.bin");
    tonic_build::configure()
        .file_descriptor_set_path(&descriptor)
        .compile_protos(&["proto/cli/prover.proto"], &["proto"])?;
    pbjson_build::Builder::new()
        .register_descriptors(&fs::read(descriptor)?)?
        .build(&[".ibc.v2.prover"])?;
    println!("cargo:rerun-if-changed=proto/cli/prover.proto");
    Ok(())
}
