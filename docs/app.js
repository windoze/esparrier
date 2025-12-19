/**
 * Esparrier WebUSB Configuration App
 */

// DOM Elements
const connectBtn = document.getElementById('connect-btn');
const disconnectBtn = document.getElementById('disconnect-btn');
const statusIndicator = document.getElementById('status-indicator');
const statusText = document.getElementById('status-text');

const deviceInfoSection = document.getElementById('device-info-section');
const configSection = document.getElementById('config-section');

const firmwareVersion = document.getElementById('firmware-version');
const modelId = document.getElementById('model-id');
const featureFlags = document.getElementById('feature-flags');
const ipAddress = document.getElementById('ip-address');
const serverConnected = document.getElementById('server-connected');
const deviceActive = document.getElementById('device-active');
const keepAwake = document.getElementById('keep-awake');

const refreshStatusBtn = document.getElementById('refresh-status-btn');
const toggleAwakeBtn = document.getElementById('toggle-awake-btn');

const configForm = document.getElementById('config-form');
const readConfigBtn = document.getElementById('read-config-btn');
const writeConfigBtn = document.getElementById('write-config-btn');
const rebootBtn = document.getElementById('reboot-btn');

const brightnessInput = document.getElementById('brightness');
const brightnessValue = document.getElementById('brightness-value');

const logOutput = document.getElementById('log-output');
const clearLogBtn = document.getElementById('clear-log-btn');

const langSelector = document.getElementById('lang-selector');

const firmwareWarning = document.getElementById('firmware-warning');
const webusbUrlGroup = document.getElementById('webusb-url-group');

const advancedConnectionToggle = document.getElementById('advanced-connection-toggle');
const advancedConnectionSettings = document.getElementById('advanced-connection-settings');
const connectVidInput = document.getElementById('connect-vid');
const connectPidInput = document.getElementById('connect-pid');

// Device instance
const device = new EsparrierDevice();

// Current state
let currentState = null;
let currentConfig = null;

/**
 * Version comparison helpers
 */
function compareVersion(major, minor, patch, targetMajor, targetMinor, targetPatch) {
    if (major !== targetMajor) return major - targetMajor;
    if (minor !== targetMinor) return minor - targetMinor;
    return patch - targetPatch;
}

function isVersionAtLeast(state, major, minor, patch = 0) {
    return compareVersion(state.versionMajor, state.versionMinor, state.versionPatch, major, minor, patch) >= 0;
}

/**
 * Logging functions
 */
function log(message, type = 'info') {
    const timestamp = new Date().toLocaleTimeString();
    const span = document.createElement('span');
    span.className = `log-${type}`;
    span.textContent = `[${timestamp}] ${message}\n`;
    logOutput.appendChild(span);
    logOutput.scrollTop = logOutput.scrollHeight;
}

function logInfo(message) { log(message, 'info'); }
function logSuccess(message) { log(message, 'success'); }
function logWarning(message) { log(message, 'warning'); }
function logError(message) { log(message, 'error'); }

/**
 * Update UI state
 */
function updateConnectionUI(connected) {
    if (connected) {
        statusIndicator.className = 'status-dot connected';
        statusText.textContent = i18n.t('connected');
        connectBtn.disabled = true;
        disconnectBtn.disabled = false;
        deviceInfoSection.classList.remove('hidden');
        configSection.classList.remove('hidden');
    } else {
        statusIndicator.className = 'status-dot disconnected';
        statusText.textContent = i18n.t('disconnected');
        connectBtn.disabled = false;
        disconnectBtn.disabled = true;
        deviceInfoSection.classList.add('hidden');
        configSection.classList.add('hidden');
        firmwareWarning.classList.add('hidden');
        webusbUrlGroup.classList.remove('hidden');
        // Reset keep awake button to default state
        toggleAwakeBtn.classList.remove('active');
        const btnText = toggleAwakeBtn.querySelector('span');
        if (btnText) {
            btnText.textContent = i18n.t('keepAwakeOff');
            btnText.setAttribute('data-i18n', 'keepAwakeOff');
        }
    }
}

