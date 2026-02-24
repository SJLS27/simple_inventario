// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::Connection;
use serde::{Serialize, Deserialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::env;
use chrono::Local;
use printpdf::*;
use dirs;
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

#[derive(Serialize, Deserialize, Clone)]
struct VentaItem {
    id: i64,
    nombre: String,
    precio: f64,
    cantidad: i64,
    subtotal: f64,
}

#[derive(Serialize, Deserialize)]
struct ReciboRequest {
    ventas: Vec<VentaItem>,
    total: f64,
    es_cierre_dia: bool,
    admin_password: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ReciboResponse {
    ruta: String,
}

fn obtener_item_por_id(conn: &Connection, id: i64) -> Result<InventarioItem, String> {
    conn.query_row(
        "SELECT id, nombre_producto AS nombre, CAST(precio_producto AS REAL) AS precio, \
         COALESCE(CAST(cantidad_producto AS INTEGER), 0) AS cantidad FROM inventario WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(InventarioItem {
                id: row.get(0)?,
                nombre: row.get(1)?,
                precio: row.get(2)?,
                cantidad: row.get(3)?,
            })
        },
    )
    .map_err(|e| format!("No se encontro el producto: {}", e))
}

fn obtener_item_por_nombre(conn: &Connection, nombre: &str) -> Result<InventarioItem, String> {
    conn.query_row(
        "SELECT id, nombre_producto AS nombre, CAST(precio_producto AS REAL) AS precio, \
         COALESCE(CAST(cantidad_producto AS INTEGER), 0) AS cantidad \
         FROM inventario WHERE LOWER(nombre_producto) = LOWER(?1) LIMIT 1",
        rusqlite::params![nombre],
        |row| {
            Ok(InventarioItem {
                id: row.get(0)?,
                nombre: row.get(1)?,
                precio: row.get(2)?,
                cantidad: row.get(3)?,
            })
        },
    )
    .map_err(|e| format!("No se encontro el producto: {}", e))
}

fn get_documentos_recibos_dir() -> Result<PathBuf, String> {
    let documentos = dirs::document_dir().ok_or_else(|| "No se pudo obtener la carpeta Documentos".to_string())?;
    let recibos_dir = documentos.join("recibos");
    if let Err(e) = fs::create_dir_all(&recibos_dir) {
        return Err(format!("No se pudo crear la carpeta de recibos: {}", e));
    }
    Ok(recibos_dir)
}

fn validar_admin_password(password: &str) -> Result<(), String> {
    let db_path = find_db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Error al conectar: {} (ruta={})", e, db_path.display()))?;

    let mut stmt = conn
        .prepare("SELECT COUNT(1) FROM users WHERE Admin = 1 AND password = ?1")
        .map_err(|e| format!("Error en la consulta: {}", e))?;

    let count: i64 = stmt
        .query_row(rusqlite::params![password], |row| row.get(0))
        .map_err(|e| format!("Error en la consulta: {}", e))?;

    if count > 0 {
        Ok(())
    } else {
        Err("Clave de administrador incorrecta".to_string())
    }
}

#[tauri::command]
fn validar_password_admin(password: String) -> Result<(), String> {
    validar_admin_password(password.trim())
}

fn format_money(value: f64) -> String {
    format!("${:.2}", value)
}

fn format_date_stamp() -> String {
    Local::now().format("%Y%m%d").to_string()
}

fn obtener_siguiente_numero(recibos_dir: &PathBuf, date_stamp: &str) -> Result<u32, String> {
    let mut max_num = 0u32;
    if let Ok(entries) = fs::read_dir(recibos_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if !name.starts_with(date_stamp) || !name.ends_with(".pdf") {
                    continue;
                }
                let trimmed = name.trim_end_matches(".pdf");
                let parts: Vec<&str> = trimmed.split('-').collect();
                if parts.len() >= 2 {
                    if let Ok(num) = parts[parts.len() - 1].parse::<u32>() {
                        if num > max_num {
                            max_num = num;
                        }
                    }
                }
            }
        }
    }
    Ok(max_num + 1)
}

