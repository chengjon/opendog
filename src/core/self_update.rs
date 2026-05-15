use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use serde::Serialize;
use walkdir::WalkDir;

use crate::error::{OpenDogError, Result};

pub const SELF_UPDATE_STATUS_SCHEMA: &str = "opendog.cli.self-update-status.v1";
pub const SELF_UPDATE_BUILD_SCHEMA: &str = "opendog.cli.self-update-build.v1";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SelfUpdateStatus {
    pub schema_version: &'static str,
    pub source_path: String,
    pub current_exe: String,
    pub release_binary: String,
    pub release_binary_exists: bool,
    pub release_binary_mtime: Option<u64>,
    pub source_latest_mtime: Option<u64>,
    pub needs_rebuild: bool,
    pub restart_required_for_mcp: bool,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SelfUpdateBuildResult {
    pub schema_version: &'static str,
    pub source_path: String,
    pub command: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub release_binary: String,
    pub restart_required_for_mcp: bool,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildCommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: PathBuf,
}

pub fn self_update_status(source: &Path, current_exe: PathBuf) -> Result<SelfUpdateStatus> {
    let source_path = validate_source_path(source)?;
    let release_binary = release_binary_path(&source_path);
    let release_binary_mtime = file_mtime_secs(&release_binary)?;
    let source_latest_mtime = source_latest_mtime_secs(&source_path)?;
    let release_binary_exists = release_binary.exists();
    let needs_rebuild = match (release_binary_mtime, source_latest_mtime) {
        (Some(binary), Some(source)) => source > binary,
        (None, Some(_)) => true,
        _ => false,
    };
    let restart_required_for_mcp = needs_rebuild;
    let mut next_steps = Vec::new();
    if needs_rebuild {
        next_steps.push(format!(
            "Run `opendog self-update build --source {}` from a WSL/Linux shell.",
            source_path.display()
        ));
        next_steps
            .push("After a successful build, restart or reconnect MCP hosts manually.".to_string());
    }

    Ok(SelfUpdateStatus {
        schema_version: SELF_UPDATE_STATUS_SCHEMA,
        source_path: source_path.display().to_string(),
        current_exe: current_exe.display().to_string(),
        release_binary: release_binary.display().to_string(),
        release_binary_exists,
        release_binary_mtime,
        source_latest_mtime,
        needs_rebuild,
        restart_required_for_mcp,
        next_steps,
    })
}

pub fn run_self_update_build(source: &Path) -> Result<SelfUpdateBuildResult> {
    let source_path = validate_source_path(source)?;
    let spec = build_command_for(&source_path)?;
    let status = Command::new(&spec.program)
        .args(&spec.args)
        .current_dir(&spec.current_dir)
        .status()?;
    if !status.success() {
        return Err(OpenDogError::InvalidInput(format!(
            "`cargo build --release` failed with exit code {:?}",
            status.code()
        )));
    }
    Ok(build_result_for(&source_path, status.code(), "built"))
}

pub fn validate_source_path(source: &Path) -> Result<PathBuf> {
    let source_path = source.canonicalize().map_err(|err| {
        OpenDogError::InvalidPath(format!(
            "OpenDog source path '{}' is not accessible: {}",
            source.display(),
            err
        ))
    })?;
    let manifest_path = source_path.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).map_err(|err| {
        OpenDogError::InvalidPath(format!(
            "OpenDog source path '{}' must contain Cargo.toml: {}",
            source_path.display(),
            err
        ))
    })?;
    if !manifest.contains("name = \"opendog\"") {
        return Err(OpenDogError::InvalidPath(format!(
            "OpenDog source path '{}' is not an OpenDog source tree",
            source_path.display()
        )));
    }
    Ok(source_path)
}

pub fn build_command_for(source: &Path) -> Result<BuildCommandSpec> {
    let source_path = validate_source_path(source)?;
    Ok(BuildCommandSpec {
        program: "cargo".to_string(),
        args: vec!["build".to_string(), "--release".to_string()],
        current_dir: source_path,
    })
}

