function tauriInvoke(command, payload) {
    if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === 'function') {
        return window.__TAURI__.core.invoke(command, payload);
    }
    if (window.__TAURI__ && window.__TAURI__.tauri && typeof window.__TAURI__.tauri.invoke === 'function') {
        return window.__TAURI__.tauri.invoke(command, payload);
    }
    if (window.tauri && typeof window.tauri.invoke === 'function') {
        return window.tauri.invoke(command, payload);
    }
    const msg = 'API de Tauri no encontrada. Ejecuta la app con `cargo tauri dev`.';
    console.error('Tauri API missing:', msg);
    return Promise.reject(new Error(msg));
}

function getAdminFlag() {
    try {
        return sessionStorage.getItem('is_admin') === '1';
    } catch (e) {
        console.warn('No se pudo leer sessionStorage:', e);
        return false;
    }
}

function normalizeValue(value) {
    return String(value ?? '').trim();
}

function createCell(text, className) {
    const td = document.createElement('td');
    if (className) td.className = className;
    td.textContent = text;
    return td;
}

function formatPrecio(value) {
    const rate = window.currencyToggle && typeof window.currencyToggle.getRate === 'function'
        ? window.currencyToggle.getRate()
        : null;
    const currency = window.currencyToggle && typeof window.currencyToggle.getCurrency === 'function'
        ? window.currencyToggle.getCurrency()
        : 'ves';
    if (currency === 'usd') {
        return '$ ' + Number(value).toFixed(2);
    }
    if (!Number.isFinite(rate) || rate <= 0) return 'Bs --';
    return 'Bs ' + (Number(value) * rate).toFixed(2);
}

function createInputCell(value, type, disabled) {
    const td = document.createElement('td');
    const input = document.createElement('input');
    input.type = type;
    input.value = value;
    input.disabled = disabled;
    td.appendChild(input);
    return { td, input };
}

function debounce(fn, delayMs) {
    let handle = null;
    return function (...args) {
        if (handle) clearTimeout(handle);
        handle = setTimeout(() => fn.apply(this, args), delayMs);
    };
}

