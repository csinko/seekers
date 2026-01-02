/// User-Agent header for API requests
pub const USER_AGENT: &str = concat!("Seekers/", env!("CARGO_PKG_VERSION"));

/// Claude API base URL
pub const CLAUDE_API_BASE: &str = "https://claude.ai/api";

/// Claude website URL
pub const CLAUDE_URL: &str = "https://claude.ai";

/// Config directory name (under ~/.config/)
pub const CONFIG_DIR_NAME: &str = "seekers";

/// Credentials filename
pub const CREDENTIALS_FILE: &str = "credentials.json";

/// Settings filename
pub const SETTINGS_FILE: &str = "settings.json";

/// File permissions for credentials (owner read/write only)
#[cfg(unix)]
pub const SECURE_FILE_MODE: u32 = 0o600;

/// Tray icon ID
pub const TRAY_ID: &str = "main-tray";

/// Default tray title when no data
pub const TRAY_TITLE_DEFAULT: &str = "--%";

/// Menu item IDs
pub mod menu {
    pub const OPEN_CLAUDE: &str = "open-claude";
    pub const REFRESH: &str = "refresh";
    pub const SETTINGS: &str = "settings";
    pub const QUIT: &str = "quit";
}

/// Time constants
pub mod time {
    /// Seconds per minute
    pub const SECONDS_PER_MINUTE: u64 = 60;

    /// Minutes per hour
    pub const MINUTES_PER_HOUR: i64 = 60;

    /// Hours per day
    pub const HOURS_PER_DAY: i64 = 24;

    /// Hours threshold for "tomorrow" display
    pub const HOURS_TOMORROW_THRESHOLD: i64 = 48;

    /// Fallback check interval when auto-refresh is disabled (seconds)
    pub const DISABLED_REFRESH_CHECK_SECS: u64 = 60;
}

/// Progress bar characters
pub mod progress {
    pub const CIRCLES: (&str, &str) = ("●", "○");
    pub const BLOCKS: (&str, &str) = ("▰", "▱");
    pub const BAR: (&str, &str) = ("█", "░");
    pub const DOTS: (&str, &str) = ("⬤", "○");
}
