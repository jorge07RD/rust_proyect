# RDO - Restaurador de Datos Odoo

Una herramienta de línea de comandos escrita en Rust para facilitar la restauración de bases de datos PostgreSQL en contenedores Docker, especialmente orientada a entornos Odoo.

## Características

- Restauración simplificada de bases de datos en contenedores Docker
- Guardado de perfiles para reutilizar configuraciones
- Generación automática de rutas basadas en el nombre de la base de datos
- Soporte para autenticación con contraseña (directa o mediante variable de entorno)
- Interfaz de línea de comandos intuitiva

## Requisitos

- Rust y Cargo instalados
- Docker en ejecución
- Contenedor con PostgreSQL configurado y accesible

## Instalación

```bash
# Clonar el repositorio
git clone https://github.com/tu-usuario/rdo.git
cd rdo

# Compilar el proyecto
cargo build --release

# Opcional: Mover el binario a una ubicación en PATH
sudo cp target/release/rdo /usr/local/bin/
```

## Uso básico

### Crear un perfil

Para guardar una configuración que se usará frecuentemente:

```bash
cargo run -- --xhost db --port 5432 --username odoo --container_id mi-contenedor --dir_backup /tmp/backups
```

Si necesitas incluir contraseña:

```bash
cargo run -- --xhost db --port 5432 --username odoo --password micontraseña --container_id mi-contenedor --dir_backup /tmp/backups
```

En forma corta
```bash
cargo run -x db -p 5432 -u odoo -w micontraseña -c mi-contenedor -d /tmp/backups
```

### Ver el perfil guardado

```bash
cargo run -- --vp
```

### Restaurar una base de datos

Usando el perfil guardado:

```bash
cargo run -- --run --namedb nombre_base_datos
```

Con parámetros adicionales:

```bash
cargo run -- --run --namedb nombre_base_datos --password micontraseña
```

O usando la variable de entorno para la contraseña:

```bash
PGPASSWORD=micontraseña cargo run -- --run --namedb nombre_base_datos
```

## Estructura de archivos

La herramienta espera que los dumps SQL estén organizados en la siguiente estructura dentro del contenedor:

```
/ruta_base_backups/
  └── nombre_base_datos/
      └── dump.sql
```

Por ejemplo, si `dir_backup` es `/tmp/backups` y el nombre de la base de datos es `produccion_04_28_2025`, buscará el archivo de dump en:

```
/tmp/backups/produccion_04_28_2025/dump.sql
```

## Opciones disponibles

| Opción | Descripción |
|--------|-------------|
| `--xhost`, `-x` | Host de la base de datos |
| `--port`, `-p` | Puerto de la base de datos |
| `--username`, `-u` | Usuario de la base de datos |
| `--password`, `-w` | Contraseña de la base de datos |
| `--container_id`, `-c` | ID del contenedor Docker |
| `--dir_backup`, `-d` | Ruta base para los backups (dentro del contenedor) |
| `--namedb`, `-n` | Nombre de la base de datos destino |
| `--run`, `-r` | Ejecutar la restauración |
| `--vp` | Ver el perfil guardado |

## Ejemplo completo

1. Guardar un perfil:
```bash
./rdo --xhost db --port 5432 --username odoo --container_id d48eed249db5 --dir_backup /tmp/backups --password secreto
```

2. Restaurar una base de datos:
```bash
./rdo --run --namedb produccion_04_28_2025
```

## Notas

- La contraseña puede proporcionarse mediante el argumento `--password` o mediante la variable de entorno `PGPASSWORD`.
- El perfil se guarda en el archivo `profile.json` en el directorio actual.
- Los mensajes de error del comando `psql` se muestran en caso de fallos.