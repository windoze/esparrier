/**
 * Esparrier i18n (Internationalization) Module
 */

const translations = {
    en: {
        // Page title and header
        pageTitle: 'Esparrier WebUSB Configuration',
        title: 'Esparrier Configuration',
        subtitle: 'WebUSB-based device configuration tool',

        // Browser warning
        browserNotSupported: '⚠️ Browser Not Supported',
        browserWarningDefault: 'Your browser does not support WebUSB. Please use a Chromium-based browser (Chrome, Edge, Opera, Brave) to use this tool.',
        browserWarningFirefox: 'Firefox does not support WebUSB. Please use a Chromium-based browser (Chrome, Edge, Opera, Brave) to use this tool.',
        browserWarningSafari: 'Safari does not support WebUSB. Please use a Chromium-based browser (Chrome, Edge, Opera, Brave) to use this tool.',
        browserWarningMobile: 'WebUSB is not supported on mobile devices. Please use a Chromium-based browser (Chrome, Edge, Opera, Brave) on a desktop computer.',

        // Connection section
        deviceConnection: 'Device Connection',
        disconnected: 'Disconnected',
        connected: 'Connected',
        connecting: 'Connecting...',
        connectDevice: 'Connect Device',
        disconnect: 'Disconnect',

        // Device info section
        deviceInformation: 'Device Information',
        firmwareVersion: 'Firmware Version',
        modelId: 'Model ID',
        features: 'Features',
        ipAddress: 'IP Address',
        serverConnected: 'Server Connected',
        active: 'Active',
        keepAwake: 'Keep Awake',
        refreshStatus: 'Refresh Status',
        toggleKeepAwake: 'Toggle Keep Awake',
        keepAwakeOn: 'Keep Awake: On',
        keepAwakeOff: 'Keep Awake: Off',
        none: 'None',
        yes: 'Yes',
        no: 'No',
        enabled: 'Enabled',
        disabled: 'Disabled',

        // Configuration section
        configuration: 'Configuration',
        networkSettings: 'Network Settings',
        wifiSsid: 'WiFi SSID',
        wifiPassword: 'WiFi Password',
        wifiPasswordHint: 'Required - public WiFi not supported',
        ssidPlaceholder: 'Enter WiFi network name',
        passwordPlaceholder: 'Enter WiFi password',
        staticIp: 'Static IP (CIDR, optional)',
        staticIpPlaceholder: 'e.g., 192.168.1.100/24',
        gateway: 'Gateway (optional)',
        gatewayPlaceholder: 'e.g., 192.168.1.1',

        // Server settings
        barrierServer: 'Barrier/Deskflow Server',
        serverAddress: 'Server Address',
        serverAddressPlaceholder: 'e.g., 192.168.1.50:24800',
        screenName: 'Screen Name',
        screenNamePlaceholder: 'Enter screen name',

        // Screen settings
        screenSettings: 'Screen Settings',
        screenWidth: 'Screen Width',
        screenHeight: 'Screen Height',
        flipMouseWheel: 'Flip Mouse Wheel Direction',

        // Performance settings
        performanceSettings: 'Performance Settings',
        pollingRate: 'Polling Rate (Hz)',
        jiggleInterval: 'Jiggle Interval (sec)',
        jiggleIntervalHint: '0 to disable',
        indicatorBrightness: 'Indicator Brightness',

        // USB HID settings
        usbHidIdentity: 'USB HID Identity',
        vendorId: 'Vendor ID (hex)',
        productId: 'Product ID (hex)',
        manufacturer: 'Manufacturer',
        productName: 'Product Name',
        serialNumber: 'Serial Number',
        webusbUrl: 'WebUSB Landing URL (optional)',
        webusbUrlHint: 'Leave empty to disable browser popup on device connect',

        // Buttons
        readConfig: 'Read Config',
        writeConfig: 'Write Config',
        rebootDevice: 'Reboot Device',

        // Log section
        log: 'Log',
        clearLog: 'Clear Log',

        // Footer
        footerText: 'Esparrier WebUSB Configuration Tool',
        githubRepo: 'GitHub Repository',

        // Log messages
        logWebUsbSupported: 'WebUSB is supported. Click "Connect Device" to begin.',
        logWebUsbNotSupported: 'WebUSB is not supported in this browser.',
        logUseChrome: 'Please use Chrome, Edge, or Opera on desktop.',
        logConnected: 'Connected to device',
        logDevice: 'Device:',
        logUnknown: 'Unknown',
        logConnectionFailed: 'Connection failed:',
        logDisconnected: 'Disconnected from device',
        logDisconnectFailed: 'Disconnect failed:',
        logDeviceDisconnected: 'Device disconnected',
        logReadingStatus: 'Reading device status...',
        logStatusUpdated: 'Status updated',
        logStatusFailed: 'Failed to read status:',
        logRefreshFirst: 'Please refresh status first',
        logSettingKeepAwake: 'Setting keep awake to',
        logKeepAwakeUpdated: 'Keep awake updated',
        logKeepAwakeFailed: 'Failed to toggle keep awake:',
        logReadingConfig: 'Reading configuration...',
        logConfigLoaded: 'Configuration loaded',
        logConfigReadFailed: 'Failed to read configuration:',
        logSsidRequired: 'WiFi SSID is required',
        logPasswordRequired: 'WiFi password is required (public WiFi not supported)',
        logServerRequired: 'Server address is required',
        logScreenNameRequired: 'Screen name is required',
        logWritingConfig: 'Writing configuration...',
        logConfigWritten: 'Configuration written (not yet saved to flash)',
        logCommitting: 'Committing configuration to flash...',
        logConfigSaved: 'Configuration saved. Device is rebooting...',
        logReconnectAfterRestart: 'Please reconnect after the device restarts.',
        logConfigNotSaved: 'Configuration NOT saved to flash. It will be lost on reboot.',
        logConfigWriteFailed: 'Failed to write configuration:',
        logRebooting: 'Rebooting device...',
        logRebootSent: 'Reboot command sent. Device is restarting...',
        logRebootFailed: 'Failed to reboot:',
        logFoundDevices: 'Found {count} previously paired device(s)',

        // Confirm dialogs
        confirmSaveAndReboot: 'Configuration validated. Save to flash and reboot device?',
        confirmReboot: 'Are you sure you want to reboot the device?',

        // Firmware version warnings
        firmwareOutdated: '⚠️ Firmware Outdated',
        firmwareWarningText: 'Your firmware version is too old to use this configuration tool. Please upgrade to version 0.6.0 or later.',
        downloadLatestFirmware: 'Download Latest Firmware',
        logFirmwareTooOld: 'Firmware version {version} is too old. Please upgrade to 0.6.0 or later.'
    },
    zh: {
        // Page title and header
        pageTitle: 'Esparrier WebUSB 配置工具',
        title: 'Esparrier 配置',
        subtitle: '基于 WebUSB 的设备配置工具',

        // Browser warning
        browserNotSupported: '⚠️ 浏览器不支持',
        browserWarningDefault: '您的浏览器不支持 WebUSB。请使用基于 Chromium 的浏览器（Chrome、Edge、Opera、Brave）。',
        browserWarningFirefox: 'Firefox 不支持 WebUSB。请使用基于 Chromium 的浏览器（Chrome、Edge、Opera、Brave）。',
        browserWarningSafari: 'Safari 不支持 WebUSB。请使用基于 Chromium 的浏览器（Chrome、Edge、Opera、Brave）。',
        browserWarningMobile: '移动设备不支持 WebUSB。请在桌面电脑上使用基于 Chromium 的浏览器（Chrome、Edge、Opera、Brave）。',

        // Connection section
        deviceConnection: '设备连接',
        disconnected: '未连接',
        connected: '已连接',
        connecting: '连接中...',
        connectDevice: '连接设备',
        disconnect: '断开连接',

        // Device info section
        deviceInformation: '设备信息',
        firmwareVersion: '固件版本',
        modelId: '型号 ID',
        features: '功能特性',
        ipAddress: 'IP 地址',
        serverConnected: '服务器连接',
        active: '活动状态',
        keepAwake: '保持唤醒',
        refreshStatus: '刷新状态',
        toggleKeepAwake: '切换保持唤醒',
        keepAwakeOn: '保持唤醒：开',
        keepAwakeOff: '保持唤醒：关',
        none: '无',
        yes: '是',
        no: '否',
        enabled: '已启用',
        disabled: '已禁用',

        // Configuration section
        configuration: '配置',
        networkSettings: '网络设置',
        wifiSsid: 'WiFi SSID',
        wifiPassword: 'WiFi 密码',
        wifiPasswordHint: '必填 - 不支持开放网络',
        ssidPlaceholder: '输入 WiFi 网络名称',
        passwordPlaceholder: '输入 WiFi 密码',
        staticIp: '静态 IP（CIDR 格式，可选）',
        staticIpPlaceholder: '例如：192.168.1.100/24',
        gateway: '网关（可选）',
        gatewayPlaceholder: '例如：192.168.1.1',

        // Server settings
        barrierServer: 'Barrier/Deskflow 服务器',
        serverAddress: '服务器地址',
        serverAddressPlaceholder: '例如：192.168.1.50:24800',
        screenName: '屏幕名称',
        screenNamePlaceholder: '输入屏幕名称',

        // Screen settings
        screenSettings: '屏幕设置',
        screenWidth: '屏幕宽度',
        screenHeight: '屏幕高度',
        flipMouseWheel: '反转鼠标滚轮方向',

        // Performance settings
        performanceSettings: '性能设置',
        pollingRate: '轮询频率 (Hz)',
        jiggleInterval: '抖动间隔（秒）',
        jiggleIntervalHint: '设为 0 禁用',
        indicatorBrightness: '指示灯亮度',

        // USB HID settings
        usbHidIdentity: 'USB HID 标识',
        vendorId: '厂商 ID（十六进制）',
        productId: '产品 ID（十六进制）',
        manufacturer: '制造商',
        productName: '产品名称',
        serialNumber: '序列号',
        webusbUrl: 'WebUSB 落地页 URL（可选）',
        webusbUrlHint: '留空以禁用设备连接时的浏览器弹窗',

        // Buttons
        readConfig: '读取配置',
        writeConfig: '写入配置',
        rebootDevice: '重启设备',

        // Log section
        log: '日志',
        clearLog: '清除日志',

        // Footer
        footerText: 'Esparrier WebUSB 配置工具',
        githubRepo: 'GitHub 仓库',

        // Log messages
        logWebUsbSupported: 'WebUSB 可用。点击"连接设备"开始。',
        logWebUsbNotSupported: '此浏览器不支持 WebUSB。',
        logUseChrome: '请在桌面电脑上使用 Chrome、Edge 或 Opera。',
        logConnected: '已连接到设备',
        logDevice: '设备：',
        logUnknown: '未知',
        logConnectionFailed: '连接失败：',
        logDisconnected: '已断开与设备的连接',
        logDisconnectFailed: '断开连接失败：',
        logDeviceDisconnected: '设备已断开',
        logReadingStatus: '正在读取设备状态...',
        logStatusUpdated: '状态已更新',
        logStatusFailed: '读取状态失败：',
        logRefreshFirst: '请先刷新状态',
        logSettingKeepAwake: '正在设置保持唤醒为',
        logKeepAwakeUpdated: '保持唤醒已更新',
        logKeepAwakeFailed: '切换保持唤醒失败：',
        logReadingConfig: '正在读取配置...',
        logConfigLoaded: '配置已加载',
        logConfigReadFailed: '读取配置失败：',
        logSsidRequired: 'WiFi SSID 为必填项',
        logPasswordRequired: 'WiFi 密码为必填项（不支持开放网络）',
        logServerRequired: '服务器地址为必填项',
        logScreenNameRequired: '屏幕名称为必填项',
        logWritingConfig: '正在写入配置...',
        logConfigWritten: '配置已写入（尚未保存到闪存）',
        logCommitting: '正在将配置保存到闪存...',
        logConfigSaved: '配置已保存。设备正在重启...',
        logReconnectAfterRestart: '请在设备重启后重新连接。',
        logConfigNotSaved: '配置未保存到闪存，重启后将丢失。',
        logConfigWriteFailed: '写入配置失败：',
        logRebooting: '正在重启设备...',
        logRebootSent: '重启命令已发送。设备正在重启...',
        logRebootFailed: '重启失败：',
        logFoundDevices: '发现 {count} 个已配对的设备',

        // Confirm dialogs
        confirmSaveAndReboot: '配置验证通过。是否保存到闪存并重启设备？',
        confirmReboot: '确定要重启设备吗？',

        // Firmware version warnings
        firmwareOutdated: '⚠️ 固件版本过旧',
        firmwareWarningText: '您的固件版本过旧，无法使用此配置工具。请升级到 0.6.0 或更高版本。',
        downloadLatestFirmware: '下载最新固件',
        logFirmwareTooOld: '固件版本 {version} 过旧。请升级到 0.6.0 或更高版本。'
    }
};

