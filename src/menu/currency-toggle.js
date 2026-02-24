(function () {
    var STORAGE_KEYS = {
        bcv: 'tasa_bcv',
        currency: 'moneda_activa'
    };

    var memoryStore = {};

    function getStoredValue(key) {
        try {
            return localStorage.getItem(key);
        } catch (e) {
            return Object.prototype.hasOwnProperty.call(memoryStore, key) ? memoryStore[key] : null;
        }
    }

    function setStoredValue(key, value) {
        try {
            localStorage.setItem(key, value);
        } catch (e) {
            memoryStore[key] = value;
        }
    }

    function normalizeRateInput(value) {
        return String(value || '').trim().replace(',', '.');
    }

    function readRate() {
        var stored = getStoredValue(STORAGE_KEYS.bcv);
        if (!stored) return null;
        var parsed = parseFloat(stored);
        return Number.isFinite(parsed) ? parsed : null;
    }

    function formatRate(value) {
        if (!Number.isFinite(value)) return 'Bs --';
        return 'Bs ' + value.toFixed(2);
    }

    function getActiveCurrency() {
        var stored = getStoredValue(STORAGE_KEYS.currency);
        return stored === 'usd' ? 'usd' : 'ves';
    }

    function setActiveCurrency(kind) {
        setStoredValue(STORAGE_KEYS.currency, kind);
    }

    function formatAmount(value, currency, rate) {
        if (!Number.isFinite(value)) return '--';
        if (currency === 'usd') {
            return '$ ' + value.toFixed(2);
        }
        if (!Number.isFinite(rate) || rate <= 0) return 'Bs --';
        return 'Bs ' + (value * rate).toFixed(2);
    }

    function updateDisplays() {
        var currency = getActiveCurrency();
        var rateValue = readRate();

        document.querySelectorAll('[data-rate-label]').forEach(function (el) {
            el.textContent = 'BCV';
        });
        document.querySelectorAll('[data-rate-value]').forEach(function (el) {
            el.textContent = formatRate(rateValue);
        });
        document.querySelectorAll('[data-currency-label]').forEach(function (el) {
            el.textContent = currency === 'usd' ? 'USD' : 'Bs';
        });
        document.querySelectorAll('[data-currency-toggle]').forEach(function (el) {
            el.textContent = currency === 'usd' ? 'Mostrar en bolivares' : 'Mostrar en dolares';
            el.setAttribute('aria-pressed', currency === 'usd' ? 'true' : 'false');
        });

        document.querySelectorAll('[data-amount]').forEach(function (el) {
            var raw = parseFloat(el.getAttribute('data-amount') || '');
            el.textContent = formatAmount(raw, currency, rateValue);
        });
    }

    function setupInputs() {
        document.querySelectorAll('[data-rate-input]').forEach(function (input) {
            var stored = getStoredValue(STORAGE_KEYS.bcv);
            if (stored) {
                input.value = stored;
            }

            input.addEventListener('input', function () {
                var normalized = normalizeRateInput(input.value);
                setStoredValue(STORAGE_KEYS.bcv, normalized);
                updateDisplays();
            });
        });
    }

    function setupToggles() {
        document.querySelectorAll('[data-currency-toggle]').forEach(function (button) {
            button.addEventListener('click', function () {
                var currency = getActiveCurrency();
                setActiveCurrency(currency === 'ves' ? 'usd' : 'ves');
                updateDisplays();
            });
        });
    }

    document.addEventListener('DOMContentLoaded', function () {
        if (!getStoredValue(STORAGE_KEYS.currency)) {
            setActiveCurrency('ves');
        }
        setupInputs();
        setupToggles();
        updateDisplays();
    });

    window.currencyToggle = {
        update: updateDisplays,
        getRate: readRate,
        getCurrency: getActiveCurrency
    };
})();
