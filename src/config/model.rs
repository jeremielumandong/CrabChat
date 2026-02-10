//! Configuration data model.
//!
//! All structs derive `Serialize`/`Deserialize` for TOML persistence.
//! Every field has a sensible default so the application works out of the box.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::nickname::generate_nickname;

/// Root application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub servers: Vec<ServerConfig>,
    #[serde(default = "default_ui")]
    pub ui: UiConfig,
    #[serde(default = "default_dcc")]
    pub dcc: DccConfig,
    #[serde(default)]
    pub behavior: BehaviorConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub ctcp: CtcpConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            servers: default_servers(),
            ui: default_ui(),
            dcc: default_dcc(),
            behavior: BehaviorConfig::default(),
            logging: LoggingConfig::default(),
            ctcp: CtcpConfig::default(),
        }
    }
}

fn default_servers() -> Vec<ServerConfig> {
    let nick = generate_nickname();
    vec![
        ServerConfig {
            name: "libera".into(),
            host: "irc.libera.chat".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec!["#crabchat".into()],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "oftc".into(),
            host: "irc.oftc.net".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "efnet".into(),
            host: "irc.efnet.org".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "undernet".into(),
            host: "irc.undernet.org".into(),
            port: 6667,
            tls: false,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "dalnet".into(),
            host: "irc.dal.net".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "rizon".into(),
            host: "irc.rizon.net".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "quakenet".into(),
            host: "irc.quakenet.org".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "ircnet".into(),
            host: "open.ircnet.net".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "snoonet".into(),
            host: "irc.snoonet.org".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "gamesurge".into(),
            host: "irc.gamesurge.net".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "esper".into(),
            host: "irc.esper.net".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "irc-hispano".into(),
            host: "irc.irc-hispano.org".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "hackint".into(),
            host: "irc.hackint.org".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "twitch".into(),
            host: "irc.chat.twitch.tv".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "slashnet".into(),
            host: "irc.slashnet.org".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "chatspike".into(),
            host: "irc.chatspike.net".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "rezosup".into(),
            host: "irc.rezosup.org".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "chathispano".into(),
            host: "irc.chathispano.com".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "europnet".into(),
            host: "irc.europnet.org".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
        ServerConfig {
            name: "interlinked".into(),
            host: "irc.interlinked.me".into(),
            port: 6697,
            tls: true,
            nickname: nick.clone(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
            alt_nicks: vec![],
            quit_message: None,
            part_message: None,
            accept_invalid_certs: true,
        },
    ]
}

/// Configuration for a single IRC server connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// User-facing label (e.g. `"libera"`).
    pub name: String,
    /// Hostname or IP address of the IRC server.
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub tls: bool,
    #[serde(default = "default_nickname")]
    pub nickname: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub realname: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub nick_password: Option<String>,
    #[serde(default)]
    pub sasl_mechanism: Option<String>,
    #[serde(default)]
    pub channels: Vec<String>,
    #[serde(default)]
    pub auto_connect: bool,
    #[serde(default)]
    pub alt_nicks: Vec<String>,
    #[serde(default)]
    pub quit_message: Option<String>,
    #[serde(default)]
    pub part_message: Option<String>,
    #[serde(default = "default_true")]
    pub accept_invalid_certs: bool,
}

/// UI appearance and behavior settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_timestamp_format")]
    pub timestamp_format: String,
    #[serde(default = "default_max_scrollback")]
    pub max_scrollback: usize,
    #[serde(default = "default_true")]
    pub parse_mirc_colors: bool,
    #[serde(default = "default_true")]
    pub highlight_urls: bool,
}

/// DCC file transfer settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DccConfig {
    #[serde(default = "default_download_dir")]
    pub download_dir: PathBuf,
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,
    #[serde(default)]
    pub reject_private_ips: bool,
    #[serde(default)]
    pub auto_accept: bool,
}

/// Client behavior settings (auto-rejoin, bell notifications, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    #[serde(default)]
    pub auto_rejoin_on_kick: bool,
    #[serde(default = "default_rejoin_delay")]
    pub rejoin_delay_secs: u64,
    #[serde(default)]
    pub bell_on_mention: bool,
    #[serde(default)]
    pub bell_on_pm: bool,
    #[serde(default = "default_quit_message")]
    pub quit_message: String,
    #[serde(default = "default_part_message")]
    pub part_message: String,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            auto_rejoin_on_kick: false,
            rejoin_delay_secs: default_rejoin_delay(),
            bell_on_mention: false,
            bell_on_pm: false,
            quit_message: default_quit_message(),
            part_message: default_part_message(),
        }
    }
}

/// Chat message logging settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_log_dir")]
    pub log_dir: String,
    #[serde(default = "default_true")]
    pub log_channels: bool,
    #[serde(default)]
    pub log_queries: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            log_dir: default_log_dir(),
            log_channels: true,
            log_queries: false,
        }
    }
}

/// CTCP (Client-To-Client Protocol) auto-reply settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtcpConfig {
    #[serde(default = "default_true")]
    pub reply_version: bool,
    #[serde(default = "default_true")]
    pub reply_ping: bool,
    #[serde(default = "default_true")]
    pub reply_time: bool,
    #[serde(default)]
    pub reply_finger: bool,
    #[serde(default = "default_version_string")]
    pub version_string: String,
    #[serde(default = "default_finger_string")]
    pub finger_string: String,
}

impl Default for CtcpConfig {
    fn default() -> Self {
        Self {
            reply_version: true,
            reply_ping: true,
            reply_time: true,
            reply_finger: false,
            version_string: default_version_string(),
            finger_string: default_finger_string(),
        }
    }
}

fn default_nickname() -> String {
    generate_nickname()
}
fn default_port() -> u16 {
    6697
}
fn default_true() -> bool {
    true
}
fn default_timestamp_format() -> String {
    "%H:%M".to_string()
}
fn default_max_scrollback() -> usize {
    10000
}
fn default_download_dir() -> PathBuf {
    PathBuf::from("./downloads")
}
fn default_max_file_size() -> u64 {
    500 * 1024 * 1024 // 500 MB
}
fn default_rejoin_delay() -> u64 {
    3
}
fn default_quit_message() -> String {
    "CrabChat".to_string()
}
fn default_part_message() -> String {
    "Leaving".to_string()
}
fn default_log_dir() -> String {
    "~/.local/share/crabchat/logs".to_string()
}
fn default_version_string() -> String {
    "CrabChat - Rust IRC Client".to_string()
}
fn default_finger_string() -> String {
    "CrabChat user".to_string()
}
fn default_ui() -> UiConfig {
    UiConfig {
        timestamp_format: default_timestamp_format(),
        max_scrollback: default_max_scrollback(),
        parse_mirc_colors: true,
        highlight_urls: true,
    }
}
fn default_dcc() -> DccConfig {
    DccConfig {
        download_dir: default_download_dir(),
        max_file_size: default_max_file_size(),
        reject_private_ips: false,
        auto_accept: false,
    }
}
