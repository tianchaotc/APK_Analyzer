use serde::{Deserialize, Serialize};

/// 日志级别
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

/// Host 调用错误
#[derive(Debug, Clone)]
pub enum HostError {
    /// 文件未找到
    NotFound(String),
    /// 参数无效
    InvalidArg(String),
    /// 操作不支持（插件未实现 export/command）
    Unsupported(String),
    /// 宿主内部错误
    Host(String),
    /// JSON 解析或其他
    Other(String),
}

impl HostError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
    pub fn invalid_arg(msg: impl Into<String>) -> Self {
        Self::InvalidArg(msg.into())
    }
    pub fn unsupported(op: &str) -> Self {
        Self::Unsupported(format!("operation '{}' not supported", op))
    }
    pub fn host(msg: impl Into<String>) -> Self {
        Self::Host(msg.into())
    }
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(m) => write!(f, "not found: {}", m),
            Self::InvalidArg(m) => write!(f, "invalid arg: {}", m),
            Self::Unsupported(m) => write!(f, "unsupported: {}", m),
            Self::Host(m) => write!(f, "host error: {}", m),
            Self::Other(m) => write!(f, "{}", m),
        }
    }
}

impl std::error::Error for HostError {}

/// 宿主能力接口。插件通过此 trait 调用宿主能力。
pub trait Host {
    /// 读取 APK 内的文件
    fn read_apk_file(&self, path: &str) -> Result<Vec<u8>, HostError>;
    /// 列出 APK 内所有文件名
    fn list_apk_files(&self) -> Result<Vec<String>, HostError>;
    /// 解析二进制 AndroidManifest.xml，返回 JSON
    fn parse_axml(&self, bytes: &[u8]) -> Result<serde_json::Value, HostError>;
    /// 解析 DEX 文件，返回 JSON
    fn parse_dex(&self, bytes: &[u8]) -> Result<serde_json::Value, HostError>;
    /// 查询其他分析器结果。key ∈ {"overview","manifest","permissions",...}
    fn get_analysis(&self, key: &str) -> Option<serde_json::Value>;
    /// 写日志
    fn log(&self, level: LogLevel, msg: &str);
}

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}