function updateDeviceInfo(state) {
    firmwareVersion.textContent = state.version;
    modelId.textContent = state.modelName;
    featureFlags.textContent = state.features.length > 0 ? state.features.join(', ') : i18n.t('none');
    ipAddress.textContent = state.ipAddressStr;
    serverConnected.textContent = state.serverConnected ? i18n.t('yes') : i18n.t('no');
    serverConnected.style.color = state.serverConnected ? 'var(--success-color)' : 'var(--danger-color)';
    deviceActive.textContent = state.active ? i18n.t('yes') : i18n.t('no');
    deviceActive.style.color = state.active ? 'var(--success-color)' : '';
    keepAwake.textContent = state.keepAwake ? i18n.t('enabled') : i18n.t('disabled');

    // Update toggle button state
    updateKeepAwakeButton(state.keepAwake);
}

/**
 * Update the Keep Awake toggle button appearance
 */
function updateKeepAwakeButton(isAwake) {
    const btnText = toggleAwakeBtn.querySelector('span');
    if (isAwake) {
        toggleAwakeBtn.classList.add('active');
        btnText.textContent = i18n.t('keepAwakeOn');
        btnText.setAttribute('data-i18n', 'keepAwakeOn');
    } else {
        toggleAwakeBtn.classList.remove('active');
        btnText.textContent = i18n.t('keepAwakeOff');
        btnText.setAttribute('data-i18n', 'keepAwakeOff');
    }
}

/**
 * Check firmware version and update UI accordingly
 * Returns true if firmware is compatible (>= 0.6.0), false otherwise
 */
function checkFirmwareVersion(state) {
    // Hide WebUSB URL option if firmware < 0.7.0
    if (!isVersionAtLeast(state, 0, 7, 0)) {
        webusbUrlGroup.classList.add('hidden');
    } else {
        webusbUrlGroup.classList.remove('hidden');
    }

    // Show error and hide config if firmware < 0.6.0
    if (!isVersionAtLeast(state, 0, 6, 0)) {
        firmwareWarning.classList.remove('hidden');
        configSection.classList.add('hidden');
        logError(i18n.t('logFirmwareTooOld', { version: state.version }));
        return false;
    }

    firmwareWarning.classList.add('hidden');
    return true;
}

/**
 * Validate required fields and update Write Config button state
 */
function validateRequiredFields() {
    const requiredFields = [
        { id: 'ssid', value: document.getElementById('ssid').value.trim() },
        { id: 'password', value: document.getElementById('password').value },
        { id: 'server', value: document.getElementById('server').value.trim() },
        { id: 'screen_name', value: document.getElementById('screen_name').value.trim() }
    ];

    let isValid = true;
    requiredFields.forEach(field => {
        const input = document.getElementById(field.id);
        if (field.value === '') {
            input.classList.add('input-error');
            isValid = false;
        } else {
            input.classList.remove('input-error');
        }
    });

    writeConfigBtn.disabled = !isValid;
    return isValid;
}

function populateConfigForm(config) {
    // Network settings
    document.getElementById('ssid').value = config.ssid || '';
    document.getElementById('password').value = ''; // Don't show password
    document.getElementById('ip_addr').value = config.ip_addr || '';
    document.getElementById('gateway').value = config.gateway || '';

    // Server settings
    document.getElementById('server').value = config.server || '';
    document.getElementById('screen_name').value = config.screen_name || '';

    // Screen settings
    document.getElementById('screen_width').value = config.screen_width || 1920;
    document.getElementById('screen_height').value = config.screen_height || 1080;
    document.getElementById('flip_wheel').checked = config.flip_wheel || false;

    // Performance settings
    document.getElementById('polling_rate').value = config.polling_rate || 200;
    document.getElementById('jiggle_interval').value = config.jiggle_interval || 60;
    document.getElementById('brightness').value = config.brightness || 30;
    brightnessValue.textContent = config.brightness || 30;

    // USB HID identity
    document.getElementById('vid').value = (config.vid || 0x0d0a).toString(16).padStart(4, '0');
    document.getElementById('pid').value = (config.pid || 0xc0de).toString(16).padStart(4, '0');
    document.getElementById('manufacturer').value = config.manufacturer || '';
    document.getElementById('product').value = config.product || '';
    document.getElementById('serial_number').value = config.serial_number || '';
    document.getElementById('webusb_url').value = config.webusb_url || '';

    // Validate required fields after populating form
    validateRequiredFields();
}