fn crear_pdf_recibo(ventas: &[VentaItem], total: f64, titulo: &str, ruta_salida: &PathBuf) -> Result<(), String> {
    let (doc, page1, layer1) = PdfDocument::new("Recibo de ventas", Mm(210.0), Mm(180.0), "Layer 1");
    let mut current_layer = doc.get_page(page1).get_layer(layer1);

    let font = doc.add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("No se pudo cargar fuente: {:?}", e))?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("No se pudo cargar fuente: {:?}", e))?;

    let start_x: f32 = 12.0;
    let start_y: f32 = 170.0;
    let min_y: f32 = 18.0;
    let fecha_texto = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let mut current_y: f32 = start_y;
    let mut page_count = 1;

    let draw_header = |layer: &PdfLayerReference, y: f32| -> f32 {
        let mut y_cursor: f32 = y;
        layer.use_text(titulo, 14.0, Mm(start_x), Mm(y_cursor), &font_bold);
        y_cursor -= 8.0;
        layer.use_text(
            format!("Fecha: {}", fecha_texto),
            10.0,
            Mm(start_x),
            Mm(y_cursor),
            &font,
        );
        y_cursor -= 10.0;
        layer.use_text(
            "ID  Producto                           Cant  Precio   Subtotal",
            9.0,
            Mm(start_x),
            Mm(y_cursor),
            &font_bold,
        );
        y_cursor - 6.0
    };

    current_y = draw_header(&current_layer, current_y);

    for venta in ventas {
        let nombre = if venta.nombre.len() > 28 {
            let mut nombre_truncado = venta.nombre.chars().take(25).collect::<String>();
            nombre_truncado.push_str("...");
            nombre_truncado
        } else {
            venta.nombre.clone()
        };
        let line = format!(
            "{:<4} {:<32} {:>4} {:>7} {:>9}",
            venta.id,
            nombre,
            venta.cantidad,
            format_money(venta.precio),
            format_money(venta.subtotal)
        );

        if current_y - 5.0 < min_y {
            page_count += 1;
            let (page, layer) = doc.add_page(Mm(210.0), Mm(180.0), format!("Layer {}", page_count));
            current_layer = doc.get_page(page).get_layer(layer);
            current_y = draw_header(&current_layer, start_y);
        }

        current_y -= 5.0;
        current_layer.use_text(line, 9.0, Mm(start_x), Mm(current_y), &font);
    }

    if current_y - 10.0 < min_y {
        page_count += 1;
        let (page, layer) = doc.add_page(Mm(210.0), Mm(180.0), format!("Layer {}", page_count));
        current_layer = doc.get_page(page).get_layer(layer);
        current_y = draw_header(&current_layer, start_y);
    }

    current_y -= 10.0;
    current_layer.use_text(
        format!("Total: {}", format_money(total)),
        12.0,
        Mm(start_x),
        Mm(current_y),
        &font_bold,
    );

    let pdf_bytes = doc.save_to_bytes()
        .map_err(|e| format!("Error al generar el PDF: {:?}", e))?;

    let mut file = fs::File::create(ruta_salida)
        .map_err(|e| format!("No se pudo crear el archivo PDF: {}", e))?;
    file.write_all(&pdf_bytes)
        .map_err(|e| format!("No se pudo escribir el PDF: {}", e))?;

    Ok(())
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
fn obtener_inventario_por_id(id: i64) -> Result<InventarioItem, String> {
    let db_path = find_db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Error al conectar: {} (ruta={})", e, db_path.display()))?;

    obtener_item_por_id(&conn, id)
}

#[tauri::command]
fn obtener_inventario_por_nombre(nombre: String) -> Result<InventarioItem, String> {
    let db_path = find_db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Error al conectar: {} (ruta={})", e, db_path.display()))?;

    obtener_item_por_nombre(&conn, &nombre)
}

#[tauri::command]
fn registrar_venta(id: i64, cantidad: i64) -> Result<InventarioItem, String> {
    if cantidad <= 0 {
        return Err("La cantidad debe ser mayor a 0".to_string());
    }

    let db_path = find_db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Error al conectar: {} (ruta={})", e, db_path.display()))?;

    let item = obtener_item_por_id(&conn, id)?;
    if item.cantidad < cantidad {
        return Err(format!(
            "Stock insuficiente. Disponible: {}",
            item.cantidad
        ));
    }

    let nueva_cantidad = item.cantidad - cantidad;
    conn.execute(
        "UPDATE inventario SET cantidad_producto = ?1 WHERE id = ?2",
        rusqlite::params![nueva_cantidad, id],
    )
    .map_err(|e| format!("Error al actualizar: {}", e))?;

    Ok(InventarioItem {
        cantidad: nueva_cantidad,
        ..item
    })
}

#[tauri::command]
fn registrar_compra(id: i64, cantidad: i64) -> Result<InventarioItem, String> {
    if cantidad <= 0 {
        return Err("La cantidad debe ser mayor a 0".to_string());
    }

    let db_path = find_db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Error al conectar: {} (ruta={})", e, db_path.display()))?;

    let item = obtener_item_por_id(&conn, id)?;
    let nueva_cantidad = item.cantidad + cantidad;
    conn.execute(
        "UPDATE inventario SET cantidad_producto = ?1 WHERE id = ?2",
        rusqlite::params![nueva_cantidad, id],
    )
    .map_err(|e| format!("Error al actualizar: {}", e))?;

    Ok(InventarioItem {
        cantidad: nueva_cantidad,
        ..item
    })
}

#[tauri::command]
fn generar_recibo_ventas(payload: ReciboRequest) -> Result<ReciboResponse, String> {
    if payload.ventas.is_empty() {
        return Err("No hay ventas para generar el recibo".to_string());
    }

    let es_admin = leer_estado_admin().unwrap_or(false);
    if payload.es_cierre_dia && !es_admin {
        let pass = payload
            .admin_password
            .as_deref()
            .ok_or_else(|| "Se requiere clave de administrador".to_string())?;
        validar_admin_password(pass)?;
    }

    let recibos_dir = get_documentos_recibos_dir()?;
    let date_stamp = format_date_stamp();
    let numero = obtener_siguiente_numero(&recibos_dir, &date_stamp)?;
    let file_name = format!("{}-{}.pdf", date_stamp, numero);
    let ruta = recibos_dir.join(file_name);

    let titulo = if payload.es_cierre_dia {
        "Recibo cierre del dia"
    } else {
        "Recibo cliente"
    };

    crear_pdf_recibo(&payload.ventas, payload.total, titulo, &ruta)?;

    Ok(ReciboResponse {
        ruta: ruta.display().to_string(),
    })
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

fn leer_estado_admin() -> Option<bool> {
    let ruta = env::temp_dir().join("ventas_admin.conf");
    let contenido = fs::read_to_string(&ruta).ok()?;
    let partes: Vec<&str> = contenido.trim().split('=').collect();
    if partes.len() != 2 {
        return None;
    }
    match partes[1].trim() {
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            validar_login,
            listar_inventarios,
            obtener_inventario_por_id,
            obtener_inventario_por_nombre,
            actualizar_inventario,
            insertar_inventario,
            registrar_venta,
            registrar_compra,
            generar_recibo_ventas,
            validar_password_admin,
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
