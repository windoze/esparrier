/**
 * Esparrier WebUSB Communication Library
 *
 * This library provides a JavaScript interface to communicate with
 * Esparrier devices via WebUSB using the vendor-specific bulk interface.
 *
 * Protocol Commands:
 * - 's' - Get State (returns device status)
 * - 'r' - Read Config (reads configuration from flash)
 * - 'w' <blocks> - Write Config (receives config in 64-byte blocks)
 * - 'c' - Commit Config (writes config to flash and reboots)
 * - 'k' <bool> - Keep Awake (prevent device sleep)
 * - 'b' - Reboot (trigger software reset)
 */

const ESPARRIER_VID = 0x0d0a;
const ESPARRIER_PID = 0xc0de;

// Vendor interface class
const VENDOR_CLASS = 0xFF;
const VENDOR_SUBCLASS = 0x0D;
const VENDOR_PROTOCOL = 0x0A;

// Commands
const CMD_GET_STATE = 's'.charCodeAt(0);
const CMD_READ_CONFIG = 'r'.charCodeAt(0);
const CMD_WRITE_CONFIG = 'w'.charCodeAt(0);
const CMD_COMMIT_CONFIG = 'c'.charCodeAt(0);
const CMD_KEEP_AWAKE = 'k'.charCodeAt(0);
const CMD_REBOOT = 'b'.charCodeAt(0);

// Responses
const RESP_STATE = 's'.charCodeAt(0);
const RESP_CONFIG = 'r'.charCodeAt(0);
const RESP_OK = 'o'.charCodeAt(0);
const RESP_ERROR = 'e'.charCodeAt(0);

// Error codes
const ERR_ENDPOINT = 'e'.charCodeAt(0);
const ERR_TIMEOUT = 't'.charCodeAt(0);
const ERR_INVALID_CONFIG = 'i'.charCodeAt(0);
const ERR_UNKNOWN_COMMAND = 'u'.charCodeAt(0);

// Feature flags
const FEATURE_LED = 0b00000001;
const FEATURE_SMARTLED = 0b00000010;
const FEATURE_GRAPHICS = 0b00000100;
const FEATURE_CLIPBOARD = 0b10000000;

// Model IDs
const MODEL_NAMES = {
    0: 'Generic',
    1: 'M5Atom S3 Lite',
    2: 'M5Atom S3',
    3: 'M5Atom S3R',
    4: 'DevKitC-1.0',
    5: 'DevKitC-1.1',
    6: 'XIAO ESP32S3',
    7: 'ESP32-S3-ETH',
    255: 'Generic ESP32-S3'
};

class EsparrierDevice {
    constructor() {
        this.device = null;
        this.interfaceNumber = null;
        this.endpointIn = null;
        this.endpointOut = null;
        this.onDisconnect = null;
    }

    /**
     * Check if WebUSB is supported in this browser
     */
    static isSupported() {
        return 'usb' in navigator;
    }

    /**
     * Request and connect to an Esparrier device
     */
    async connect() {
        if (!EsparrierDevice.isSupported()) {
            throw new Error('WebUSB is not supported in this browser');
        }

        // Request device with specific VID/PID
        this.device = await navigator.usb.requestDevice({
            filters: [
                { vendorId: ESPARRIER_VID, productId: ESPARRIER_PID },
                // Also allow any device with our vendor interface class
                { classCode: VENDOR_CLASS, subclassCode: VENDOR_SUBCLASS, protocolCode: VENDOR_PROTOCOL }
            ]
        });

        await this.device.open();

        // Find the vendor-specific interface
        for (const config of this.device.configurations) {
            for (const iface of config.interfaces) {
                for (const alt of iface.alternates) {
                    if (alt.interfaceClass === VENDOR_CLASS &&
                        alt.interfaceSubclass === VENDOR_SUBCLASS &&
                        alt.interfaceProtocol === VENDOR_PROTOCOL) {
                        this.interfaceNumber = iface.interfaceNumber;

                        // Find endpoints
                        for (const ep of alt.endpoints) {
                            if (ep.direction === 'in') {
                                this.endpointIn = ep.endpointNumber;
                            } else if (ep.direction === 'out') {
                                this.endpointOut = ep.endpointNumber;
                            }
                        }
                        break;
                    }
                }
                if (this.interfaceNumber !== null) break;
            }
            if (this.interfaceNumber !== null) break;
        }

        if (this.interfaceNumber === null) {
            await this.device.close();
            throw new Error('Vendor interface not found on device');
        }

        // Select configuration and claim interface
        if (this.device.configuration === null) {
            await this.device.selectConfiguration(1);
        }

        await this.device.claimInterface(this.interfaceNumber);

        // Set up disconnect handler
        navigator.usb.addEventListener('disconnect', (event) => {
            if (event.device === this.device) {
                this.device = null;
                this.interfaceNumber = null;
                this.endpointIn = null;
                this.endpointOut = null;
                if (this.onDisconnect) {
                    this.onDisconnect();
                }
            }
        });

        return true;
    }

