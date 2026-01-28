# RDO - Restaurador de Datos Odoo

Una herramienta TUI (Terminal User Interface) escrita en Rust para facilitar la restauración de bases de datos PostgreSQL, especialmente orientada a entornos Odoo.

## Características

- **Interfaz TUI moderna y navegable** con pestañas y navegación por teclado
- **Dos modos de operación**: Docker y Local
  - **Modo Docker**: Ejecuta comandos dentro de contenedores Docker
  - **Modo Local**: Trabaja directamente con PostgreSQL local en el sistema
- **Gestión de perfiles** con editor interactivo
- **Visualización de backups** con filtrado en tiempo real
- **Restauración con logs en tiempo real** y barra de progreso
- **Historial persistente** de todas las restauraciones
- **Verificación automática** de backups disponibles
- **Soporte para autenticación** con contraseña

## Requisitos

- Rust 1.75.0 o superior
- PostgreSQL instalado (local o en Docker)
- **Para modo Docker**: Docker en ejecución con contenedor PostgreSQL
- **Para modo local**: PostgreSQL instalado en el sistema con psql disponible

## Instalación

```bash
# Clonar el repositorio
git clone https://github.com/jorge07RD/rust_proyect.git
cd rdo

# Compilar el proyecto
cargo build --release

# Opcional: Mover el binario a una ubicación en PATH
sudo cp target/release/rdo /usr/local/bin/
```

## Uso

### Iniciar la aplicación

```bash
cargo run
# o si está instalado en PATH:
rdo
```

### Navegación general

- **Tab / Shift+Tab**: Cambiar entre pestañas
- **1-4**: Saltar directamente a una pestaña
- **q**: Salir de la aplicación
- **Ctrl+C**: Salir forzado

### Pestaña Home

Muestra:
- Estado del perfil cargado
- Estadísticas de backups disponibles
- Historial reciente de restauraciones (últimas 5)

### Pestaña Perfil

Gestiona la configuración de conexión:

- **e**: Iniciar edición del perfil
- **↑/↓**: Navegar entre campos
- **Enter**: Editar campo seleccionado
- **Esc**: Cancelar edición
- **Ctrl+S**: Guardar perfil

Campos del perfil:
- **Host**: Dirección del servidor PostgreSQL (ej: localhost, db, 192.168.1.100)
- **Puerto**: Puerto de PostgreSQL (ej: 5432)
- **Usuario**: Usuario de la base de datos
- **Contraseña**: Contraseña (se oculta con asteriscos)
- **Container ID**: ID del contenedor Docker (**DEJAR VACÍO para modo local**)
- **Directorio Backup**: Ruta base de backups
  - **Modo Docker**: Ruta dentro del contenedor (ej: /tmp/backups)
  - **Modo Local**: Ruta en el sistema local (ej: /home/user/backups)

### Pestaña Backups

Lista y filtra los backups disponibles:

