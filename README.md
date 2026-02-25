<pre style="font-size: 14px; line-height: 0.9; font-family: 'monospace'; background-color: transparent; border: none;">
<span style="color: #FF69B4;">#       _____ _____ __  __ _____  _      ______   _____ _   ___      ________ _   _ _______ ____  _______     __</span>
<span style="color: #FF69B4;">#      / ____|_   _|  \/  |  __ \| |    |  ____| |_   _| \ | \ \    / /  ____| \ | |__   __/ __ \|  __ \ \   / /</span>
<span style="color: #FFA07A;">#     | (___   | | | \  / | |__) | |    | |__      | | |  \| |\ \  / /| |__  |  \| |  | | | |  | | |__) \ \_/ / </span>
<span style="color: #FFA07A;">#      \___ \  | | | |\/| |  ___/| |    |  __|     | | | . ` | \ \/ / |  __| | . ` |  | | | |  | |  _  / \   /  </span>
<span style="color: #98FB98;">#      ____) |_| |_| |  | | |    | |____| |____   _| |_| |\  |  \  /  | |____| |\  |  | | | |__| | | \ \  | |   </span>
<span style="color: #98FB98;">#     |_____/|_____|_|  |_|_|    |______|______| |_____|_| \_|   \/   |______|_| \_|  |_|  \____/|_|  \_\ |_|   </span>
</pre>                                                                                                           

## 📸 Screenshots | Capturas

| English | Español |
| --- | --- |
| **Login**<br>![Login screen](assets/login.png) | **Inicio de sesión**<br>![Pantalla de inicio de sesión](assets/login.png) |
| **Main menu**<br>![Main menu](assets/menu-principal.png) | **Menú principal**<br>![Menú principal](assets/menu-principal.png) |
| **Inventory**<br>![Inventory view](assets/inventario.png) | **Inventario**<br>![Vista de inventario](assets/inventario.png) |
| **Purchases**<br>![Purchases module](assets/compras.png) | **Compras**<br>![Módulo de compras](assets/compras.png) |
| **Sales**<br>![Sales module](assets/ventas.png) | **Ventas**<br>![Módulo de ventas](assets/ventas.png) |
| **User management**<br>![User management](assets/usuarios.png) | **Gestión de usuarios**<br>![Gestión de usuarios](assets/usuarios.png) |

## 📦 Acerca del Proyecto

Este es un sistema de inventario simple, rápido y seguro. La interfaz de usuario está construida con **HTML** puro para mantenerla ligera, mientras que toda la lógica de negocio, el acceso a datos y el procesamiento se ejecutan en el backend usando **Rust**. Todo esto está empaquetado en una aplicación de escritorio nativa gracias a **Tauri**.

## ✨ Características Principales

* **Rendimiento Nativo:** El motor en Rust garantiza una ejecución ultrarrápida y un consumo mínimo de memoria RAM.
* **Almacenamiento Local (Offline-first):** Los datos se guardan de forma segura en tu máquina mediante una base de datos **SQLite** integrada. No requiere conexión a internet.
* **Interfaz Minimalista:** Un diseño en HTML limpio y sin complicaciones para gestionar tu inventario sin distracciones.
* **Multiplataforma:** Listo para ser compilado y ejecutado en Windows, macOS y Linux.

## 🚀 Requisitos Previos

Antes de clonar y ejecutar el proyecto, asegúrate de tener instalado tu entorno de desarrollo:

* [Rust (y Cargo)](https://www.rust-lang.org/tools/install)
* [Dependencias del sistema para Tauri](https://tauri.app/v1/guides/getting-started/prerequisites) (Varía según tu sistema operativo: C++ Build Tools en Windows, `webkit2gtk` en Linux, etc.)

## Este proyecto corre gracias a las tecnologias:

<div align="center">
  <pre style="background-color: transparent; border: none; font-size: 16px; font-family: monospace;">
<span style="color: #F46623; font-weight: bold;">      (🦀) RUST   </span>            <span style="color: #FFC131; font-weight: bold;">   (∞) TAURI   </span>            <span style="color: #0F80CC; font-weight: bold;">   (≡) SQLITE   </span>



<span style="color: #F46623;">       _~^~^~_    </span>            <span style="color: #FFC131;">    .-.   .-.    </span>            <span style="color: #0F80CC;">    ________    </span>
<span style="color: #F46623;">   \) /  o o  \ (/</span>            <span style="color: #FFC131;">   /   \ /   \   </span>            <span style="color: #0F80CC;">   /=======/|   </span>
<span style="color: #F46623;">     '_   v   _'  </span>            <span style="color: #FFC131;">   \    X    /   </span>            <span style="color: #0F80CC;">  | SQLite | |  </span>
<span style="color: #F46623;">     / '-----' \  </span>            <span style="color: #FFC131;">    '-'   '-'    </span>            <span style="color: #0F80CC;">  |   v3   |/   </span>
<span style="color: #F46623;">                  </span>            <span style="color: #FFC131;">                 </span>            <span style="color: #0F80CC;">   --------    </span>
  </pre>
</div>