class I18n {
    constructor() {
        this.currentLang = this.detectLanguage();
        this.listeners = [];
    }

    detectLanguage() {
        // Check URL parameter
        const urlParams = new URLSearchParams(window.location.search);
        const langParam = urlParams.get('lang');
        if (langParam && translations[langParam]) {
            return langParam;
        }

        // Check localStorage
        const savedLang = localStorage.getItem('esparrier-lang');
        if (savedLang && translations[savedLang]) {
            return savedLang;
        }

        // Check browser language
        const browserLang = navigator.language.split('-')[0];
        if (translations[browserLang]) {
            return browserLang;
        }

        // Default to English
        return 'en';
    }

    setLanguage(lang) {
        if (translations[lang]) {
            this.currentLang = lang;
            localStorage.setItem('esparrier-lang', lang);
            this.updatePage();
            this.notifyListeners();
        }
    }

    t(key, params = {}) {
        let text = translations[this.currentLang][key] || translations['en'][key] || key;

        // Replace parameters like {count}
        for (const [param, value] of Object.entries(params)) {
            text = text.replace(`{${param}}`, value);
        }

        return text;
    }

    updatePage() {
        // Update page title
        document.title = this.t('pageTitle');

        // Update all elements with data-i18n attribute
        document.querySelectorAll('[data-i18n]').forEach(el => {
            const key = el.getAttribute('data-i18n');
            el.textContent = this.t(key);
        });

        // Update all elements with data-i18n-placeholder attribute
        document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
            const key = el.getAttribute('data-i18n-placeholder');
            el.placeholder = this.t(key);
        });

        // Update all elements with data-i18n-title attribute
        document.querySelectorAll('[data-i18n-title]').forEach(el => {
            const key = el.getAttribute('data-i18n-title');
            el.title = this.t(key);
        });

        // Update html lang attribute
        document.documentElement.lang = this.currentLang;
    }

    onChange(callback) {
        this.listeners.push(callback);
    }

    notifyListeners() {
        this.listeners.forEach(callback => callback(this.currentLang));
    }

    getAvailableLanguages() {
        return Object.keys(translations);
    }

    getCurrentLanguage() {
        return this.currentLang;
    }
}

// Create global instance
const i18n = new I18n();
