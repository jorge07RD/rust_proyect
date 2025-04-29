use clap::Parser;
use std::io::{Write, Read, stdin, stdout};
use serde::{Serialize, Deserialize};
use std::fs;
use std::process::Command;
use std::env;

/// Herramienta para restaurar bases de datos Odoo en contenedores Docker

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    /// Host de la base de datos
    #[arg(short, long)]
    xhost: Option<String>,
    
    /// Puerto de la base de datos
    #[arg(short, long)]
    port: Option<u16>,

    /// Usuario de la base de datos
    #[arg(short, long)]
    username: Option<String>,
    
    /// Contraseña de la base de datos (opcional, se puede usar PGPASSWORD)
    #[arg(short = 'w', long)]
    password: Option<String>,

    /// ID del contenedor docker
    #[arg(short, long)]
    container_id: Option<String>,
    
    /// Ver perfil guardado
    #[arg(long = "vp")]
    #[serde(skip)]
    view_profile: bool,

    /// Ver backups
    #[arg(long = "vb")]
    #[serde(skip)]
    view_backups: bool,
    
    /// Nombre de la base de datos destino
    #[arg(short, long)]
    namedb: Option<String>,
    
    /// Ruta base para backups (dentro del contenedor)
    #[arg(short, long)]
    dir_backup: Option<String>,
    
    /// Ejecutar el comando de restauración
    #[arg(short, long)]
    run: bool,
}

impl Args {
    fn new(xhost: String, port: u16, username: String, password: Option<String>, 
           container_id: String, dir_backup: Option<String>) -> Self {
        Self { 
            xhost: Some(xhost), 
            port: Some(port), 
            username: Some(username),
            password,
            container_id: Some(container_id),
            dir_backup,
            view_profile: false,
            view_backups: false, // Campo faltante añadido aquí
            namedb: None,
            run: false
        }
    }
    // create and save a Profile

    fn save(&self) -> Result<(), std::io::Error> {
        // Solo guardar si tenemos todos los campos necesarios
        if self.xhost.is_some() && self.port.is_some() && self.username.is_some() && self.container_id.is_some() {
            // parse the profile to json
            let json = serde_json::to_string(self).unwrap();
            // save the json to a file
            let mut file = std::fs::File::create("profile.json")?;
            file.write_all(json.as_bytes())?;
            println!("Profile saved to profile.json");
        } else {
            println!("Faltan campos necesarios para guardar el perfil.");
        }
        Ok(())
    }
    
    // Function to print the JSON to console
    fn print_json(&self) {
        let json = serde_json::to_string_pretty(self).unwrap();
        println!("Profile JSON:");
        println!("{}", json);
    }
    
    // Function to load and print profile from file
    fn load_and_print() -> Result<(), std::io::Error> {
        // Check if profile exists
        if !std::path::Path::new("profile.json").exists() {
            println!("No profile found. Create one first.");
            return Ok(());
        }
        
        // Read the profile file
        let mut file = fs::File::open("profile.json")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // Pretty print the JSON
        let parsed: serde_json::Value = serde_json::from_str(&contents)?;
        let pretty = serde_json::to_string_pretty(&parsed)?;
        println!("{}", pretty);
        
        Ok(())
    }
    