    /**
     * Disconnect from the device
     */
    async disconnect() {
        if (this.device) {
            try {
                await this.device.releaseInterface(this.interfaceNumber);
                await this.device.close();
            } catch (e) {
                // Ignore errors during disconnect
            }
            this.device = null;
            this.interfaceNumber = null;
            this.endpointIn = null;
            this.endpointOut = null;
        }
    }

    /**
     * Check if connected to a device
     */
    isConnected() {
        return this.device !== null && this.device.opened;
    }

    /**
     * Send a command and receive response
     */
    async sendCommand(data) {
        if (!this.isConnected()) {
            throw new Error('Not connected to device');
        }

        // Send command
        await this.device.transferOut(this.endpointOut, new Uint8Array(data));

        // Receive response
        const result = await this.device.transferIn(this.endpointIn, 64);
        return new Uint8Array(result.data.buffer);
    }

    /**
     * Send data without expecting a response
     */
    async sendData(data) {
        if (!this.isConnected()) {
            throw new Error('Not connected to device');
        }
        await this.device.transferOut(this.endpointOut, new Uint8Array(data));
    }

    /**
     * Receive data from the device
     */
    async receiveData() {
        if (!this.isConnected()) {
            throw new Error('Not connected to device');
        }
        const result = await this.device.transferIn(this.endpointIn, 64);
        return new Uint8Array(result.data.buffer);
    }

    /**
     * Parse error response
     */
    parseError(errorCode) {
        switch (errorCode) {
            case ERR_ENDPOINT: return 'Endpoint error';
            case ERR_TIMEOUT: return 'Timeout';
            case ERR_INVALID_CONFIG: return 'Invalid configuration';
            case ERR_UNKNOWN_COMMAND: return 'Unknown command';
            default: return `Unknown error (${String.fromCharCode(errorCode)})`;
        }
    }

    /**
     * Get device running state
     */
    async getState() {
        const response = await this.sendCommand([CMD_GET_STATE]);

        if (response[0] !== RESP_STATE) {
            if (response[0] === RESP_ERROR) {
                throw new Error(this.parseError(response[1]));
            }
            throw new Error('Unexpected response');
        }

        // Parse running state (13 bytes after response code)
        const state = {
            versionMajor: response[1],
            versionMinor: response[2],
            versionPatch: response[3],
            featureFlags: response[4],
            ipAddress: null,
            serverConnected: response[10] !== 0,
            active: response[11] !== 0,
            keepAwake: response[12] !== 0,
            modelId: response[13]
        };

        // Parse IP address if present
        if (response[5] !== 0 || response[6] !== 0 || response[7] !== 0 || response[8] !== 0) {
            state.ipAddress = {
                octets: [response[5], response[6], response[7], response[8]],
                prefixLen: response[9]
            };
        }

        // Add derived fields
        state.version = `${state.versionMajor}.${state.versionMinor}.${state.versionPatch}`;
        state.modelName = MODEL_NAMES[state.modelId] || `Unknown (${state.modelId})`;
        state.features = [];
        if (state.featureFlags & FEATURE_LED) state.features.push('LED');
        if (state.featureFlags & FEATURE_SMARTLED) state.features.push('SmartLED');
        if (state.featureFlags & FEATURE_GRAPHICS) state.features.push('Graphics');
        if (state.featureFlags & FEATURE_CLIPBOARD) state.features.push('Clipboard');

        if (state.ipAddress) {
            state.ipAddressStr = `${state.ipAddress.octets.join('.')}/${state.ipAddress.prefixLen}`;
        } else {
            state.ipAddressStr = 'Not assigned';
        }

        return state;
    }