pub fn build_result_for(
    source: &Path,
    exit_code: Option<i32>,
    status: &str,
) -> SelfUpdateBuildResult {
    let release_binary = release_binary_path(source);
    SelfUpdateBuildResult {
        schema_version: SELF_UPDATE_BUILD_SCHEMA,
        source_path: source.display().to_string(),
        command: "cargo build --release".to_string(),
        status: status.to_string(),
        exit_code,
        release_binary: release_binary.display().to_string(),
        restart_required_for_mcp: true,
        next_steps: vec!["Restart or reconnect MCP hosts that use this binary.".to_string()],
    }
}

fn release_binary_path(source: &Path) -> PathBuf {
    source.join("target").join("release").join("opendog")
}

pub fn build_info_needs_rebuild() -> Option<bool> {
    let exe = std::env::current_exe().ok()?;
    // Expect: {source}/target/release/opendog → source = parent.parent.parent
    let source = exe.parent()?.parent()?.parent()?;
    let manifest = source.join("Cargo.toml");
    if !manifest.exists() {
        return None;
    }
    let content = fs::read_to_string(&manifest).ok()?;
    if !content.contains("name = \"opendog\"") {
        return None;
    }
    let binary_mtime = file_mtime_secs(&exe).ok()??;
    let source_mtime = source_latest_mtime_secs(source).ok()??;
    Some(source_mtime > binary_mtime)
}

fn file_mtime_secs(path: &Path) -> Result<Option<u64>> {
    if !path.exists() {
        return Ok(None);
    }
    let modified = fs::metadata(path)?.modified()?;
    Ok(Some(
        modified
            .duration_since(UNIX_EPOCH)
            .map_err(|err| OpenDogError::InvalidInput(err.to_string()))?
            .as_secs(),
    ))
}

fn source_latest_mtime_secs(source: &Path) -> Result<Option<u64>> {
    let mut latest = None;
    for root in ["src", "Cargo.toml", "Cargo.lock", "build.rs"] {
        let path = source.join(root);
        if !path.exists() {
            continue;
        }
        if path.is_file() {
            latest = latest.max(file_mtime_secs(&path)?);
            continue;
        }
        for entry in WalkDir::new(&path).into_iter().filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            name != "target" && name != ".git"
        }) {
            let entry = entry?;
            if entry.file_type().is_file() {
                latest = latest.max(file_mtime_secs(entry.path())?);
            }
        }
    }
    Ok(latest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_source_tree() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"opendog\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "pub fn demo() {}\n").unwrap();
        dir
    }

    #[test]
    fn rejects_non_opendog_source_path() {
        let dir = tempfile::tempdir().unwrap();
        let err = validate_source_path(dir.path()).unwrap_err();
        assert!(err.to_string().contains("OpenDog source"));
    }

    #[test]
    fn status_marks_rebuild_needed_when_release_binary_is_missing() {
        let dir = fixture_source_tree();
        let status = self_update_status(dir.path(), PathBuf::from("/tmp/opendog")).unwrap();
        assert!(status.needs_rebuild);
        assert!(status.restart_required_for_mcp);
        assert!(!status.release_binary_exists);
        assert_eq!(status.schema_version, SELF_UPDATE_STATUS_SCHEMA);
    }

    #[test]
    fn build_command_uses_cargo_release_in_source_dir() {
        let dir = fixture_source_tree();
        let spec = build_command_for(dir.path()).unwrap();
        assert_eq!(spec.program, "cargo");
        assert_eq!(spec.args, vec!["build", "--release"]);
        assert_eq!(spec.current_dir, dir.path().canonicalize().unwrap());
    }

    #[test]
    fn build_result_reports_manual_mcp_reconnect_requirement() {
        let dir = fixture_source_tree();
        let source = dir.path().canonicalize().unwrap();
        let result = build_result_for(&source, Some(0), "built");
        assert_eq!(result.schema_version, SELF_UPDATE_BUILD_SCHEMA);
        assert_eq!(result.command, "cargo build --release");
        assert!(result.restart_required_for_mcp);
        assert!(result.next_steps[0].contains("Restart or reconnect MCP hosts"));
    }
}
