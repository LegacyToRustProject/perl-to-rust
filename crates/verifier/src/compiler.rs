use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use tracing::{debug, warn};

/// Result of running `cargo check` on generated Rust code.
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub success: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Run `cargo check` on a Rust project directory.
pub fn cargo_check(project_dir: &Path) -> Result<CompileResult> {
    debug!(dir = %project_dir.display(), "Running cargo check");

    let output = Command::new("cargo")
        .args(["check", "--message-format=short"])
        .current_dir(project_dir)
        .output()
        .context("Failed to run cargo check")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for line in stderr.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("error") {
            errors.push(trimmed.to_string());
        } else if trimmed.starts_with("warning") {
            warnings.push(trimmed.to_string());
        }
    }

    // Also check stdout for error messages
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("error") {
            errors.push(trimmed.to_string());
        }
    }

    let success = output.status.success();
    if !success {
        warn!(error_count = errors.len(), "cargo check failed");
    }

    Ok(CompileResult {
        success,
        errors,
        warnings,
    })
}

/// Run `cargo build` on a Rust project directory.
pub fn cargo_build(project_dir: &Path) -> Result<CompileResult> {
    debug!(dir = %project_dir.display(), "Running cargo build");

    let output = Command::new("cargo")
        .args(["build", "--message-format=short"])
        .current_dir(project_dir)
        .output()
        .context("Failed to run cargo build")?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for line in stderr.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("error") {
            errors.push(trimmed.to_string());
        } else if trimmed.starts_with("warning") {
            warnings.push(trimmed.to_string());
        }
    }

    Ok(CompileResult {
        success: output.status.success(),
        errors,
        warnings,
    })
}

/// Run the compiled Rust binary and capture output.
pub fn run_binary(project_dir: &Path, args: &[&str]) -> Result<String> {
    // First build
    let build_result = cargo_build(project_dir)?;
    if !build_result.success {
        anyhow::bail!("Build failed: {}", build_result.errors.join("\n"));
    }

    let output = Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .current_dir(project_dir)
        .output()
        .context("Failed to run binary")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        anyhow::bail!("Binary execution failed: {}", stderr);
    }

    Ok(stdout)
}

/// Run a Perl script and capture output.
pub fn run_perl(script_path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("perl")
        .arg(script_path)
        .args(args)
        .output()
        .context("Failed to run perl")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        anyhow::bail!("Perl execution failed: {}", stderr);
    }

    Ok(stdout)
}

/// Check if Perl is available on the system.
pub fn perl_available() -> bool {
    Command::new("perl")
        .arg("-v")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if cargo is available on the system.
pub fn cargo_available() -> bool {
    Command::new("cargo")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_available() {
        assert!(cargo_available());
    }

    #[test]
    fn test_cargo_check_nonexistent() {
        let result = cargo_check(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_cargo_check_valid_project() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("main.rs"), "fn main() {}\n").unwrap();

        let result = cargo_check(tmp.path()).unwrap();
        assert!(result.success);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_cargo_check_invalid_code() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("main.rs"),
            "fn main() { let x: i32 = \"hello\"; }\n",
        )
        .unwrap();

        let result = cargo_check(tmp.path()).unwrap();
        assert!(!result.success);
        assert!(!result.errors.is_empty());
    }
}