    // Cargar perfil desde archivo
    fn load() -> Result<Self, std::io::Error> {
        if !std::path::Path::new("profile.json").exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No profile found. Create one first."
            ));
        }
        
        let mut file = fs::File::open("profile.json")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let profile: Self = serde_json::from_str(&contents)?;
        Ok(profile)
    }
    
    // Verificar si una base de datos existe
    fn database_exists(&self, dbname: &str) -> Result<bool, Box<dyn std::error::Error>> {
        if let (Some(xhost), Some(port), Some(username), Some(container_id)) = 
            (&self.xhost, self.port, &self.username, &self.container_id) {
            
            // Construimos un comando psql para verificar si la base de datos existe
            let check_cmd = format!(
                "psql --host \"{}\" --port \"{}\" --username \"{}\" --dbname postgres -c \"SELECT 1 FROM pg_database WHERE datname = '{}';\"",
                xhost, port, username, dbname
            );
            
            // Preparamos el comando docker con variables de entorno si hay contraseña
            let mut cmd = Command::new("docker");
            cmd.args(&["exec"]);
            
            // Si hay contraseña, la agregamos como variable de entorno PGPASSWORD
            if let Some(password) = &self.password {
                cmd.args(&["-e", &format!("PGPASSWORD={}", password)]);
            }
            
            // Completamos el comando con el ID del contenedor y el comando a ejecutar
            cmd.args(&[container_id, "bash", "-c", &check_cmd]);
            
            // Ejecutamos el comando
            let output = cmd.output()?;
            
            // Si el comando fue exitoso y la salida contiene un "1", la base de datos existe
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(output.status.success() && stdout.contains("(1 row)"))
            
        } else {
            Err("Faltan datos del perfil".into())
        }
    }
    
    // Crear una nueva base de datos
    fn create_database(&self, dbname: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(xhost), Some(port), Some(username), Some(container_id)) = 
            (&self.xhost, self.port, &self.username, &self.container_id) {
            
            println!("Creando base de datos '{}'...", dbname);
            
            // Construimos el comando para crear la base de datos
            let create_cmd = format!(
                "psql --host \"{}\" --port \"{}\" --username \"{}\" --dbname postgres -c \"CREATE DATABASE \\\"{}\\\";\"",
                xhost, port, username, dbname
            );
            
            // Preparamos el comando docker con variables de entorno si hay contraseña
            let mut cmd = Command::new("docker");
            cmd.args(&["exec"]);
            
            // Si hay contraseña, la agregamos como variable de entorno PGPASSWORD
            let mut password_provided = false;
            if let Some(password) = &self.password {
                cmd.args(&["-e", &format!("PGPASSWORD={}", password)]);
                password_provided = true;
            }
            
            // Completamos el comando con el ID del contenedor y el comando a ejecutar
            cmd.args(&[container_id, "bash", "-c", &create_cmd]);
            
            // Ejecutamos el comando
            let output = cmd.output()?;
            
            if output.status.success() {
                println!("Base de datos '{}' creada exitosamente", dbname);
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                // Detectar si el error es por falta de contraseña
                if stderr.contains("fe_sendauth: no password supplied") {
                    if !password_provided {
                        println!("Se requiere contraseña para el usuario '{}'", username);
                        print!("Ingrese la contraseña: ");
                        stdout().flush()?;
                        
                        // Leer la contraseña desde la entrada estándar
                        let mut password = String::new();
                        stdin().read_line(&mut password)?;
                        let password = password.trim().to_string();
                        
                        // Crear una nueva instancia con la contraseña
                        let mut new_self = self.clone();
                        new_self.password = Some(password);
                        
                        // Intentar nuevamente con la nueva contraseña
                        return new_self.create_database(dbname);
                    }
                }
                
                Err(format!("Error al crear la base de datos: {}", stderr).into())
            }
            
        } else {
            Err("Faltan datos del perfil".into())
        }
    }
    
    // Generar ruta al archivo SQL basado en el nombre de la base de datos
    fn generate_file_path(&self, namedb: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Usamos la ruta base del perfil, o una predeterminada si no está definida
        // NOTA: Esta ruta es DENTRO del contenedor Docker
        let base_dir = self.dir_backup.as_ref()
            .map(|s| s.as_str())
            .unwrap_or("/tmp/backups");
        
        // Construimos la ruta completa: dir_base/namedb/dump.sql (DENTRO del contenedor)
        let file_path = format!("{}/{}/dump.sql", base_dir, namedb);
        
        Ok(file_path)
    }
    
    // Ejecutar comando psql dentro del contenedor
    fn execute_psql(&self, namedb: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(xhost), Some(port), Some(username), Some(container_id)) = 
            (&self.xhost, self.port, &self.username, &self.container_id) {
            
            // Verificamos si la base de datos existe
            if !self.database_exists(namedb)? {
                println!("La base de datos '{}' no existe.", namedb);
                print!("¿Desea crearla? (s/n): ");
                stdout().flush()?;
                
                let mut input = String::new();
                stdin().read_line(&mut input)?;
                
                if input.trim().to_lowercase() == "s" {
                    match self.create_database(namedb) {
                        Ok(_) => println!("Base de datos creada correctamente."),
                        Err(e) => {
                            // Si el error es porque no se puede crear la BD, intentamos detectar
                            // si es porque falta la contraseña
                            let error_msg = e.to_string();
                            if error_msg.contains("fe_sendauth: no password supplied") {
                                println!("Se requiere contraseña para el usuario '{}'", username);
                                print!("Ingrese la contraseña: ");
                                stdout().flush()?;
                                
                                // Leer la contraseña desde la entrada estándar
                                let mut password = String::new();
                                stdin().read_line(&mut password)?;
                                let password = password.trim().to_string();
                                
                                // Crear una nueva instancia con la contraseña
                                let mut new_self = self.clone();
                                new_self.password = Some(password);
                                
                                // Intentar nuevamente con la nueva contraseña
                                new_self.create_database(namedb)?;
                            } else {
                                return Err(e);
                            }
                        }
                    }
                } else {
                    return Err(format!("Operación cancelada. La base de datos '{}' no existe.", namedb).into());
                }
            }
            
            // Generamos la ruta al archivo SQL basado en el nombre de la BD
            let file_path = self.generate_file_path(namedb)?;
            
            // Construimos el comando psql básico
            let psql_cmd = format!(
                "psql --host \"{}\" --port \"{}\" --username \"{}\" --dbname \"{}\" -f \"{}\"",
                xhost, port, username, namedb, file_path
            );
            
            println!("Ejecutando en el contenedor {}:", container_id);
            println!("{}", psql_cmd);
            
            // Preparamos el comando docker con variables de entorno si hay contraseña
            let mut cmd = Command::new("docker");
            cmd.args(&["exec"]);
            
            // Si hay contraseña, la agregamos como variable de entorno PGPASSWORD
            let mut password_provided = false;
            if let Some(password) = &self.password {
                cmd.args(&["-e", &format!("PGPASSWORD={}", password)]);
                password_provided = true;
            }
            
            // Completamos el comando con el ID del contenedor y el comando a ejecutar
            cmd.args(&[container_id, "bash", "-c", &psql_cmd]);
            
            // Ejecutamos el comando
            let output = cmd.output()?;
            
            if output.status.success() {
                println!("Comando ejecutado con éxito");
                println!("Salida:");
                println!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                eprintln!("Error al ejecutar el comando:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                
                // Verificamos si el error es por base de datos no existente
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                if stderr.contains("fe_sendauth: no password supplied") {
                    if !password_provided {
                        println!("Se requiere contraseña para el usuario '{}'", username);
                        print!("Ingrese la contraseña: ");
                        stdout().flush()?;
                        
                        // Leer la contraseña desde la entrada estándar
                        let mut password = String::new();
                        stdin().read_line(&mut password)?;
                        let password = password.trim().to_string();
                        
                        // Crear una nueva instancia con la contraseña
                        let mut new_self = self.clone();
                        new_self.password = Some(password);
                        
                        // Intentar nuevamente con la nueva contraseña
                        return new_self.execute_psql(namedb);
                    }
                } else if stderr.contains("database") && stderr.contains("does not exist") {
                    print!("¿Desea crear la base de datos '{}' y reintentar? (s/n): ", namedb);
                    stdout().flush()?;
                    
                    let mut input = String::new();
                    stdin().read_line(&mut input)?;
                    
                    if input.trim().to_lowercase() == "s" {
                        // Crear la base de datos y reintentar
                        self.create_database(namedb)?;
                        return self.execute_psql(namedb);
                    } else {
                        return Err("Operación cancelada".into());
                    }
                }
                
                // Si llegamos aquí, no fue ninguno de los errores conocidos
                return Err(format!("Error al ejecutar el comando: {}", stderr).into());
            }
            
            Ok(())
        } else {
            Err("Faltan datos del perfil".into())
        }
    }

    // Listar carpetas de backup y verificar si contienen dump.sql
    fn view_backup_folders(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(container_id), Some(dir_backup)) = (&self.container_id, &self.dir_backup) {
            println!("Listando carpetas de backup en {} (dentro del contenedor {}):", dir_backup, container_id);
            
            // Comando para listar directorios en la ruta de backup (dentro del contenedor)
            let list_cmd = format!("find {} -maxdepth 1 -type d -not -path {} | sort", dir_backup, dir_backup);
            
            // Ejecutar el comando en el contenedor
            let mut cmd = Command::new("docker");
            cmd.args(&["exec", container_id, "bash", "-c", &list_cmd]);
            
            let output = cmd.output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Error al listar directorios: {}", stderr).into());
            }
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let dirs: Vec<&str> = stdout.lines().collect();
            
            if dirs.is_empty() {
                println!("No se encontraron carpetas de backup.");
                return Ok(());
            }
            
            println!("Carpetas encontradas:");
            
            for dir in dirs {
                let dir_name = dir.split('/').last().unwrap_or(dir);
                
                // Verificar si existe dump.sql en la carpeta
                let check_sql_cmd = format!("[ -f {}/{}/dump.sql ] && echo 'true' || echo 'false'", dir_backup, dir_name);
                
                let mut check_cmd = Command::new("docker");
                check_cmd.args(&["exec", container_id, "bash", "-c", &check_sql_cmd]); // Corrigiendo el nombre de la variable
                
                let check_output = check_cmd.output()?;
                let has_dump = String::from_utf8_lossy(&check_output.stdout).trim() == "true";
                
                // Mostrar el resultado con una marca según si tiene dump.sql o no
                let marker = if has_dump { "✓" } else { "X" };
                println!("{} {} - {}", marker, dir_name, if has_dump { "Tiene dump.sql" } else { "No tiene dump.sql" });
            }
            
            Ok(())
        } else {
            if self.container_id.is_none() {
                Err("Falta ID del contenedor. Especifique --container_id".into())
            } else {
                Err("Falta directorio de backup. Especifique --dir_backup".into())
            }
        }
    }
} 

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Si se usa la bandera --vp, cargar y mostrar el perfil guardado
    if args.view_profile {
        if let Err(e) = Args::load_and_print() {
            eprintln!("Error al cargar el perfil: {}", e);
        }
        return Ok(());
    }
    
    // Si se usa la bandera --vb, listar las carpetas de backup
    if args.view_backups {
        // Intentamos cargar el perfil guardado para obtener container_id y dir_backup
        let profile = match Args::load() {
            Ok(mut p) => {
                // Actualizamos dir_backup si se proporcionó en línea de comandos
                if let Some(dir_backup) = &args.dir_backup {
                    p.dir_backup = Some(dir_backup.clone());
                }
                // Actualizamos container_id si se proporcionó en línea de comandos
                if let Some(container_id) = &args.container_id {
                    p.container_id = Some(container_id.clone());
                }
                p
            },
            Err(e) => {
                // Si no hay perfil guardado, usamos los argumentos actuales
                if let (Some(container_id), Some(dir_backup)) = (&args.container_id, &args.dir_backup) {
                    let profile = Args::new(
                        "".to_string(),  // xhost (no importa en este caso)
                        0,              // port (no importa en este caso)
                        "".to_string(),  // username (no importa en este caso)
                        None,           // password (no importa en este caso)
                        container_id.clone(),
                        Some(dir_backup.clone())
                    );
                    profile
                } else {
                    return Err(format!("Error al cargar el perfil: {}. Especifique --container_id y --dir_backup.", e).into());
                }
            }
        };
        
        profile.view_backup_folders()?;
        return Ok(());
    }
    
    // Si se proporciona run y namedb, ejecutar el comando psql
    if args.run {
        // Intentamos cargar el perfil guardado
        let mut profile = match Args::load() {
            Ok(mut p) => {
                // Actualizamos dir_backup si se proporcionó en línea de comandos
                if let Some(dir_backup) = &args.dir_backup {
                    p.dir_backup = Some(dir_backup.clone());
                }
                // Actualizamos password si se proporcionó en línea de comandos
                if let Some(password) = &args.password {
                    p.password = Some(password.clone());
                }
                p
            },
            Err(e) => {
                // Si no hay perfil guardado, usamos los argumentos actuales
                if let (Some(xhost), Some(port), Some(username), Some(container_id)) = 
                    (&args.xhost, args.port, &args.username, &args.container_id) {
                    Args::new(
                        xhost.clone(), 
                        port, 
                        username.clone(),
                        args.password.clone(),
                        container_id.clone(),
                        args.dir_backup.clone()
                    )
                } else {
                    return Err(format!("Error al cargar el perfil: {}. Especifique todos los parámetros.", e).into());
                }
            }
        };
        
        // Si no hay contraseña en los argumentos o perfil, intentamos usar PGPASSWORD
        if profile.password.is_none() {
            if let Ok(pg_pass) = env::var("PGPASSWORD") {
                profile.password = Some(pg_pass);
            }
        }
        
        if let Some(namedb) = &args.namedb {
            profile.execute_psql(namedb)?;
        } else {
            eprintln!("Para ejecutar necesita especificar --namedb");
        }
        
        return Ok(());
    }
    
    // De lo contrario, procede con el flujo normal
    // Solo ejecutar si tenemos todos los campos necesarios
    if let (Some(xhost), Some(port), Some(username), Some(container_id)) = 
        (&args.xhost, args.port, &args.username, &args.container_id) {
        
        // crear un instance of Profile
        let profile = Args::new(
            xhost.clone(), 
            port, 
            username.clone(),
            args.password.clone(),
            container_id.clone(),
            args.dir_backup.clone()
        );
        
        // save the profile
        if let Err(e) = profile.save() {
            eprintln!("Error al guardar el perfil: {}", e);
        }
        
        // print the profile as JSON to console
        profile.print_json();
    } else if !args.view_profile && !args.view_backups {
        println!("Para guardar un perfil, especifique xhost, port, username y container_id");
        println!("Ejemplo: cargo run -- --xhost db --port 5432 --username odoo --container_id mi-contenedor --dir_backup /tmp/backups");
        println!("Para autenticación con contraseña: --password mypassword o configure la variable PGPASSWORD");
        println!("Para ver el perfil guardado: cargo run -- --vp");
        println!("Para ver las carpetas de backup: cargo run -- --vb");
        println!("Para restaurar una base de datos:");
        println!("cargo run -- --run --namedb mi_base_datos");
    }
    
    Ok(())
}
