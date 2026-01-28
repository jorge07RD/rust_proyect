# Configuración para PostgreSQL Local

Si tu PostgreSQL está instalado directamente en tu sistema (no en Docker), sigue estos pasos:

## Paso 1: Preparar el directorio de backups

Crea un directorio local donde tengas tus backups:

```bash
mkdir -p /home/jorge/backups
```

Asegúrate de que la estructura sea:
```
/home/jorge/backups/
  ├── nombre_backup_1/
  │   └── dump.sql
  ├── nombre_backup_2/
  │   └── dump.sql
  └── nombre_backup_3/
      └── dump.sql
```

## Paso 2: Configurar el perfil en RDO

1. Ejecuta la aplicación:
   ```bash
   cd /home/jorge/proyect/rust_proyect/rdo
   cargo run
   ```

2. Presiona `2` para ir a la pestaña **Perfil**

3. Presiona `e` para editar

4. Configura los campos:
   - **Host**: `localhost` (o `127.0.0.1`)
   - **Puerto**: `5432` (o el puerto que uses)
   - **Usuario**: Tu usuario de PostgreSQL (ej: `postgres`, `odoo`, etc.)
   - **Contraseña**: Tu contraseña de PostgreSQL
   - **Container ID**: **DEJAR VACÍO** ← Importante! Esto activa el modo local
   - **Directorio Backup**: `/home/jorge/backups` (tu ruta local)

5. Presiona `Ctrl+S` para guardar

## Paso 3: Verificar backups disponibles

1. Presiona `3` para ir a la pestaña **Backups**

2. La aplicación cargará automáticamente la lista de carpetas en `/home/jorge/backups`

3. Verás una lista con:
   - ✓ (verde) = La carpeta tiene dump.sql
   - ✗ (rojo) = La carpeta NO tiene dump.sql

## Paso 4: Restaurar una base de datos

1. En la pestaña **Backups**, selecciona un backup con las flechas ↑/↓

2. Presiona `Enter` para ir a **Restaurar**

3. Escribe el nombre de la base de datos destino (ej: `produccion_nueva`)

4. Presiona `Enter` para iniciar la restauración

5. Verás los logs en tiempo real

## Notas importantes

### Permisos de PostgreSQL

Asegúrate de que tu usuario tenga permisos para:
- Crear bases de datos
- Restaurar dumps

Si usas el usuario `postgres`, generalmente tiene todos los permisos.

### Archivo pg_hba.conf

Si obtienes errores de autenticación, verifica que tu `pg_hba.conf` permita conexiones locales:

```
# En /etc/postgresql/[version]/main/pg_hba.conf
local   all             postgres                                peer
host    all             all             127.0.0.1/32            md5
```

### Comando psql disponible

La aplicación usa el comando `psql`. Verifica que esté disponible:

```bash
which psql
# Debería mostrar algo como: /usr/bin/psql
```

Si no está disponible, instala PostgreSQL client:

```bash
# Ubuntu/Debian
sudo apt install postgresql-client

# Fedora/RHEL
sudo dnf install postgresql
```

## Ejemplo de configuración completa

Tu `profile.json` debería verse así:

```json
{
  "xhost": "localhost",
  "port": 5432,
  "username": "postgres",
  "password": "mi_password",
  "container_id": null,
  "dir_backup": "/home/jorge/backups"
}
```

## Solución de problemas

### Error: "No se puede conectar a la base de datos"

1. Verifica que PostgreSQL esté corriendo:
   ```bash
   sudo systemctl status postgresql
   ```

2. Si no está corriendo, inícialo:
   ```bash
   sudo systemctl start postgresql
   ```

### Error: "fe_sendauth: no password supplied"

Tu configuración requiere contraseña. Asegúrate de haberla ingresado en el campo **Contraseña** del perfil.

### Error: "El directorio /home/jorge/backups no existe"

Crea el directorio:
```bash
mkdir -p /home/jorge/backups
```

### Los backups no aparecen

Verifica que:
1. La ruta del directorio sea correcta
2. Las carpetas de backup existan
3. Cada carpeta tenga un archivo `dump.sql` dentro

## Ventajas del modo local

- ✅ Más rápido (sin overhead de Docker)
- ✅ Acceso directo a archivos del sistema
- ✅ No necesitas gestionar contenedores
- ✅ Ideal para desarrollo local
