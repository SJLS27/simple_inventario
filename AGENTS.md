# AGENTS

## Overview
This repo is a simple desktop inventory system built with HTML/CSS/JS for the UI and Rust (Tauri) for backend logic and SQLite access.

## Architecture map
- Frontend (static): `src/index.html`, `src/main.js`, `src/styles.css`
- Menu pages: `src/menu/` (subfolders `inventario/`, `ventas/`)
- Tauri backend: `src-tauri/src/main.rs`
- Tauri config: `src-tauri/tauri.conf.json`
- SQLite DB: `src/database/database.db`
- DB init script: `src/database/init_db.py`

## Backend commands and data flow
- Tauri commands are defined in `src-tauri/src/main.rs` and exposed to the frontend.
- The backend connects to SQLite and reads/writes `users` and `inventario` tables.
- DB lookup uses a fallback search for the DB file; the canonical path is `src/database/database.db`.

## Database setup
- Create tables (users + inventario):
  - `python3 src/database/init_db.py`
- Add a test user (user/user, Admin=1):
  - `python3 src/database/init_db.py --add-user`
- Add a test user with Admin=0:
  - `python3 src/database/init_db.py --add-user --admin 0`

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
- Update the SQLite DDL in `src/database/init_db.py`.
- Update backend queries in `src-tauri/src/main.rs`.
- Update UI fields and labels in `src/` or `src/menu/`.