function getConfigFromForm() {
    const config = {
        ssid: document.getElementById('ssid').value,
        server: document.getElementById('server').value,
        screen_name: document.getElementById('screen_name').value,
        screen_width: parseInt(document.getElementById('screen_width').value),
        screen_height: parseInt(document.getElementById('screen_height').value),
        flip_wheel: document.getElementById('flip_wheel').checked,
        polling_rate: parseInt(document.getElementById('polling_rate').value),
        jiggle_interval: parseInt(document.getElementById('jiggle_interval').value),
        brightness: parseInt(document.getElementById('brightness').value),
        vid: parseInt(document.getElementById('vid').value, 16),
        pid: parseInt(document.getElementById('pid').value, 16),
        manufacturer: document.getElementById('manufacturer').value,
        product: document.getElementById('product').value,
        serial_number: document.getElementById('serial_number').value
    };

    // Password is required (firmware doesn't support public WiFi)
    const password = document.getElementById('password').value;
    if (password) {
        config.password = password;
    }
    // Note: password field may be empty if user didn't enter it,
    // validation happens in handleWriteConfig

    // Optional fields
    const ipAddr = document.getElementById('ip_addr').value;
    if (ipAddr) {
        config.ip_addr = ipAddr;
    }

    const gateway = document.getElementById('gateway').value;
    if (gateway) {
        config.gateway = gateway;
    }

    const webusbUrl = document.getElementById('webusb_url').value;
    if (webusbUrl) {
        config.webusb_url = webusbUrl;
    }

    return config;
}

/**
 * Event Handlers
 */
async function handleConnect() {
    if (!EsparrierDevice.isSupported()) {
        logError(i18n.t('logWebUsbNotSupported'));
        return;
    }

    statusIndicator.className = 'status-dot connecting';
    statusText.textContent = i18n.t('connecting');

    try {
        // Get custom VID/PID if advanced settings is enabled
        let customVid, customPid;
        if (advancedConnectionToggle.checked) {
            const vidStr = connectVidInput.value.trim();
            const pidStr = connectPidInput.value.trim();
            if (vidStr) {
                customVid = parseInt(vidStr, 16);
                if (isNaN(customVid)) {
                    logError(i18n.t('logInvalidVid'));
                    updateConnectionUI(false);
                    return;
                }
            }
            if (pidStr) {
                customPid = parseInt(pidStr, 16);
                if (isNaN(customPid)) {
                    logError(i18n.t('logInvalidPid'));
                    updateConnectionUI(false);
                    return;
                }
            }
        }

        await device.connect(customVid, customPid);
        logSuccess(i18n.t('logConnected'));

        // Get device info from USB descriptors
        const info = device.getDeviceInfo();
        if (info) {
            logInfo(`${i18n.t('logDevice')} ${info.productName || i18n.t('logUnknown')} (${info.vendorId}:${info.productId})`);
        }

        updateConnectionUI(true);

        // Set up disconnect handler
        device.onDisconnect = () => {
            logWarning(i18n.t('logDeviceDisconnected'));
            updateConnectionUI(false);
            currentState = null;
            currentConfig = null;
        };

        // Automatically read state and check firmware version
        await handleRefreshStatus();

        // Only read config if firmware is compatible
        if (currentState && checkFirmwareVersion(currentState)) {
            await handleReadConfig();
        }

    } catch (error) {
        logError(`${i18n.t('logConnectionFailed')} ${error.message}`);
        updateConnectionUI(false);
    }
}

async function handleDisconnect() {
    try {
        await device.disconnect();
        logInfo(i18n.t('logDisconnected'));
    } catch (error) {
        logError(`${i18n.t('logDisconnectFailed')} ${error.message}`);
    }
    updateConnectionUI(false);
    currentState = null;
    currentConfig = null;
}

async function handleRefreshStatus() {
    try {
        logInfo(i18n.t('logReadingStatus'));
        currentState = await device.getState();
        updateDeviceInfo(currentState);
        checkFirmwareVersion(currentState);
        logSuccess(i18n.t('logStatusUpdated'));
    } catch (error) {
        logError(`${i18n.t('logStatusFailed')} ${error.message}`);
    }
}

