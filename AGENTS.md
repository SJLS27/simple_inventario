# AGENTS

## Overview
This repo is a simple desktop inventory system built with HTML/CSS/JS for the UI and Rust (Tauri) for backend logic and SQLite access.


## Architecture map
- Frontend (static): `src/index.html`, `src/main.js`, `src/styles.css`
- Menu pages: `src/menu/` (subfolders `inventario/`, `ventas/`, `compras/`)
- Shared UI helpers: `src/menu/currency-toggle.js`
- Tauri backend: `src-tauri/src/main.rs`
- Tauri config: `src-tauri/tauri.conf.json`
- SQLite DB: `src/database/database.db` (auto-creada por el backend)

## Backend commands and data flow
- Tauri commands are defined in `src-tauri/src/main.rs` and exposed to the frontend.
- The backend connects to SQLite and reads/writes `users` and `inventario` tables.
- DB lookup uses a fallback search for the DB file; the canonical path is `src/database/database.db`.
- Inventario queries: `listar_inventarios`, `obtener_inventario_por_id`, `obtener_inventario_por_nombre`.
- Stock mutations: `registrar_venta` (decrementa stock) and `registrar_compra` (incrementa stock).
- Ventas/Compras UI uses `listar_inventarios` for autocomplete suggestions and registers stock updates via the commands above.
- Ventas guarda las ventas del día en `localStorage` (`ventas_diarias`) y persiste el listado en curso (`ventas_en_curso`) para sobrevivir recargas del webview; el cierre del día consume `ventas_diarias`.
- Currency display toggle uses the BCV rate stored in `localStorage` (`tasa_bcv`) and switches between USD/Bs with `currency-toggle.js`.
- Receipt generation: `generar_recibo_ventas` produces a PDF in `Documentos/recibos/` with date-number naming; admin password is required only for day-close receipts when the session is not admin.
- Admin password check helper: `validar_password_admin`.

## Database setup
- El backend Rust crea el archivo `src/database/database.db` si no existe y ejecuta el DDL de `users` e `inventario` al iniciar.
- Semilla automática: si no hay usuario `user`, se inserta `user/user` con Admin=1.
- Si necesitas reiniciar, elimina `src/database/database.db` y vuelve a lanzar la app.

Note: passwords are stored in plain text in the local DB. Keep this for development only.

## Build and run (typical)
- Ensure Rust toolchain and Tauri prerequisites are installed.
- If the Tauri CLI is available:
  - `cargo tauri dev`
  - `cargo tauri build`

## Conventions
- Keep UI assets under `src/` and backend logic in `src-tauri/`.
- Prefer minimal HTML/CSS/JS without extra build tooling.
- Use Spanish identifiers for DB fields and UI labels to match existing schema.

## When changing the schema
- Update the SQLite DDL in `src-tauri/src/main.rs` (función `ensure_db_initialized`).
- Update backend queries in `src-tauri/src/main.rs`.
- Update UI fields and labels in `src/` or `src/menu/`.
