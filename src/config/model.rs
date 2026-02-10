use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub servers: Vec<ServerConfig>,
    #[serde(default = "default_ui")]
    pub ui: UiConfig,
    #[serde(default = "default_dcc")]
    pub dcc: DccConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            servers: default_servers(),
            ui: default_ui(),
            dcc: default_dcc(),
        }
    }
}

fn default_servers() -> Vec<ServerConfig> {
    vec![
        ServerConfig {
            name: "libera".into(),
            host: "irc.libera.chat".into(),
            port: 6697,
            tls: true,
            nickname: "ircchat_user".into(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec!["#ircchat".into()],
            auto_connect: false,
        },
        ServerConfig {
            name: "undernet".into(),
            host: "irc.undernet.org".into(),
            port: 6667,
            tls: false,
            nickname: "ircchat_user".into(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
        },
        ServerConfig {
            name: "efnet".into(),
            host: "irc.efnet.org".into(),
            port: 6697,
            tls: true,
            nickname: "ircchat_user".into(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
        },
        ServerConfig {
            name: "dalnet".into(),
            host: "irc.dal.net".into(),
            port: 6697,
            tls: true,
            nickname: "ircchat_user".into(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
        },
        ServerConfig {
            name: "oftc".into(),
            host: "irc.oftc.net".into(),
            port: 6697,
            tls: true,
            nickname: "ircchat_user".into(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
        },
        ServerConfig {
            name: "rizon".into(),
            host: "irc.rizon.net".into(),
            port: 6697,
            tls: true,
            nickname: "ircchat_user".into(),
            username: None,
            realname: None,
            password: None,
            nick_password: None,
            sasl_mechanism: None,
            channels: vec![],
            auto_connect: false,
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub tls: bool,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_timestamp_format")]
    pub timestamp_format: String,
    #[serde(default = "default_max_scrollback")]
    pub max_scrollback: usize,
}

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
fn default_ui() -> UiConfig {
    UiConfig {
        timestamp_format: default_timestamp_format(),
        max_scrollback: default_max_scrollback(),
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
