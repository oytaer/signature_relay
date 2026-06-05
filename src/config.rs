//! 配置模块 - 用于读取和管理.env环境变量配置
//!
//! 本模块负责从.env文件中读取RSA公钥、服务端口等配置信息
//! 确保敏感信息（如RSA公钥）不会硬编码在代码中

// 引入标准库的env模块，用于访问环境变量
use std::env;

// 引入标准库的result模块，用于定义Result类型
use std::result::Result;

// 引入thiserror宏，用于定义自定义错误类型
use thiserror::Error;

/// 配置错误枚举类型
/// 定义配置加载过程中可能发生的各种错误
#[derive(Error, Debug)]
pub enum ConfigError {
    /// RSA公钥未配置错误
    /// 当.env文件中未设置RSA_PUBLIC_KEY时触发
    #[error("RSA公钥未配置，请在.env文件中设置RSA_PUBLIC_KEY")]
    RsaPublicKeyMissing,

    /// 监控密码未配置错误
    /// 当.env文件中未设置MONITOR_PASSWORD时触发
    #[error("监控密码未配置，请在.env文件中设置MONITOR_PASSWORD")]
    MonitorPasswordMissing,
}

/// 配置结构体
/// 存储签名中转服务的所有配置信息
pub struct Config {
    /// RSA公钥字符串
    /// 由星驿支付官方提供，用于签名和验签
    /// 格式为PEM格式的Base64编码字符串
    pub rsa_public_key: String,

    /// 监控页面密码
    /// 用于访问监控页面时的身份验证
    /// 必须配置，确保监控页面安全
    pub monitor_password: String,

    /// 服务监听地址
    /// 默认为0.0.0.0，表示监听所有网络接口
    pub server_host: String,

    /// 服务监听端口
    /// 默认为8080端口
    pub server_port: u16,
}

/// 实现Config结构体的方法
impl Config {
    /// 从环境变量加载配置
    ///
    /// 该方法会从.env文件和系统环境变量中读取配置信息
    /// 如果某些配置项未设置，则使用默认值
    ///
    /// # 返回值
    ///
    /// 返回Result类型：
    /// - Ok(Config)：配置加载成功，返回Config实例
    /// - Err(ConfigError)：配置加载失败，返回错误信息
    ///
    /// # 错误情况
    ///
    /// 当RSA公钥未配置时返回错误，因为这是必须的配置项
    pub fn from_env() -> Result<Self, ConfigError> {
        // 从环境变量获取RSA公钥
        // RSA_PUBLIC_KEY是必须配置的项，没有默认值
        let rsa_public_key = env::var("RSA_PUBLIC_KEY")
            // 如果环境变量不存在，返回错误
            .map_err(|_| ConfigError::RsaPublicKeyMissing)?;

        // 从环境变量获取监控密码
        // MONITOR_PASSWORD是必须配置的项，没有默认值
        let monitor_password = env::var("MONITOR_PASSWORD")
            // 如果环境变量不存在，返回错误
            .map_err(|_| ConfigError::MonitorPasswordMissing)?;

        // 从环境变量获取服务监听地址
        // SERVER_HOST是可选配置，默认值为"0.0.0.0"
        let server_host = env::var("SERVER_HOST")
            // 如果环境变量不存在，使用默认值
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        // 从环境变量获取服务监听端口
        // SERVER_PORT是可选配置，默认值为8080
        let server_port = env::var("SERVER_PORT")
            // 如果环境变量不存在，使用默认值8080
            .unwrap_or_else(|_| "8080".to_string())
            // 将字符串解析为u16类型的端口号
            .parse::<u16>()
            // 如果解析失败，使用默认值8080
            .unwrap_or(8080);

        // 返回配置实例
        Ok(Config {
            // RSA公钥，必须配置
            rsa_public_key,
            // 监控密码，必须配置
            monitor_password,
            // 服务监听地址
            server_host,
            // 服务监听端口
            server_port,
        })
    }

    /// 获取完整的监听地址字符串
    ///
    /// 将host和port组合成完整的监听地址
    /// 格式为"host:port"，例如"0.0.0.0:8080"
    ///
    /// # 返回值
    ///
    /// 返回格式化的监听地址字符串
    pub fn listen_address(&self) -> String {
        // 使用format!宏格式化监听地址
        format!("{}:{}", self.server_host, self.server_port)
    }
}

/// 加载.env文件
///
/// 该函数会尝试从当前目录加载.env文件
/// 如果.env文件不存在，不会报错，而是使用系统环境变量
///
/// # 说明
///
/// 此函数应在程序启动时调用，确保配置信息可用
pub fn load_dotenv() {
    // 尝试加载.env文件
    // dotenvy::dotenv()返回Result，如果文件不存在会返回Err
    // 使用ok()忽略错误，继续使用系统环境变量
    dotenvy::dotenv().ok();
}
