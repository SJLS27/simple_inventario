Instrucciones para inicializar la base de datos del proyecto

Archivo: src/database/init_db.py

Uso:

1) Crear la tabla `users` (si no existe):

```sh
python3 src/database/init_db.py
```

2) Crear la tabla y añadir un usuario de prueba `user` con contraseña `user` y `Admin=1`:

```sh
python3 src/database/init_db.py --add-user
```

3) Añadir el usuario con Admin=0:

```sh
python3 src/database/init_db.py --add-user --admin 0
```

Notas:
- El script modifica el archivo `src/database/database.db` (crea el archivo si no existe).
- Este script usa sqlite3 incluido en Python y está pensado para uso local de desarrollo.
- Para producción considera scripts de migración y no almacenar contraseñas en texto plano.
