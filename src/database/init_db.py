#!/usr/bin/env python3
"""
Inicializa la base de datos SQLite del proyecto con la tabla `users` según el DDL proporcionado
y opcionalmente inserta un usuario de prueba.

Uso:
  python3 src/database/init_db.py            # crea la tabla si no existe, no inserta usuario por defecto
  python3 src/database/init_db.py --add-user  # crea tabla y añade usuario de prueba 'user'/'user' con admin=1
  python3 src/database/init_db.py --add-user --admin 0  # añade user con Admin=0

Este script modifica el archivo src/database/database.db en el repositorio.
"""
import sqlite3
import argparse
from pathlib import Path
import sys

DB_PATH = Path(__file__).resolve().parent / "database.db"

CREATE_USERS_SQL = r'''
CREATE TABLE IF NOT EXISTS "users" (
	"name" 	TEXT NOT NULL UNIQUE,
	"password" 	TEXT NOT NULL UNIQUE,
	"correo electronico" 	TEXT NOT NULL UNIQUE,
	"Admin" 	INTEGER NOT NULL,
	PRIMARY KEY("name","password","correo electronico","Admin")
);
'''

CREATE_INVENTARIO_SQL = r'''
CREATE TABLE IF NOT EXISTS "inventario" (
    "id" INTEGER NOT NULL UNIQUE,
    "nombre_producto" TEXT NOT NULL,
    "precio_producto" TEXT NOT NULL,
    "cantidad_producto" TEXT,
    PRIMARY KEY("id","nombre_producto")
);
'''

def ensure_db_exists(path: Path):
    if not path.exists():
        # Create empty database file (sqlite3 will create on connect, but touch for clarity)
        path.parent.mkdir(parents=True, exist_ok=True)
        open(path, 'a').close()
        print(f"[info] Creado nuevo archivo de base de datos en: {path}")


def create_table(conn: sqlite3.Connection):
    cur = conn.cursor()
    cur.executescript(CREATE_USERS_SQL + CREATE_INVENTARIO_SQL)
    conn.commit()
    print("[info] Tablas 'users' e 'inventario' creadas o ya existentes.")


def add_user(conn: sqlite3.Connection, name: str, password: str, correo: str, admin: int):
    cur = conn.cursor()
    cur.execute('SELECT COUNT(1) FROM users WHERE name = ?', (name,))
    exists = cur.fetchone()[0]
    if exists:
        print(f"[info] El usuario '{name}' ya existe. No se inserta.")
        return
    try:
        cur.execute('INSERT INTO users (name,password,"correo electronico",Admin) VALUES (?,?,?,?)',
                    (name, password, correo, admin))
        conn.commit()
        print(f"[ok] Usuario '{name}' insertado con Admin={admin}.")
    except sqlite3.IntegrityError as e:
        print(f"[error] No se pudo insertar usuario: {e}")


def main():
    parser = argparse.ArgumentParser(description='Inicializar DB y tabla users')
    parser.add_argument('--add-user', action='store_true', help='Añadir usuario de prueba user/user')
    parser.add_argument('--admin', type=int, choices=(0,1), default=1, help='Valor Admin para el usuario de prueba')
    parser.add_argument('--db', type=str, default=str(DB_PATH), help='Ruta a la base de datos SQLite')
    args = parser.parse_args()

    db_file = Path(args.db)
    if not db_file.exists():
        print(f"[warn] No se encontró {db_file}. Se creará al conectar.")
    try:
        conn = sqlite3.connect(str(db_file))
    except Exception as e:
        print(f"[error] No se pudo abrir la base de datos: {e}")
        sys.exit(1)

    create_table(conn)

    if args.add_user:
        add_user(conn, 'user', 'user', 'user@example.com', args.admin)

    conn.close()

if __name__ == '__main__':
    main()
