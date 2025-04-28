# Proyecto Rust - RDO

## Descripción
RDO es una herramienta CLI desarrollada en Rust que permite guardar y visualizar perfiles de configuración en formato JSON. Simplifica la gestión de credenciales y configuraciones para bases de datos y otros servicios.

## Requisitos previos
- Rust (última versión estable recomendada)
- Cargo (generalmente incluido con la instalación de Rust)

## Instalación

1. Clona este repositorio:
```bash
git clone https://github.com/tu-usuario/rust_proyect.git
cd rust_proyect/rdo
```

2. Compila el proyecto:
```bash
cargo build --release
```

## Uso

### Guardar un nuevo perfil:
```bash
cargo run -- --xhost db --port 5432 --username odoo
```

### Ver el perfil guardado:
```bash
cargo run -- --vp
```

## Estructura del proyecto
```
rust_proyect/
├── rdo/
│   ├── src/           # Código fuente
│   │   └── main.rs    # Código principal
│   ├── Cargo.toml     # Configuración y dependencias del proyecto
│   └── profile.json   # Archivo de perfil guardado (generado)
├── .gitignore         # Archivos ignorados por Git
└── README.md          # Este archivo
```

## Características
- Guardado de perfiles de configuración en formato JSON
- Visualización de perfiles guardados con formato legible
- Interfaz de línea de comandos intuitiva con argumentos nombrados
- Validación de datos de entrada

## Dependencias principales
- clap: Para el análisis de argumentos de la línea de comandos
- serde: Para la serialización y deserialización de JSON
- serde_json: Para el manejo de formato JSON

## Contribuir
Las contribuciones son bienvenidas. Por favor, sigue estos pasos para contribuir:
1. Haz fork del proyecto
2. Crea tu rama de características (`git checkout -b feature/amazing-feature`)
3. Haz commit de tus cambios (`git commit -m 'Añadir increíble característica'`)
4. Haz push a la rama (`git push origin feature/amazing-feature`)
5. Abre un Pull Request

## Licencia
MIT

## Contacto
[Tu nombre] - [Tu correo electrónico] - [Enlace a tu perfil en GitHub]