    /**
     * Read configuration from device
     */
    async readConfig() {
        const response = await this.sendCommand([CMD_READ_CONFIG]);

        if (response[0] !== RESP_CONFIG) {
            if (response[0] === RESP_ERROR) {
                throw new Error(this.parseError(response[1]));
            }
            throw new Error('Unexpected response');
        }

        const blockCount = response[1];
        const configData = new Uint8Array(blockCount * 64);

        // Receive all blocks
        for (let i = 0; i < blockCount; i++) {
            const block = await this.receiveData();
            configData.set(block, i * 64);
        }

        // Find end of JSON (null terminator or invalid UTF-8)
        let jsonEnd = configData.length;
        for (let i = 0; i < configData.length; i++) {
            if (configData[i] === 0 || configData[i] > 0xF4) {
                jsonEnd = i;
                break;
            }
        }

        // Parse JSON
        const jsonStr = new TextDecoder().decode(configData.subarray(0, jsonEnd));
        return JSON.parse(jsonStr);
    }

    /**
     * Write configuration to device (does not commit)
     */
    async writeConfig(config) {
        // Serialize config to JSON
        const jsonStr = JSON.stringify(config);
        const jsonBytes = new TextEncoder().encode(jsonStr);

        // Calculate block count
        const blockCount = Math.ceil(jsonBytes.length / 64);

        if (blockCount > 64) { // Max 4096 bytes
            throw new Error('Configuration too large');
        }

        // Send write command (device will immediately start receiving blocks)
        await this.sendData([CMD_WRITE_CONFIG, blockCount]);

        // Send config blocks
        for (let i = 0; i < blockCount; i++) {
            const block = new Uint8Array(64);
            const start = i * 64;
            const end = Math.min(start + 64, jsonBytes.length);
            block.set(jsonBytes.subarray(start, end));
            await this.sendData(block);
        }

        // Receive validation response after all blocks are sent
        const validationResponse = await this.receiveData();

        if (validationResponse[0] !== RESP_OK) {
            if (validationResponse[0] === RESP_ERROR) {
                throw new Error(this.parseError(validationResponse[1]));
            }
            throw new Error('Configuration validation failed');
        }

        return true;
    }

    /**
     * Commit written configuration (writes to flash and reboots)
     */
    async commitConfig() {
        const response = await this.sendCommand([CMD_COMMIT_CONFIG]);

        if (response[0] !== RESP_OK) {
            if (response[0] === RESP_ERROR) {
                throw new Error(this.parseError(response[1]));
            }
            throw new Error('Commit failed');
        }

        // Device will reboot, connection will be lost
        return true;
    }

    /**
     * Set keep awake mode
     */
    async setKeepAwake(enabled) {
        const response = await this.sendCommand([CMD_KEEP_AWAKE, enabled ? 1 : 0]);

        if (response[0] !== RESP_OK) {
            if (response[0] === RESP_ERROR) {
                throw new Error(this.parseError(response[1]));
            }
            throw new Error('Failed to set keep awake');
        }

        return true;
    }

    /**
     * Reboot the device
     */
    async reboot() {
        const response = await this.sendCommand([CMD_REBOOT]);

        if (response[0] !== RESP_OK) {
            if (response[0] === RESP_ERROR) {
                throw new Error(this.parseError(response[1]));
            }
            throw new Error('Reboot command failed');
        }

        // Device will reboot, connection will be lost
        return true;
    }

    /**
     * Get device info from USB descriptors
     */
    getDeviceInfo() {
        if (!this.device) return null;

        return {
            vendorId: this.device.vendorId.toString(16).padStart(4, '0'),
            productId: this.device.productId.toString(16).padStart(4, '0'),
            manufacturerName: this.device.manufacturerName,
            productName: this.device.productName,
            serialNumber: this.device.serialNumber
        };
    }
}

// Export for use in other scripts
window.EsparrierDevice = EsparrierDevice;
