use clap::Parser;
use std::io::{Write, Read};
use serde::{Serialize, Deserialize};
use std::fs;
/// Simple program to greet a person

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    xhost: Option<String>,
    /// Number of times to greet
    #[arg(short, long)]
    port: Option<u16>,

    #[arg(short, long)]
    username: Option<String>,
    
    /// View profile - print the saved profile
    #[arg(long = "vp")]
    view_profile: bool,
}





impl Args {

    fn new(xhost: String, port: u16, username: String) -> Self {
        Self { 
            xhost: Some(xhost), 
            port: Some(port), 
            username: Some(username),
            view_profile: false 
        }
    }
    // create and save a Profile

    fn save(&self) -> Result<(), std::io::Error> {
        // Solo guardar si tenemos todos los campos necesarios
        if self.xhost.is_some() && self.port.is_some() && self.username.is_some() {
            // parse the profile to json
            let json = serde_json::to_string(self).unwrap();
            // save the json to a file
            let mut file = std::fs::File::create("profile.json")?;
            file.write_all(json.as_bytes())?;
            println!("Profile saved to profile.json");
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
} 

fn main() {
    let args = Args::parse();
    
    // Si se usa la bandera --vp, cargar y mostrar el perfil guardado
    if args.view_profile {
        if let Err(e) = Args::load_and_print() {
            eprintln!("Error al cargar el perfil: {}", e);
        }
        return;
    }
    
    // De lo contrario, procede con el flujo normal
    // Solo ejecutar si tenemos todos los campos necesarios
    if let (Some(xhost), Some(port), Some(username)) = (&args.xhost, args.port, &args.username) {
        // crear un instance of Profile
        let profile = Args::new(xhost.clone(), port, username.clone());
        
        // save the profile
        if let Err(e) = profile.save() {
            eprintln!("Error al guardar el perfil: {}", e);
        }
        
        // print the profile as JSON to console
        profile.print_json();
    } else if !args.view_profile {
        println!("Para guardar un perfil, especifique xhost, port y username");
        println!("Ejemplo: cargo run -- --xhost db --port 5432 --username odoo");
        println!("Para ver el perfil guardado: cargo run -- --vp");
    }
}
