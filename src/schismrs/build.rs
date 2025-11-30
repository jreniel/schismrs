// schismrs/build.rs
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let crate_env_name = "SCHISMRS_CLI_VERSION";
    let version = std::env::var("CARGO_PKG_VERSION").unwrap();
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    let gitrepo_path = ".gitrepo";

    let version_suffix = if Path::new(gitrepo_path).exists() {
        // We're in a git subrepo - get both workspace and local info

        // Get current workspace HEAD (what the workspace is at)
        let workspace_hash = get_git_hash(".")
            .or_else(|| get_git_hash("..")) // Try parent directory if current fails
            .unwrap_or_else(|| "unknown".to_string());

        // Check if workspace (excluding current directory) is dirty
        let workspace_dirty = check_git_dirty_excluding_current().unwrap_or("");

        // Get local subrepo commit from .gitrepo file
        let gitrepo_content = fs::read_to_string(gitrepo_path)?;
        let local_commit =
            parse_subrepo_commit(&gitrepo_content).unwrap_or_else(|| "unknown".to_string());

        let local_hash = if local_commit.len() >= 8 {
            local_commit[..8].to_string()
        } else {
            local_commit
        };

        // Check if local directory is dirty
        let local_dirty = check_git_dirty_current().unwrap_or("");

        format!(
            "{}{}-{}{}-{}",
            workspace_hash, workspace_dirty, local_hash, local_dirty, profile
        )
    } else {
        // Standalone mode - just local hash and dirty state
        let local_hash = get_git_hash(".").unwrap_or_else(|| "unknown".to_string());

        let local_dirty = check_git_dirty_current().unwrap_or("");

        format!("{}{}-{}", local_hash, local_dirty, profile)
    };

    let full_version = format!("{} {}", version, version_suffix);

    // println!("cargo:warning=Final version: {}", full_version);
    println!("cargo:rustc-env={}={}", crate_env_name, full_version);

    // Tell cargo to rerun if relevant files change
    println!("cargo:rerun-if-changed=.gitrepo");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/HEAD");

    Ok(())
}

fn get_git_hash(git_dir: &str) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(&["-C", git_dir, "rev-parse", "HEAD"])
        .output()
        .ok()?;

    if !output.status.success() {
        // Check if it's because there are no commits yet
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("bad revision 'HEAD'") || stderr.contains("ambiguous argument 'HEAD'") {
            return Some("no-commits".to_string());
        }
        return None;
    }

    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Ensure we got a real hash, not just "HEAD"
    if hash == "HEAD" || hash.is_empty() || hash.len() < 8 {
        return Some("no-commits".to_string());
    }

    // Validate it's a proper git hash (hex characters)
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some("no-commits".to_string());
    }

    Some(if hash.len() >= 8 {
        hash[..8].to_string()
    } else {
        hash
    })
}

fn check_git_dirty_excluding_current() -> Option<&'static str> {
    let status = std::process::Command::new("git")
        .args(&["diff", "--quiet", ":(exclude)."])
        .status()
        .ok()?;

    Some(if status.success() { "" } else { "-dirty" })
}

fn check_git_dirty_current() -> Option<&'static str> {
    let status = std::process::Command::new("git")
        .args(&["diff", "--quiet", "."])
        .status()
        .ok()?;

    Some(if status.success() { "" } else { "-dirty" })
}

fn parse_subrepo_commit(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("commit = ") {
            let commit = line.strip_prefix("commit = ")?.to_string();
            // Validate it looks like a real commit hash
            if commit.len() >= 8 && commit.chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(commit);
            }
        }
    }
    None
}
