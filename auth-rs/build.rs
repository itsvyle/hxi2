use anyhow::Context as _;
fn main() -> anyhow::Result<()> {
    let permissions_file = generate_permissions_checking::find_permissions_path()
        .context("Failed to find permissions file in workspace")?;
    println!("cargo:rerun-if-changed={}", permissions_file);
    let generated_code = generate_permissions_checking::main(&permissions_file)?;
    std::fs::write("src/permissions_checking.rs", generated_code)
        .context("Failed to write generated permissions checking code to file")?;
    Ok(())
}
