// main.js - moved from inline scripts in index.html
// Handles Tauri invocation, login form binding, sessionStorage admin flag and diagnostics

// Tauri v2: robust invoke helper (uses window.__TAURI__.core.invoke when available)
function tauriInvoke(command, payload) {
    // modern: __TAURI__ with core.invoke (Tauri v2)
    if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === 'function') {
        return window.__TAURI__.core.invoke(command, payload);
    }
    // fallback older: window.__TAURI__.tauri.invoke or window.tauri.invoke
    if (window.__TAURI__ && window.__TAURI__.tauri && typeof window.__TAURI__.tauri.invoke === 'function') {
        return window.__TAURI__.tauri.invoke(command, payload);
    }
    if (window.tauri && typeof window.tauri.invoke === 'function') {
        return window.tauri.invoke(command, payload);
    }
    const msg = 'API de Tauri no encontrada en el webview. Ejecuta la app con `tauri dev` o asegúrate de servir la página desde Tauri.';
    console.error('Tauri API missing:', msg);
    return Promise.reject(new Error(msg));
}

// Helper to detect Tauri presence
function isTauriAvailable() {
    return (typeof window.__TAURI__ !== 'undefined' && (window.__TAURI__.core || window.__TAURI__.tauri)) || (typeof window.tauri !== 'undefined');
}

// Bind login form
document.addEventListener('DOMContentLoaded', function() {
    const form = document.getElementById('login-form');
    const btnClose = document.getElementById('btn-close');
    const mensaje = document.getElementById('mensaje');

    if (!form) return;

    form.addEventListener('submit', async function(event) {
        event.preventDefault();
        const usuario = document.getElementById('u').value;
        const contrasena = document.getElementById('p').value;

        // Check Tauri availability
        if (!isTauriAvailable()) {
            mensaje.innerHTML = `<p style="color: red;">La API de Tauri no está disponible. Ejecuta la app con <code>cargo tauri dev</code> o sirve esta página desde Tauri.</p>`;
            return;
        }

        try {
            const resultado = await tauriInvoke('validar_login', { usuario, contrasena });
            if (resultado.success) {
                mensaje.innerHTML = `<p style="color: green;">✓ ${resultado.message}</p>`;
                try {
                    sessionStorage.setItem('is_admin', resultado.is_admin ? '1' : '0');
                } catch (e) {
                    console.warn('No se pudo usar sessionStorage:', e);
                }
                setTimeout(() => { window.location.href = 'menu/index.html'; }, 700);
            } else {
                mensaje.innerHTML = `<p style="color: red;">✗ ${resultado.message}</p>`;
            }
        } catch (err) {
            mensaje.innerHTML = `<p style="color: red;">Error: ${err}</p>`;
        }
    });

    if (btnClose) btnClose.addEventListener('click', function(){ window.close(); });

    // Diagnostic banner for non-tauri contexts
    (function(){
        var isTauri = isTauriAvailable();
        if (!isTauri) {
            var b = document.getElementById('tauri-missing-banner');
            if (b) b.style.display = 'block';
            console.error('Tauri API missing: window.__TAURI__ / window.tauri is undefined. Ejecuta: cargo tauri dev (o revisa webkit2gtk).');
            console.info('navigator.userAgent =', navigator.userAgent);
        } else {
            console.info('Tauri API disponible en webview.');
        }
        window.isTauri = function(){ return isTauri; };
    })();
});
