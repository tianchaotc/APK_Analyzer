/// Database of known Android permissions with their protection levels and descriptions
pub struct PermissionDBEntry {
    pub protection_level: String,
    pub description: String,
    pub risk_level: String,
    pub recommended_usage: String,
    pub category: String,
}

impl Default for PermissionDBEntry {
    fn default() -> Self {
        Self {
            protection_level: "unknown".to_string(),
            description: "Custom or unknown permission".to_string(),
            risk_level: "unknown".to_string(),
            recommended_usage: "Review the necessity of this permission".to_string(),
            category: "Other".to_string(),
        }
    }
}

/// Look up a permission in the database
pub fn lookup(name: &str) -> PermissionDBEntry {
    match name {
        // === Normal permissions ===
        "android.permission.INTERNET" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows applications to open network sockets".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Required for any network communication".to_string(),
            category: "Network".to_string(),
        },
        "android.permission.ACCESS_NETWORK_STATE" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows access to information about networks".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Check network connectivity status".to_string(),
            category: "Network".to_string(),
        },
        "android.permission.ACCESS_WIFI_STATE" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows access to information about Wi-Fi networks".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Read Wi-Fi connection state".to_string(),
            category: "Network".to_string(),
        },
        "android.permission.CHANGE_WIFI_STATE" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows applications to change Wi-Fi connectivity state".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Enable/disable Wi-Fi".to_string(),
            category: "Network".to_string(),
        },
        "android.permission.VIBRATE" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows access to the vibrator".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Haptic feedback for notifications".to_string(),
            category: "Hardware".to_string(),
        },
        "android.permission.WAKE_LOCK" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows using PowerManager WakeLocks to keep processor from sleeping".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Keep CPU running during background tasks".to_string(),
            category: "Power".to_string(),
        },
        "android.permission.RECEIVE_BOOT_COMPLETED" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows an application to receive ACTION_BOOT_COMPLETED broadcast".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Start service on device boot".to_string(),
            category: "System".to_string(),
        },
        "android.permission.FOREGROUND_SERVICE" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows a regular application to use foreground services".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Required for foreground services on Android 9+".to_string(),
            category: "System".to_string(),
        },
        "android.permission.POST_NOTIFICATIONS" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows an app to post notifications (Android 13+)".to_string(),
            risk_level: "medium".to_string(),
            recommended_usage: "Display notifications to the user".to_string(),
            category: "Notification".to_string(),
        },
        "android.permission.ACCESS_FINE_LOCATION" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows access to precise location (GPS)".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "Maps, navigation, location-based services".to_string(),
            category: "Location".to_string(),
        },
        "android.permission.ACCESS_COARSE_LOCATION" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows access to approximate location (cell/Wi-Fi)".to_string(),
            risk_level: "medium".to_string(),
            recommended_usage: "Approximate location for area-based features".to_string(),
            category: "Location".to_string(),
        },
        "android.permission.ACCESS_BACKGROUND_LOCATION" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows access to location in the background".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "Background location tracking (requires strong justification)".to_string(),
            category: "Location".to_string(),
        },
        "android.permission.CAMERA" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Required to access the camera device".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "Photo capture, QR scanning, video calls".to_string(),
            category: "Camera".to_string(),
        },
        "android.permission.RECORD_AUDIO" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows an application to record audio".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "Voice recording, voice calls, speech recognition".to_string(),
            category: "Microphone".to_string(),
        },
        "android.permission.READ_CONTACTS" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows an application to read the user's contacts data".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "Contact list display, caller ID, social features".to_string(),
            category: "Contacts".to_string(),
        },
        "android.permission.WRITE_CONTACTS" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows an application to write the user's contacts data".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "Adding/modifying contacts".to_string(),
            category: "Contacts".to_string(),
        },
        "android.permission.READ_EXTERNAL_STORAGE" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows reading from external storage".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "File access (deprecated on Android 13+, use scoped storage)".to_string(),
            category: "Storage".to_string(),
        },
        "android.permission.WRITE_EXTERNAL_STORAGE" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows writing to external storage".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "File writing (deprecated on Android 10+ with scoped storage)".to_string(),
            category: "Storage".to_string(),
        },
        "android.permission.MANAGE_EXTERNAL_STORAGE" => PermissionDBEntry {
            protection_level: "special".to_string(),
            description: "Allows broad access to external storage (all files access)".to_string(),
            risk_level: "critical".to_string(),
            recommended_usage: "File managers only - requires Play Store declaration".to_string(),
            category: "Storage".to_string(),
        },
        "android.permission.READ_SMS" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows an application to read SMS messages".to_string(),
            risk_level: "critical".to_string(),
            recommended_usage: "SMS apps only - requires Play Store declaration".to_string(),
            category: "SMS".to_string(),
        },
        "android.permission.SEND_SMS" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows an application to send SMS messages".to_string(),
            risk_level: "critical".to_string(),
            recommended_usage: "SMS apps only - may incur costs".to_string(),
            category: "SMS".to_string(),
        },
        "android.permission.RECEIVE_SMS" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows an application to receive SMS messages".to_string(),
            risk_level: "critical".to_string(),
            recommended_usage: "SMS apps only - requires Play Store declaration".to_string(),
            category: "SMS".to_string(),
        },
        "android.permission.READ_PHONE_STATE" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows read-only access to phone state (IMEI, IMSI, etc.)".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "Device identification, call state monitoring".to_string(),
            category: "Phone".to_string(),
        },
        "android.permission.CALL_PHONE" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows an application to initiate a phone call".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "Dialer apps, click-to-call features".to_string(),
            category: "Phone".to_string(),
        },
        "android.permission.READ_CALL_LOG" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows an application to read the user's call log".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "Dialer and call management apps".to_string(),
            category: "Phone".to_string(),
        },
        "android.permission.WRITE_CALL_LOG" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows an application to write to the user's call log".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "Call management apps".to_string(),
            category: "Phone".to_string(),
        },
        "android.permission.SYSTEM_ALERT_WINDOW" => PermissionDBEntry {
            protection_level: "special".to_string(),
            description: "Allows drawing over other applications".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "Floating windows, screen filters - requires special approval".to_string(),
            category: "Overlay".to_string(),
        },
        "android.permission.BIND_ACCESSIBILITY_SERVICE" => PermissionDBEntry {
            protection_level: "signature".to_string(),
            description: "Must be required by accessibility services".to_string(),
            risk_level: "critical".to_string(),
            recommended_usage: "Accessibility services only - requires explicit user grant".to_string(),
            category: "Accessibility".to_string(),
        },
        "android.permission.REQUEST_INSTALL_PACKAGES" => PermissionDBEntry {
            protection_level: "special".to_string(),
            description: "Allows an application to request installing packages".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "App stores and updaters only".to_string(),
            category: "Installer".to_string(),
        },
        "android.permission.QUERY_ALL_PACKAGES" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows query of any normal app on device (Play Store restricted)".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "App stores, launchers - requires Play Store declaration".to_string(),
            category: "Package".to_string(),
        },
        "android.permission.READ_MEDIA_IMAGES" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows reading image files from external storage (Android 13+)".to_string(),
            risk_level: "medium".to_string(),
            recommended_usage: "Image gallery, photo editing apps".to_string(),
            category: "Storage".to_string(),
        },
        "android.permission.READ_MEDIA_VIDEO" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows reading video files from external storage (Android 13+)".to_string(),
            risk_level: "medium".to_string(),
            recommended_usage: "Video player, video editing apps".to_string(),
            category: "Storage".to_string(),
        },
        "android.permission.READ_MEDIA_AUDIO" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows reading audio files from external storage (Android 13+)".to_string(),
            risk_level: "medium".to_string(),
            recommended_usage: "Music player, audio editing apps".to_string(),
            category: "Storage".to_string(),
        },
        "android.permission.BLUETOOTH" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows connections to Bluetooth devices".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Bluetooth communication (deprecated on Android 12+)".to_string(),
            category: "Bluetooth".to_string(),
        },
        "android.permission.BLUETOOTH_ADMIN" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows discovery and pairing of Bluetooth devices".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Bluetooth device management (deprecated on Android 12+)".to_string(),
            category: "Bluetooth".to_string(),
        },
        "android.permission.BLUETOOTH_SCAN" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows scanning for Bluetooth devices (Android 12+)".to_string(),
            risk_level: "medium".to_string(),
            recommended_usage: "BLE scanning, device discovery".to_string(),
            category: "Bluetooth".to_string(),
        },
        "android.permission.BLUETOOTH_CONNECT" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows connecting to paired Bluetooth devices (Android 12+)".to_string(),
            risk_level: "medium".to_string(),
            recommended_usage: "Bluetooth communication".to_string(),
            category: "Bluetooth".to_string(),
        },
        "android.permission.NFC" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows NFC communication".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "NFC tag reading, card emulation".to_string(),
            category: "NFC".to_string(),
        },
        "android.permission.SET_ALARM" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows setting the system alarm".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Clock and alarm apps".to_string(),
            category: "System".to_string(),
        },
        "android.permission.EXPAND_STATUS_BAR" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows expanding/collapsing the status bar".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Custom launchers, quick settings".to_string(),
            category: "System".to_string(),
        },
        "android.permission.SCHEDULE_EXACT_ALARM" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows scheduling exact alarms (Android 12+)".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Alarm clock apps requiring precise timing".to_string(),
            category: "System".to_string(),
        },
        "android.permission.USE_BIOMETRIC" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows use of biometric hardware".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Biometric authentication".to_string(),
            category: "Security".to_string(),
        },
        "android.permission.USE_FINGERPRINT" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows use of fingerprint hardware (deprecated)".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "Fingerprint authentication (use USE_BIOMETRIC instead)".to_string(),
            category: "Security".to_string(),
        },
        "com.android.vending.BILLING" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "In-app billing service permission".to_string(),
            risk_level: "low".to_string(),
            recommended_usage: "In-app purchases via Google Play Billing".to_string(),
            category: "Billing".to_string(),
        },
        "android.permission.READ_PHONE_NUMBERS" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows read access to the phone numbers".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "Phone number verification, account linking".to_string(),
            category: "Phone".to_string(),
        },
        "android.permission.BODY_SENSORS" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows access to body sensors (heart rate, etc.)".to_string(),
            risk_level: "medium".to_string(),
            recommended_usage: "Fitness and health monitoring apps".to_string(),
            category: "Sensors".to_string(),
        },
        "android.permission.ACTIVITY_RECOGNITION" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows recognition of physical activity (walking, running, etc.)".to_string(),
            risk_level: "medium".to_string(),
            recommended_usage: "Fitness tracking, pedometer".to_string(),
            category: "Sensors".to_string(),
        },
        "android.permission.GET_ACCOUNTS" => PermissionDBEntry {
            protection_level: "dangerous".to_string(),
            description: "Allows access to the list of accounts in the Account Service".to_string(),
            risk_level: "high".to_string(),
            recommended_usage: "Account selection, authentication".to_string(),
            category: "Accounts".to_string(),
        },
        "android.permission.USE_CREDENTIALS" => PermissionDBEntry {
            protection_level: "normal".to_string(),
            description: "Allows requesting authentication tokens (deprecated)".to_string(),
            risk_level: "medium".to_string(),
            recommended_usage: "Account authentication (deprecated, use AccountManager)".to_string(),
            category: "Accounts".to_string(),
        },
        _ => {
            // Classify based on name pattern
            if name.starts_with("android.permission") {
                PermissionDBEntry {
                    protection_level: "unknown".to_string(),
                    description: format!("Android system permission: {}", name),
                    risk_level: "unknown".to_string(),
                    recommended_usage: "Review documentation for this permission".to_string(),
                    category: "System".to_string(),
                }
            } else if name.starts_with("com.google.android") || name.starts_with("com.google.android.gms") {
                PermissionDBEntry {
                    protection_level: "normal".to_string(),
                    description: format!("Google services permission: {}", name),
                    risk_level: "low".to_string(),
                    recommended_usage: "Google Play Services integration".to_string(),
                    category: "Google".to_string(),
                }
            } else {
                PermissionDBEntry::default()
            }
        }
    }
}
