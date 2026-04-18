//! Postprocessing Module
//!
//! Handles post-download actions:
//! - Moving completed files to a separate directory
//! - Executing external scripts for unpacking/renaming

use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Postprocessing configuration
#[derive(Debug, Clone)]
pub struct PostprocessConfig {
    /// Move completed files to this directory
    pub move_completed_dir: Option<String>,
    /// Path to external postprocessing script
    pub script_path: Option<String>,
    /// Timeout for script execution in seconds
    pub script_timeout_secs: u64,
}

impl Default for PostprocessConfig {
    fn default() -> Self {
        Self {
            move_completed_dir: None,
            script_path: None,
            script_timeout_secs: 300, // 5 minutes default
        }
    }
}

/// Result of postprocessing
#[derive(Debug)]
pub struct PostprocessResult {
    pub moved_to: Option<String>,
    pub script_exit_code: Option<i32>,
    pub script_output: Option<String>,
    pub errors: Vec<String>,
}

/// Run postprocessing on a completed download
///
/// # Arguments
/// * `source_path` - Full path to the downloaded file
/// * `config` - Postprocessing configuration
///
/// # Returns
/// * `PostprocessResult` with details of what was done
pub async fn run_postprocess(source_path: &str, config: &PostprocessConfig) -> PostprocessResult {
    let mut result = PostprocessResult {
        moved_to: None,
        script_exit_code: None,
        script_output: None,
        errors: Vec::new(),
    };

    let source = Path::new(source_path);

    // Check if source file exists
    if !source.exists() {
        result
            .errors
            .push(format!("Source file not found: {}", source_path));
        return result;
    }

    // Current file path (may change after move)
    let mut current_path = source_path.to_string();

    // Step 1: Move file if configured
    if let Some(ref move_dir) = config.move_completed_dir {
        if !move_dir.is_empty() {
            match move_file(&current_path, move_dir).await {
                Ok(new_path) => {
                    tracing::info!("Moved file to: {}", new_path);
                    result.moved_to = Some(new_path.clone());
                    current_path = new_path;
                }
                Err(e) => {
                    let err = format!("Failed to move file: {}", e);
                    tracing::error!("{}", err);
                    result.errors.push(err);
                }
            }
        }
    }

    // Step 2: Execute script if configured
    if let Some(ref script) = config.script_path {
        if !script.is_empty() {
            match run_script(script, &current_path, config.script_timeout_secs).await {
                Ok((exit_code, output)) => {
                    tracing::info!("Script exited with code: {}", exit_code);
                    result.script_exit_code = Some(exit_code);
                    result.script_output = Some(output);
                }
                Err(e) => {
                    let err = format!("Script execution failed: {}", e);
                    tracing::error!("{}", err);
                    result.errors.push(err);
                }
            }
        }
    }

    result
}

/// Move a file to a target directory
async fn move_file(source_path: &str, target_dir: &str) -> Result<String, std::io::Error> {
    let source = Path::new(source_path);
    let target_directory = Path::new(target_dir);

    // Create target directory if it doesn't exist
    if !target_directory.exists() {
        tokio::fs::create_dir_all(target_directory).await?;
        tracing::info!("Created directory: {}", target_dir);
    }

    // Get filename from source
    let filename = source.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid source path")
    })?;

    let target_path = target_directory.join(filename);
    let target_str = target_path.to_string_lossy().to_string();

    // Try rename first (fast, same filesystem)
    match tokio::fs::rename(source, &target_path).await {
        Ok(_) => Ok(target_str),
        Err(_) => {
            // Cross-filesystem: copy then delete
            tokio::fs::copy(source, &target_path).await?;
            tokio::fs::remove_file(source).await?;
            Ok(target_str)
        }
    }
}

/// Execute a postprocessing script
async fn run_script(
    script_path: &str,
    file_path: &str,
    timeout_secs: u64,
) -> Result<(i32, String), String> {
    let script = Path::new(script_path);

    if !script.exists() {
        return Err(format!("Script not found: {}", script_path));
    }

    // Create the command
    let mut cmd = Command::new(script_path);
    cmd.arg(file_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Spawn with timeout
    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn script: {}", e))?;

    let timeout = tokio::time::Duration::from_secs(timeout_secs);

    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(output)) => {
            let exit_code = output.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let combined_output = if stderr.is_empty() {
                stdout.to_string()
            } else {
                format!("{}\n{}", stdout, stderr)
            };

            Ok((exit_code, combined_output))
        }
        Ok(Err(e)) => Err(format!("Script execution error: {}", e)),
        Err(_) => Err(format!("Script timed out after {} seconds", timeout_secs)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_move_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target");

        std::fs::create_dir_all(&source_dir).unwrap();

        let source_file = source_dir.join("test.txt");
        std::fs::write(&source_file, "test content").unwrap();

        let result = move_file(source_file.to_str().unwrap(), target_dir.to_str().unwrap()).await;

        assert!(result.is_ok());
        let new_path = result.unwrap();
        assert!(Path::new(&new_path).exists());
        assert!(!source_file.exists());
    }

    #[tokio::test]
    async fn test_move_file_missing_source() {
        let temp_dir = TempDir::new().unwrap();
        let result = move_file("/nonexistent/file.txt", temp_dir.path().to_str().unwrap()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_postprocess_missing_file() {
        let config = PostprocessConfig::default();
        let result = run_postprocess("/nonexistent/file.txt", &config).await;

        assert!(!result.errors.is_empty());
        assert!(result.errors[0].contains("not found"));
    }

    #[tokio::test]
    async fn test_script_execution() {
        let temp_dir = TempDir::new().unwrap();

        // Create a simple test script
        let script_path = temp_dir.path().join("test.sh");
        let mut file = std::fs::File::create(&script_path).unwrap();
        writeln!(file, "#!/bin/bash").unwrap();
        writeln!(file, "echo \"Processed: $1\"").unwrap();
        drop(file);

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms).unwrap();
        }

        // Create a test file
        let test_file = temp_dir.path().join("download.mkv");
        std::fs::write(&test_file, "test").unwrap();

        let result = run_script(
            script_path.to_str().unwrap(),
            test_file.to_str().unwrap(),
            10,
        )
        .await;

        assert!(result.is_ok());
        let (exit_code, output) = result.unwrap();
        assert_eq!(exit_code, 0);
        assert!(output.contains("Processed:"));
    }
}