document.addEventListener('DOMContentLoaded', () => {
    const searchInput = document.getElementById('search');
    const bodyEl = document.getElementById('inventario-body');
    const statusEl = document.getElementById('status');
    const roleEl = document.getElementById('role-indicator');
    const btnVolver = document.getElementById('btn-volver');
    const btnRecargar = document.getElementById('btn-recargar');
    const addSection = document.getElementById('add-product');
    const addStatusEl = document.getElementById('add-status');
    const addId = document.getElementById('add-id');
    const addNombre = document.getElementById('add-nombre');
    const addPrecio = document.getElementById('add-precio');
    const addCantidad = document.getElementById('add-cantidad');
    const btnAdd = document.getElementById('btn-add');

    const isAdmin = getAdminFlag();
    if (roleEl) {
        roleEl.textContent = isAdmin ? 'Tipo 1: edicion habilitada' : 'Tipo 0: solo lectura';
    }
    if (!isAdmin && addSection) {
        addSection.classList.add('hidden');
    }

    let inventarios = [];

    function setStatus(message, isError) {
        if (!statusEl) return;
        statusEl.textContent = message;
        statusEl.style.color = isError ? '#ef4444' : '#16a34a';
    }

    function setAddStatus(message, isError) {
        if (!addStatusEl) return;
        addStatusEl.textContent = message;
        addStatusEl.style.color = isError ? '#ef4444' : '#16a34a';
    }

    function filterInventarios(term) {
        if (!term) return inventarios;
        const lowerTerm = term.toLowerCase();
        return inventarios.filter((item) => {
            const idMatch = String(item.id).includes(lowerTerm);
            const nameMatch = String(item.nombre).toLowerCase().includes(lowerTerm);
            return idMatch || nameMatch;
        });
    }

    function renderTable(data) {
        if (!bodyEl) return;
        bodyEl.innerHTML = '';
        if (!data.length) {
            const emptyRow = document.createElement('tr');
            const emptyCell = document.createElement('td');
            emptyCell.colSpan = 4;
            emptyCell.textContent = 'No hay resultados.';
            emptyRow.appendChild(emptyCell);
            bodyEl.appendChild(emptyRow);
            return;
        }

        data.forEach((item) => {
            const row = document.createElement('tr');
            row.appendChild(createCell(item.id, 'tree-id'));

            if (isAdmin) {
                const nombreCell = createInputCell(item.nombre, 'text', false);
                const precioCell = createInputCell(item.precio, 'number', false);
                precioCell.input.step = '0.01';
                const cantidadCell = createInputCell(item.cantidad, 'number', false);
                cantidadCell.input.step = '1';

                const debouncedUpdate = debounce(async () => {
                    const nombre = normalizeValue(nombreCell.input.value);
                    const precio = parseFloat(precioCell.input.value);
                    const cantidad = parseInt(cantidadCell.input.value, 10);

                    if (!nombre || Number.isNaN(precio) || Number.isNaN(cantidad)) {
                        setStatus('Revisa los valores antes de guardar.', true);
                        return;
                    }

                    try {
                        await tauriInvoke('actualizar_inventario', {
                            id: item.id,
                            nombre,
                            precio,
                            cantidad,
                        });
                        setStatus('Cambios guardados.', false);
                    } catch (err) {
                        setStatus(`Error al guardar: ${err}`, true);
                    }
                }, 400);

                [nombreCell.input, precioCell.input, cantidadCell.input].forEach((input) => {
                    input.addEventListener('input', debouncedUpdate);
                });

                row.appendChild(nombreCell.td);
                row.appendChild(precioCell.td);
                row.appendChild(cantidadCell.td);
            } else {
                row.appendChild(createCell(item.nombre));
                row.appendChild(createCell(formatPrecio(item.precio)));
                row.appendChild(createCell(item.cantidad));
            }

            bodyEl.appendChild(row);
        });
    }

    async function cargarInventarios() {
        setStatus('', false);
        try {
            inventarios = await tauriInvoke('listar_inventarios');
            renderTable(filterInventarios(normalizeValue(searchInput?.value)));
        } catch (err) {
            setStatus(`Error al cargar: ${err}`, true);
        }
    }

    if (searchInput) {
        searchInput.addEventListener('input', () => {
            renderTable(filterInventarios(normalizeValue(searchInput.value)));
        });
    }

    if (btnVolver) {
        btnVolver.addEventListener('click', () => {
            window.location.href = '../index.html';
        });
    }

    if (btnRecargar) {
        btnRecargar.addEventListener('click', () => {
            cargarInventarios();
        });
    }

    if (window.currencyToggle && typeof window.currencyToggle.update === 'function') {
        document.querySelectorAll('[data-currency-toggle]').forEach((button) => {
            button.addEventListener('click', () => {
                setTimeout(() => {
                    renderTable(filterInventarios(normalizeValue(searchInput?.value)));
                }, 0);
            });
        });
    }

    if (btnAdd) {
        btnAdd.addEventListener('click', async () => {
            if (!isAdmin) return;
            const idValue = parseInt(normalizeValue(addId?.value), 10);
            const nombre = normalizeValue(addNombre?.value);
            const precioValue = parseFloat(normalizeValue(addPrecio?.value));
            const cantidadRaw = normalizeValue(addCantidad?.value);
            const cantidadValue = cantidadRaw ? parseInt(cantidadRaw, 10) : null;

            if (!idValue || !nombre || Number.isNaN(precioValue)) {
                setAddStatus('Completa ID, nombre y precio correctamente.', true);
                return;
            }
            if (cantidadRaw && Number.isNaN(cantidadValue)) {
                setAddStatus('Cantidad invalida.', true);
                return;
            }

            try {
                await tauriInvoke('insertar_inventario', {
                    id: idValue,
                    nombre,
                    precio: precioValue,
                    cantidad: cantidadValue,
                });
                setAddStatus('Producto agregado.', false);
                if (addId) addId.value = '';
                if (addNombre) addNombre.value = '';
                if (addPrecio) addPrecio.value = '';
                if (addCantidad) addCantidad.value = '';
                await cargarInventarios();
            } catch (err) {
                setAddStatus(`Error al agregar: ${err}`, true);
            }
        });
    }

    cargarInventarios();
});
