use std::{io, process::Command};

fn main() -> io::Result<()> {
    let otterscan_dir = std::fs::canonicalize("src/otterscan")?;

    for cmds in [&["install"] as &[&str], &["run", "build"]] {
        assert!(Command::new("npm")
            .current_dir(&otterscan_dir)
            .args(cmds)
            .status()?
            .success());
    }

    Ok(())
}
