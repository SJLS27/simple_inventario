import sqlite3
import os

def conectar_db():
    """Conecta a la base de datos"""
    try:
        conexion = sqlite3.connect('src/database/database.db')
        return conexion
    except sqlite3.Error as e:
        print(f"Error al conectar: {e}")
        return None

def crear_tabla_si_no_existe(conexion):
    """Crea las tablas users e inventarios si no existen"""
    try:
        cursor = conexion.cursor()
        
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS "users" (
                "name"	TEXT NOT NULL UNIQUE,
                "password"	TEXT NOT NULL UNIQUE,
                "correo electronico"	TEXT NOT NULL UNIQUE,
                "Admin"	INTEGER NOT NULL,
                PRIMARY KEY("name","password","correo electronico","Admin")
            )
        ''')

        cursor.execute('''
            CREATE TABLE IF NOT EXISTS "inventario" (
                "id" INTEGER NOT NULL UNIQUE,
                "nombre_producto" TEXT NOT NULL,
                "precio_producto" TEXT NOT NULL,
                "cantidad_producto" TEXT,
                PRIMARY KEY("id","nombre_producto")
            )
        ''')
        
        conexion.commit()
        print("✓ Tablas 'users' e 'inventario' listas.\n")
        
    except sqlite3.Error as e:
        print(f"Error al crear tabla: {e}")

def insertar_usuario(conexion):
    """Inserta un usuario en la tabla users"""
    try:
        cursor = conexion.cursor()
        
        # Solicitar datos al usuario
        name = input("Ingresa el nombre de usuario: ").strip()
        password = input("Ingresa la contraseña: ").strip()
        correo = input("Ingresa el correo electrónico: ").strip()
        
        # Solicitar si es Admin (0 = No, 1 = Sí)
        while True:
            admin_input = input("¿Es administrador? (s/n): ").strip().lower()
            if admin_input == 's':
                admin = 1
                break
            elif admin_input == 'n':
                admin = 0
                break
            else:
                print("Por favor ingresa 's' o 'n'.")
        
        # Validación básica
        if not name or not password or not correo:
            print("Todos los campos son obligatorios.\n")
            return
        
        # Insertar datos
        cursor.execute('''
            INSERT INTO users (name, password, "correo electronico", Admin)
            VALUES (?, ?, ?, ?)
        ''', (name, password, correo, admin))
        
        conexion.commit()
        print("\n✓ Usuario insertado correctamente.\n")
        
    except sqlite3.IntegrityError as e:
        print(f"\n✗ Error: El nombre de usuario, contraseña o correo ya existen.\n")
        conexion.rollback()
    except sqlite3.Error as e:
        print(f"\n✗ Error al insertar: {e}\n")
        conexion.rollback()

def main():
    """Función principal"""
    conexion = conectar_db()
    
    if conexion is None:
        return
    
    # Crear tabla si no existe
    crear_tabla_si_no_existe(conexion)
    
    print("=== Sistema de Inserción de Usuarios ===\n")
    
    while True:
        insertar_usuario(conexion)
        
        otra = input("¿Deseas insertar otro usuario? (s/n): ").strip().lower()
        if otra != 's':
            break
    
    conexion.close()
    print("\nConexión cerrada.")

if __name__ == "__main__":
    main()