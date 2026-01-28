use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use anyhow::Result;
use std::path::Path;

pub struct CommandExecutor {
    pub container_id: Option<String>,
}

impl CommandExecutor {
    pub fn new(container_id: Option<String>) -> Self {
        Self { container_id }
    }

    /// Check if running in Docker mode
    pub fn is_docker_mode(&self) -> bool {
        self.container_id.as_ref().map(|id| !id.is_empty()).unwrap_or(false)
    }

    /// Execute a command (in Docker container or locally)
    pub async fn exec(&self, command: &str, env_vars: &[(&str, &str)]) -> Result<(String, String, bool)> {
        let mut cmd = if self.is_docker_mode() {
            let mut docker_cmd = Command::new("docker");
            docker_cmd.args(&["exec"]);

            // Add environment variables
            for (key, value) in env_vars {
                docker_cmd.args(&["-e", &format!("{}={}", key, value)]);
            }

            docker_cmd.args(&[
                self.container_id.as_ref().unwrap(),
                "bash",
                "-c",
                command
            ]);
            docker_cmd
        } else {
            // Local mode: execute command directly
            let mut local_cmd = Command::new("bash");
            local_cmd.arg("-c");
            local_cmd.arg(command);

            // Add environment variables
            for (key, value) in env_vars {
                local_cmd.env(key, value);
            }

            local_cmd
        };

        let output = cmd.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        Ok((stdout, stderr, success))
    }

    /// Execute a command and stream output line by line
    pub async fn exec_streaming<F>(
        &self,
        command: &str,
        env_vars: &[(&str, &str)],
        mut callback: F,
    ) -> Result<bool>
    where
        F: FnMut(String) + Send,
    {
        let mut cmd = if self.is_docker_mode() {
            let mut docker_cmd = Command::new("docker");
            docker_cmd.args(&["exec"]);

            for (key, value) in env_vars {
                docker_cmd.args(&["-e", &format!("{}={}", key, value)]);
            }

            docker_cmd.args(&[
                self.container_id.as_ref().unwrap(),
                "bash",
                "-c",
                command
            ]);
            docker_cmd
        } else {
            // Local mode
            let mut local_cmd = Command::new("bash");
            local_cmd.arg("-c");
            local_cmd.arg(command);

            for (key, value) in env_vars {
                local_cmd.env(key, value);
            }

            local_cmd
        };

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Some(line) = lines.next_line().await? {
                callback(line);
            }
        }

        let status = child.wait().await?;
        Ok(status.success())
    }

    /// List directories in a given path
    pub async fn list_directories(&self, path: &str) -> Result<Vec<String>> {
        if self.is_docker_mode() {
            // Docker mode: use find command in container
            let command = format!("find {} -maxdepth 1 -type d -not -path {} | sort", path, path);
            let (stdout, stderr, success) = self.exec(&command, &[]).await?;

            if !success {
                anyhow::bail!("Error al listar directorios: {}", stderr);
            }

            let dirs: Vec<String> = stdout
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| {
                    l.split('/').last().unwrap_or(l).to_string()
                })
                .collect();

            Ok(dirs)
        } else {
            // Local mode: use std::fs
            let path_obj = Path::new(path);
            if !path_obj.exists() {
                anyhow::bail!("El directorio {} no existe", path);
            }

            let mut dirs = Vec::new();
            let entries = tokio::fs::read_dir(path).await?;
            let mut entries = entries;

            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await?.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        dirs.push(name.to_string());
                    }
                }
            }

            dirs.sort();
            Ok(dirs)
        }
    }

    /// Check if a file exists
    pub async fn file_exists(&self, path: &str) -> Result<bool> {
        if self.is_docker_mode() {
            // Docker mode: use bash test
            let command = format!("[ -f {} ] && echo 'true' || echo 'false'", path);
            let (stdout, _stderr, _success) = self.exec(&command, &[]).await?;
            Ok(stdout.trim() == "true")
        } else {
            // Local mode: use tokio::fs
            Ok(tokio::fs::metadata(path).await.is_ok())
        }
    }
}