async function handleToggleAwake() {
    if (!currentState) {
        logError(i18n.t('logRefreshFirst'));
        return;
    }

    try {
        const newState = !currentState.keepAwake;
        logInfo(`${i18n.t('logSettingKeepAwake')} ${newState ? i18n.t('enabled') : i18n.t('disabled')}...`);
        await device.setKeepAwake(newState);
        logSuccess(i18n.t('logKeepAwakeUpdated'));
        await handleRefreshStatus();
    } catch (error) {
        logError(`${i18n.t('logKeepAwakeFailed')} ${error.message}`);
    }
}

async function handleReadConfig() {
    try {
        logInfo(i18n.t('logReadingConfig'));
        currentConfig = await device.readConfig();
        populateConfigForm(currentConfig);
        logSuccess(i18n.t('logConfigLoaded'));
    } catch (error) {
        logError(`${i18n.t('logConfigReadFailed')} ${error.message}`);
    }
}

async function handleWriteConfig() {
    const config = getConfigFromForm();

    // Basic validation
    if (!config.ssid) {
        logError(i18n.t('logSsidRequired'));
        return;
    }
    if (!config.password) {
        logError(i18n.t('logPasswordRequired'));
        return;
    }
    if (!config.server) {
        logError(i18n.t('logServerRequired'));
        return;
    }
    if (!config.screen_name) {
        logError(i18n.t('logScreenNameRequired'));
        return;
    }

    try {
        logInfo(i18n.t('logWritingConfig'));
        await device.writeConfig(config);
        logSuccess(i18n.t('logConfigWritten'));

        // Ask user to confirm commit
        if (confirm(i18n.t('confirmSaveAndReboot'))) {
            logInfo(i18n.t('logCommitting'));
            await device.commitConfig();
            logSuccess(i18n.t('logConfigSaved'));
            logWarning(i18n.t('logReconnectAfterRestart'));
        } else {
            logWarning(i18n.t('logConfigNotSaved'));
        }
    } catch (error) {
        logError(`${i18n.t('logConfigWriteFailed')} ${error.message}`);
    }
}

async function handleReboot() {
    if (!confirm(i18n.t('confirmReboot'))) {
        return;
    }

    try {
        logInfo(i18n.t('logRebooting'));
        await device.reboot();
        logSuccess(i18n.t('logRebootSent'));
    } catch (error) {
        logError(`${i18n.t('logRebootFailed')} ${error.message}`);
    }
}

function handleClearLog() {
    logOutput.innerHTML = '';
}

/**
 * Initialize
 */
