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

// Device instance
const device = new EsparrierDevice();

// Current state
let currentState = null;
let currentConfig = null;

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
        statusText.textContent = 'Connected';
        connectBtn.disabled = true;
        disconnectBtn.disabled = false;
        deviceInfoSection.classList.remove('hidden');
        configSection.classList.remove('hidden');
    } else {
        statusIndicator.className = 'status-dot disconnected';
        statusText.textContent = 'Disconnected';
        connectBtn.disabled = false;
        disconnectBtn.disabled = true;
        deviceInfoSection.classList.add('hidden');
        configSection.classList.add('hidden');
    }
}

function updateDeviceInfo(state) {
    firmwareVersion.textContent = state.version;
    modelId.textContent = state.modelName;
    featureFlags.textContent = state.features.length > 0 ? state.features.join(', ') : 'None';
    ipAddress.textContent = state.ipAddressStr;
    serverConnected.textContent = state.serverConnected ? 'Yes' : 'No';
    serverConnected.style.color = state.serverConnected ? 'var(--success-color)' : 'var(--danger-color)';
    deviceActive.textContent = state.active ? 'Yes' : 'No';
    deviceActive.style.color = state.active ? 'var(--success-color)' : '';
    keepAwake.textContent = state.keepAwake ? 'Enabled' : 'Disabled';
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
        logError('WebUSB is not supported in this browser. Please use Chrome, Edge, or Opera.');
        return;
    }

    statusIndicator.className = 'status-dot connecting';
    statusText.textContent = 'Connecting...';

    try {
        await device.connect();
        logSuccess('Connected to device');

        // Get device info from USB descriptors
        const info = device.getDeviceInfo();
        if (info) {
            logInfo(`Device: ${info.productName || 'Unknown'} (${info.vendorId}:${info.productId})`);
        }

        updateConnectionUI(true);

        // Set up disconnect handler
        device.onDisconnect = () => {
            logWarning('Device disconnected');
            updateConnectionUI(false);
            currentState = null;
            currentConfig = null;
        };

        // Automatically read state and config
        await handleRefreshStatus();
        await handleReadConfig();

    } catch (error) {
        logError(`Connection failed: ${error.message}`);
        updateConnectionUI(false);
    }
}

async function handleDisconnect() {
    try {
        await device.disconnect();
        logInfo('Disconnected from device');
    } catch (error) {
        logError(`Disconnect failed: ${error.message}`);
    }
    updateConnectionUI(false);
    currentState = null;
    currentConfig = null;
}

async function handleRefreshStatus() {
    try {
        logInfo('Reading device status...');
        currentState = await device.getState();
        updateDeviceInfo(currentState);
        logSuccess('Status updated');
    } catch (error) {
        logError(`Failed to read status: ${error.message}`);
    }
}

async function handleToggleAwake() {
    if (!currentState) {
        logError('Please refresh status first');
        return;
    }

    try {
        const newState = !currentState.keepAwake;
        logInfo(`Setting keep awake to ${newState ? 'enabled' : 'disabled'}...`);
        await device.setKeepAwake(newState);
        logSuccess('Keep awake updated');
        await handleRefreshStatus();
    } catch (error) {
        logError(`Failed to toggle keep awake: ${error.message}`);
    }
}

async function handleReadConfig() {
    try {
        logInfo('Reading configuration...');
        currentConfig = await device.readConfig();
        populateConfigForm(currentConfig);
        logSuccess('Configuration loaded');
    } catch (error) {
        logError(`Failed to read configuration: ${error.message}`);
    }
}

async function handleWriteConfig() {
    const config = getConfigFromForm();

    // Basic validation
    if (!config.ssid) {
        logError('WiFi SSID is required');
        return;
    }
    if (!config.password) {
        logError('WiFi password is required (public WiFi not supported)');
        return;
    }
    if (!config.server) {
        logError('Server address is required');
        return;
    }
    if (!config.screen_name) {
        logError('Screen name is required');
        return;
    }

    try {
        logInfo('Writing configuration...');
        await device.writeConfig(config);
        logSuccess('Configuration written (not yet saved to flash)');

        // Ask user to confirm commit
        if (confirm('Configuration validated. Save to flash and reboot device?')) {
            logInfo('Committing configuration to flash...');
            await device.commitConfig();
            logSuccess('Configuration saved. Device is rebooting...');
            logWarning('Please reconnect after the device restarts.');
        } else {
            logWarning('Configuration NOT saved to flash. It will be lost on reboot.');
        }
    } catch (error) {
        logError(`Failed to write configuration: ${error.message}`);
    }
}

async function handleReboot() {
    if (!confirm('Are you sure you want to reboot the device?')) {
        return;
    }

    try {
        logInfo('Rebooting device...');
        await device.reboot();
        logSuccess('Reboot command sent. Device is restarting...');
    } catch (error) {
        logError(`Failed to reboot: ${error.message}`);
    }
}

function handleClearLog() {
    logOutput.innerHTML = '';
}

/**
 * Initialize
 */
function init() {
    // Check WebUSB support
    if (!EsparrierDevice.isSupported()) {
        logError('WebUSB is not supported in this browser.');
        logError('Please use Chrome, Edge, or Opera on desktop.');
        connectBtn.disabled = true;
        return;
    }

    logInfo('WebUSB is supported. Click "Connect Device" to begin.');

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

    // Prevent form submission
    configForm.addEventListener('submit', (e) => {
        e.preventDefault();
    });

    // Check for already paired devices
    navigator.usb.getDevices().then(devices => {
        if (devices.length > 0) {
            logInfo(`Found ${devices.length} previously paired device(s)`);
        }
    });
}

// Start the app
document.addEventListener('DOMContentLoaded', init);
