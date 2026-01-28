use super::docker::CommandExecutor;
use super::profile::Profile;
use anyhow::Result;

pub struct DatabaseManager {
    profile: Profile,
    executor: CommandExecutor,
}

impl DatabaseManager {
    pub fn new(profile: Profile) -> Self {
        let executor = CommandExecutor::new(profile.container_id.clone());
        Self { profile, executor }
    }

    /// Check if a database exists
    pub async fn database_exists(&self, dbname: &str) -> Result<bool> {
        let check_cmd = format!(
            "psql --host \"{}\" --port \"{}\" --username \"{}\" --dbname postgres -c \"SELECT 1 FROM pg_database WHERE datname = '{}';\"",
            self.profile.xhost, self.profile.port, self.profile.username, dbname
        );

        let mut env_vars = vec![];
        if let Some(password) = &self.profile.password {
            env_vars.push(("PGPASSWORD", password.as_str()));
        }

        let (stdout, _stderr, success) = self.executor.exec(&check_cmd, &env_vars).await?;

        Ok(success && stdout.contains("(1 row)"))
    }

    /// Create a new database
    pub async fn create_database(&self, dbname: &str) -> Result<()> {
        let create_cmd = format!(
            "psql --host \"{}\" --port \"{}\" --username \"{}\" --dbname postgres -c \"CREATE DATABASE \\\"{}\\\";\"",
            self.profile.xhost, self.profile.port, self.profile.username, dbname
        );

        let mut env_vars = vec![];
        if let Some(password) = &self.profile.password {
            env_vars.push(("PGPASSWORD", password.as_str()));
        }

        let (_stdout, stderr, success) = self.executor.exec(&create_cmd, &env_vars).await?;

        if success {
            Ok(())
        } else {
            if stderr.contains("fe_sendauth: no password supplied") {
                anyhow::bail!("Se requiere contraseña para el usuario '{}'", self.profile.username);
            }
            anyhow::bail!("Error al crear la base de datos: {}", stderr);
        }
    }

    /// Restore a database from a backup with streaming logs
    pub async fn restore_database<F>(
        &self,
        dbname: &str,
        backup_folder: &str,
        mut log_callback: F,
    ) -> Result<()>
    where
        F: FnMut(String) + Send,
    {
        // Generate file path
        let base_dir = self.profile.dir_backup.as_ref()
            .map(|s| s.as_str())
            .unwrap_or("/tmp/backups");
        let file_path = format!("{}/{}/dump.sql", base_dir, backup_folder);

        // Check if database exists, create if needed
        if !self.database_exists(dbname).await? {
            log_callback(format!("La base de datos '{}' no existe. Creándola...", dbname));
            self.create_database(dbname).await?;
            log_callback(format!("Base de datos '{}' creada exitosamente", dbname));
        }

        // Build psql command
        let psql_cmd = format!(
            "psql --host \"{}\" --port \"{}\" --username \"{}\" --dbname \"{}\" -f \"{}\"",
            self.profile.xhost, self.profile.port, self.profile.username, dbname, file_path
        );

        log_callback(format!("Ejecutando restauración desde: {}", file_path));

        let mut env_vars = vec![];
        if let Some(password) = &self.profile.password {
            env_vars.push(("PGPASSWORD", password.as_str()));
        }

        let success = self.executor.exec_streaming(&psql_cmd, &env_vars, log_callback).await?;

        if success {
            Ok(())
        } else {
            anyhow::bail!("Error durante la restauración de la base de datos");
        }
    }

    /// Get the executor for direct access
    pub fn executor(&self) -> &CommandExecutor {
        &self.executor
    }
}