function init() {
    const browserWarning = document.getElementById('browser-warning');
    const browserWarningText = document.getElementById('browser-warning-text');

    // Initialize language selector
    langSelector.value = i18n.getCurrentLanguage();
    langSelector.addEventListener('change', (e) => {
        i18n.setLanguage(e.target.value);
    });

    // Update page with current language
    i18n.updatePage();

    // Listen for language changes to update dynamic content
    i18n.onChange(() => {
        // Re-update device info if we have state
        if (currentState) {
            updateDeviceInfo(currentState);
        }
        // Update connection status text
        if (device.isConnected && device.isConnected()) {
            statusText.textContent = i18n.t('connected');
        } else {
            statusText.textContent = i18n.t('disconnected');
        }
    });

    // Check WebUSB support
    if (!EsparrierDevice.isSupported()) {
        // Show warning banner
        browserWarning.classList.remove('hidden');

        // Detect browser for more specific message
        const ua = navigator.userAgent;
        if (ua.includes('Firefox')) {
            browserWarningText.setAttribute('data-i18n', 'browserWarningFirefox');
            browserWarningText.textContent = i18n.t('browserWarningFirefox');
        } else if (ua.includes('Safari') && !ua.includes('Chrome')) {
            browserWarningText.setAttribute('data-i18n', 'browserWarningSafari');
            browserWarningText.textContent = i18n.t('browserWarningSafari');
        } else if (ua.includes('iPhone') || ua.includes('iPad') || ua.includes('Android')) {
            browserWarningText.setAttribute('data-i18n', 'browserWarningMobile');
            browserWarningText.textContent = i18n.t('browserWarningMobile');
        }

        logError(i18n.t('logWebUsbNotSupported'));
        logError(i18n.t('logUseChrome'));
        connectBtn.disabled = true;
        return;
    }

    logInfo(i18n.t('logWebUsbSupported'));

    // Event listeners
    connectBtn.addEventListener('click', handleConnect);
    disconnectBtn.addEventListener('click', handleDisconnect);
    refreshStatusBtn.addEventListener('click', handleRefreshStatus);
    toggleAwakeBtn.addEventListener('click', handleToggleAwake);
    readConfigBtn.addEventListener('click', handleReadConfig);
    writeConfigBtn.addEventListener('click', handleWriteConfig);
    rebootBtn.addEventListener('click', handleReboot);
    clearLogBtn.addEventListener('click', handleClearLog);

    // Brightness slider
    brightnessInput.addEventListener('input', () => {
        brightnessValue.textContent = brightnessInput.value;
    });

    // Advanced connection settings toggle
    advancedConnectionToggle.addEventListener('change', () => {
        if (advancedConnectionToggle.checked) {
            advancedConnectionSettings.classList.remove('hidden');
        } else {
            advancedConnectionSettings.classList.add('hidden');
        }
    });

    // Password visibility toggle
    const togglePasswordBtn = document.getElementById('toggle-password-btn');
    const passwordInput = document.getElementById('password');
    togglePasswordBtn.addEventListener('click', () => {
        const isPassword = passwordInput.type === 'password';
        passwordInput.type = isPassword ? 'text' : 'password';
        togglePasswordBtn.classList.toggle('showing', isPassword);
        togglePasswordBtn.title = i18n.t(isPassword ? 'hidePassword' : 'showPassword');
        togglePasswordBtn.setAttribute('data-i18n-title', isPassword ? 'hidePassword' : 'showPassword');
    });

    // Required field validation - listen for input changes
    const requiredFields = ['ssid', 'password', 'server', 'screen_name'];
    requiredFields.forEach(fieldId => {
        document.getElementById(fieldId).addEventListener('input', validateRequiredFields);
    });

    // Initially disable write button until fields are validated
    writeConfigBtn.disabled = true;

    // Prevent form submission
    configForm.addEventListener('submit', (e) => {
        e.preventDefault();
    });

    // Auto-reconnect to previously paired device
    tryAutoReconnect();
}

/**
 * Try to auto-reconnect to a previously paired device
 */
async function tryAutoReconnect() {
    try {
        const pairedDevices = await EsparrierDevice.getPairedDevices();

        if (pairedDevices.length === 0) {
            return;
        }

        logInfo(i18n.t('logFoundDevices', { count: pairedDevices.length }));

        // Try to connect to the first available paired device
        const usbDevice = pairedDevices[0];
        logInfo(i18n.t('logAutoReconnecting'));

        statusIndicator.className = 'status-dot connecting';
        statusText.textContent = i18n.t('connecting');

        await device.connectToDevice(usbDevice);
        logSuccess(i18n.t('logConnected'));

        // Get device info from USB descriptors
        const info = device.getDeviceInfo();
        if (info) {
            logInfo(`${i18n.t('logDevice')} ${info.productName || i18n.t('logUnknown')} (${info.vendorId}:${info.productId})`);
        }

        updateConnectionUI(true);

        // Set up disconnect handler
        device.onDisconnect = () => {
            logWarning(i18n.t('logDeviceDisconnected'));
            updateConnectionUI(false);
            currentState = null;
            currentConfig = null;
        };

        // Automatically read state and check firmware version
        await handleRefreshStatus();

        // Only read config if firmware is compatible
        if (currentState && checkFirmwareVersion(currentState)) {
            await handleReadConfig();
        }

    } catch (error) {
        // Auto-reconnect failed silently - user can manually connect
        logWarning(`${i18n.t('logAutoReconnectFailed')} ${error.message}`);
        updateConnectionUI(false);
    }
}

// Start the app
document.addEventListener('DOMContentLoaded', init);
