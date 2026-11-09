use std::path::Path;

/// Replaces a JSON file after its complete contents have been written.
pub fn write_atomic(path: &Path, encoded: &[u8]) -> std::io::Result<()> {
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    std::fs::write(&temporary, encoded)?;
    match std::fs::rename(&temporary, path) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = std::fs::remove_file(temporary);
            Err(error)
        }
    }
}
