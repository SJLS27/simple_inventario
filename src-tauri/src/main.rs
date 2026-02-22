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

fn crear_archivo_admin(es_admin: bool) {
    let admin_value = if es_admin { 1 } else { 0 };
    let contenido = format!("admin={}", admin_value);

    // Guardar en la carpeta del proyecto (directorio de trabajo actual)
    let ruta = env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("admin.conf");

    if let Err(e) = fs::write(&ruta, contenido) {
        println!("[warn] no se pudo escribir admin.conf: {}", e);
    } else {
        println!("[debug] admin.conf escrito en {}", ruta.display());
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![validar_login, greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Mantener un saludo simple disponible desde la UI (por compatibilidad)
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}