use clap::Parser;
use std::io::{Write, Read};
use serde::{Serialize, Deserialize};
use std::fs;
use std::process::Command;
use std::env;

/// Herramienta para restaurar bases de datos Odoo en contenedores Docker

#[derive(Parser, Debug, Serialize, Deserialize)]
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
    
    // Generar ruta al archivo SQL basado en el nombre de la base de datos
    fn generate_file_path(&self, namedb: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Usamos la ruta base del perfil, o una predeterminada si no está definida
        let base_dir = self.dir_backup.as_ref()
            .map(|s| s.as_str())
            .unwrap_or("/tmp/backups");
        
        // Construimos la ruta completa: dir_base/namedb/dump.sql
        let file_path = format!("{}/{}/dump.sql", base_dir, namedb);
        
        Ok(file_path)
    }
    
    // Ejecutar comando psql dentro del contenedor
    fn execute_psql(&self, namedb: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(xhost), Some(port), Some(username), Some(container_id)) = 
            (&self.xhost, self.port, &self.username, &self.container_id) {
            
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
            if let Some(password) = &self.password {
                cmd.args(&["-e", &format!("PGPASSWORD={}", password)]);
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
            }
            
            Ok(())
        } else {
            Err("Faltan datos del perfil".into())
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
    } else if !args.view_profile {
        println!("Para guardar un perfil, especifique xhost, port, username y container_id");
        println!("Ejemplo: cargo run -- --xhost db --port 5432 --username odoo --container_id mi-contenedor --dir_backup /tmp/backups");
        println!("Para autenticación con contraseña: --password mypassword o configure la variable PGPASSWORD");
        println!("Para ver el perfil guardado: cargo run -- --vp");
        println!("Para restaurar una base de datos:");
        println!("cargo run -- --run --namedb mi_base_datos");
    }
    
    Ok(())
}