- **r**: Refrescar lista de backups
- **↑/↓**: Navegar entre backups
- **/**: Activar filtro de búsqueda
- **Esc**: Limpiar filtro
- **Enter**: Ir a restauración con el backup seleccionado

Indicadores:
- **✓** (verde): El backup tiene dump.sql
- **✗** (rojo): El backup no tiene dump.sql

### Pestaña Restaurar

Realiza la restauración de bases de datos:

1. Ingresar el nombre de la base de datos destino
2. Presionar **Enter** para iniciar la restauración
3. Ver logs en tiempo real
4. **↑/↓**: Hacer scroll en los logs
5. **Esc**: Cancelar restauración (mientras está en progreso)

Estados de la restauración:
- **Listo**: Esperando inicio
- **Restaurando**: En proceso (con logs en tiempo real)
- **Completado**: Restauración exitosa
- **Falló**: Error durante la restauración

## Modos de operación

### Modo Docker

En este modo, la aplicación ejecuta todos los comandos dentro de un contenedor Docker. Útil cuando PostgreSQL está corriendo en Docker.

**Configuración de ejemplo:**
```
Host: db
Puerto: 5432
Usuario: odoo
Contraseña: mi_password
Container ID: d48eed249db5 (o nombre del contenedor)
Directorio Backup: /tmp/backups
```

### Modo Local

En este modo, la aplicación ejecuta los comandos directamente en el sistema local. Útil cuando PostgreSQL está instalado en el sistema.

**Configuración de ejemplo:**
```
Host: localhost (o 127.0.0.1)
Puerto: 5432
Usuario: postgres
Contraseña: mi_password
Container ID: (DEJAR VACÍO)
Directorio Backup: /home/usuario/backups
```

**Importante**: En modo local, asegúrate de que:
- PostgreSQL esté instalado y en ejecución
- El comando `psql` esté disponible en el PATH
- El usuario tenga permisos para acceder a la base de datos
- La ruta de backups sea accesible desde el sistema

## Estructura de archivos

La herramienta espera que los dumps SQL estén organizados de la siguiente manera:

```
/ruta_base_backups/
  └── nombre_carpeta_backup/
      └── dump.sql
```

**Ejemplos:**

**Modo Docker:**
```
Container: /tmp/backups/produccion_04_28_2025/dump.sql
```
La ruta es relativa al sistema de archivos dentro del contenedor Docker.

**Modo Local:**
```
Sistema local: /home/usuario/backups/produccion_04_28_2025/dump.sql
```
La ruta es relativa al sistema de archivos local donde se ejecuta la aplicación.

## Archivos generados

- **profile.json**: Configuración del perfil guardado
- **restore_history.json**: Historial de todas las restauraciones realizadas

## Ejemplo de flujo completo

### Para PostgreSQL Local (sin Docker)

1. Iniciar la aplicación: `cargo run`
2. Ir a la pestaña **Perfil** (presionar `2`)
3. Presionar `e` para editar
4. Configurar campos:
   - Host: `localhost`
   - Puerto: `5432`
   - Usuario: `postgres` (o tu usuario de PostgreSQL)
   - Contraseña: tu contraseña
   - **Container ID: DEJAR VACÍO** ← Esto activa el modo local
   - Directorio Backup: `/home/usuario/backups` (ruta local)
5. Guardar con Ctrl+S
6. Ir a la pestaña **Backups** (presionar `3`)
7. Esperar a que carguen los backups automáticamente (de tu sistema local)
8. Navegar con ↑/↓ y seleccionar un backup
9. Presionar Enter para ir a restaurar
10. Ingresar nombre de la base de datos destino
11. Presionar Enter para iniciar
12. Ver logs en tiempo real
13. Al completar, ir a **Home** (presionar `1`) para ver el historial

### Para PostgreSQL en Docker

Similar al flujo anterior, pero:
- En el paso 4, **SÍ incluir** el Container ID del contenedor Docker
- El directorio de backup será una ruta dentro del contenedor (ej: `/tmp/backups`)

## Características avanzadas

### Historial persistente

Todas las restauraciones se guardan en `restore_history.json` con:
- Fecha y hora
- Nombre de la base de datos
- Carpeta de backup utilizada
- Resultado (exitoso/fallido)
- Mensaje de error (si aplica)

### Logs en tiempo real

Durante la restauración, se muestran todos los mensajes del proceso psql en tiempo real, permitiendo diagnosticar problemas inmediatamente.

### Filtrado de backups

En la pestaña Backups, puedes filtrar por nombre escribiendo `/` seguido del texto a buscar. Útil cuando tienes muchos backups.

### Verificación automática

La herramienta verifica automáticamente:
- Si el backup tiene dump.sql antes de restaurar
- Si la base de datos destino existe (la crea si no existe)
- La conexión al contenedor Docker

## Migración desde la versión CLI

Si tienes un `profile.json` de la versión CLI anterior, la nueva versión TUI lo cargará automáticamente. Los campos adicionales de la versión anterior serán ignorados.

### Migrar de Docker a modo Local

Si tenías configurado el modo Docker y ahora quieres usar PostgreSQL local:

1. Iniciar la aplicación
2. Ir a **Perfil** y presionar `e` para editar
3. Cambiar el **Host** a `localhost` o `127.0.0.1`
4. **Borrar completamente** el campo **Container ID** (dejar vacío)
5. Cambiar **Directorio Backup** a una ruta local (ej: `/home/usuario/backups`)
6. Guardar con Ctrl+S

La aplicación detectará automáticamente que no hay container_id y cambiará a modo local.

## Notas técnicas

- **Async/Await**: Usa Tokio para operaciones asíncronas, evitando bloqueos de UI
- **Arquitectura TEA**: Implementa The Elm Architecture para gestión de estado predecible
- **Ratatui**: Framework TUI moderno para interfaces de terminal
- **Crossterm**: Backend multiplataforma para control de terminal
