// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::Connection;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::{PathBuf, Path};
use std::env;
use tauri_plugin_opener;

fn find_db_path() -> PathBuf {
    // Prioritized candidate locations relative to current working dir or executable
    let mut candidates = Vec::new();

    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd.join("src").join("database").join("database.db"));
        candidates.push(cwd.join("../src/database/database.db"));
    }

    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("../src/database/database.db"));
            candidates.push(dir.join("../../src/database/database.db"));
        }
    }

    // Add the original absolute path from the project as last-resort (keep as fallback)
    candidates.push(PathBuf::from("/home/elsanti/Documentos/proyecto inventario/src/database/database.db"));

    for p in candidates {
        if p.exists() {
            println!("[debug] seleccionando base de datos en {}", p.display());
            return p;
        }
    }

    // If none exists, return the first candidate (cwd/src/database/database.db) so errors are clear
    let default = env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("src/database/database.db");
    println!("[warn] no se encontró el archivo de base de datos en las rutas habituales; usando {}", default.display());
    default
}

#[derive(Serialize, Deserialize)]
struct LoginResponse {
    success: bool,
    message: String,
    is_admin: bool,
}

#[derive(Serialize, Deserialize)]
struct InventarioItem {
    id: i64,
    nombre: String,
    precio: f64,
    cantidad: i64,
}

#[tauri::command]
fn validar_login(usuario: String, contrasena: String) -> LoginResponse {
    // Determinar la ruta a la base de datos usando búsqueda de candidatos
    let db_path = find_db_path();

    println!(
        "[debug] validar_login called for user='{}' using DB path={}",
        usuario,
        db_path.display()
    );

    // Conectar a la base de datos
    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return LoginResponse {
            success: false,
            message: format!("Error al conectar: {} (ruta={})", e, db_path.display()),
            is_admin: false,
        },
    };

    // Consultar la base de datos: la tabla en el proyecto se llama `users`
    let mut stmt = match conn.prepare(
        "SELECT password, Admin FROM users WHERE name = ?1"
    ) {
        Ok(s) => s,
        Err(e) => return LoginResponse {
            success: false,
            message: format!("Error en la consulta: {}", e),
            is_admin: false,
        },
    };

    let result = stmt.query_row(
        rusqlite::params![&usuario],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i32>(1)?,
            ))
        },
    );

    match result {
        Ok((pass_db, admin_value)) => {
            println!("[debug] usuario encontrado. admin_value={}", admin_value);
            if contrasena == pass_db {
                let es_admin = admin_value == 1;

                // Crear archivo con estado admin
                crear_archivo_admin(es_admin);

                LoginResponse {
                    success: true,
                    message: "Login exitoso".to_string(),
                    is_admin: es_admin,
                }
            } else {
                println!("[debug] contrasena incorrecta para user={}", usuario);
                LoginResponse {
                    success: false,
                    message: "Contraseña incorrecta".to_string(),
                    is_admin: false,
                }
            }
        }
        Err(e) => {
            println!("[debug] usuario no encontrado: {}", e);
            LoginResponse {
                success: false,
                message: "Usuario no encontrado".to_string(),
                is_admin: false,
            }
        }
    }
}

#[tauri::command]
fn listar_inventarios() -> Result<Vec<InventarioItem>, String> {
    let db_path = find_db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Error al conectar: {} (ruta={})", e, db_path.display()))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, nombre_producto AS nombre, CAST(precio_producto AS REAL) AS precio, CAST(cantidad_producto AS INTEGER) AS cantidad \
             FROM inventario ORDER BY id, nombre_producto, precio_producto, cantidad_producto",
        )
        .map_err(|e| format!("Error en la consulta: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(InventarioItem {
                id: row.get(0)?,
                nombre: row.get(1)?,
                precio: row.get(2)?,
                cantidad: row.get(3)?,
            })
        })
        .map_err(|e| format!("Error al leer inventarios: {}", e))?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| format!("Error en fila: {}", e))?);
    }

    Ok(items)
}

#[tauri::command]
fn actualizar_inventario(id: i64, nombre: String, precio: f64, cantidad: i64) -> Result<(), String> {
    let db_path = find_db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Error al conectar: {} (ruta={})", e, db_path.display()))?;

    let affected = conn
        .execute(
            "UPDATE inventario SET nombre_producto = ?1, precio_producto = ?2, cantidad_producto = ?3 WHERE id = ?4",
            rusqlite::params![nombre, precio, cantidad, id],
        )
        .map_err(|e| format!("Error al actualizar: {}", e))?;

    if affected == 0 {
        return Err("No se encontro el registro para actualizar".to_string());
    }

    Ok(())
}

#[tauri::command]
fn cerrar_ventana(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| format!("Error al cerrar la ventana: {}", e))
}

#[tauri::command]
fn insertar_inventario(id: i64, nombre: String, precio: f64, cantidad: Option<i64>) -> Result<(), String> {
    let db_path = find_db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Error al conectar: {} (ruta={})", e, db_path.display()))?;

    conn.execute(
        "INSERT INTO inventario (id, nombre_producto, precio_producto, cantidad_producto) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, nombre, precio, cantidad],
    )
    .map_err(|e| format!("Error al insertar: {}", e))?;

    Ok(())
}

fn crear_archivo_admin(es_admin: bool) {
    let admin_value = if es_admin { 1 } else { 0 };
    let contenido = format!("admin={}", admin_value);

    // Escribir en un directorio seguro que normalmente no está observado por herramientas
    // de desarrollo (por ejemplo, `cargo tauri dev`). Usamos el directorio temporal del
    // sistema para evitar que la creación/actualización del archivo dispare recargas.
    let ruta = env::temp_dir().join("ventas_admin.conf");

    if let Err(e) = fs::write(&ruta, contenido) {
        println!("[warn] no se pudo escribir ventas_admin.conf en {}: {}", ruta.display(), e);
    } else {
        println!("[debug] ventas_admin.conf escrito en {}", ruta.display());
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            validar_login,
            listar_inventarios,
            actualizar_inventario,
            insertar_inventario,
            cerrar_ventana,
            greet
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Mantener un saludo simple disponible desde la UI (por compatibilidad)
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}