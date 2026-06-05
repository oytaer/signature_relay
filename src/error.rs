//! 错误处理模块 - 定义签名验签过程中可能发生的各种错误
//!
//! 本模块定义了签名验签中转服务中所有可能发生的错误类型
//! 使用thiserror宏简化错误类型的定义，提供清晰的错误信息

// 引入thiserror宏，用于定义自定义错误类型
use thiserror::Error;

/// 签名验签错误枚举类型
/// 定义签名和验签过程中可能发生的各种错误
#[derive(Error, Debug)]
pub enum SignError {
    /// RSA公钥解析错误
    /// 当RSA公钥格式不正确无法解析时触发
    #[error("RSA公钥解析失败: {0}")]
    RsaPublicKeyError(String),

    /// RSA加密错误
    /// 当使用RSA公钥加密数据失败时触发
    #[error("RSA加密失败: {0}")]
    RsaEncryptError(String),

    /// RSA解密错误
    /// 当使用RSA公钥解密数据失败时触发
    #[error("RSA解密失败: {0}")]
    RsaDecryptError(String),

    /// Base64解码错误
    /// 当无法进行Base64解码时触发
    #[error("Base64解码失败: {0}")]
    Base64DecodeError(String),

    /// 参数缺失错误
    /// 当必要的参数缺失时触发
    #[error("必要参数缺失: {0}")]
    MissingParameter(String),

    /// JSON解析错误
    /// 当无法解析JSON数据时触发
    #[error("JSON解析失败: {0}")]
    JsonParseError(String),

    /// 通用错误
    /// 用于包装其他未分类的错误
    #[error("{0}")]
    Other(String),
}

/// 实现SignError的转换方法
impl SignError {
    /// 创建参数缺失错误
    ///
    /// # 参数
    ///
    /// * `param_name` - 缺失的参数名称
    ///
    /// # 返回值
    ///
    /// 返回MissingParameter类型的错误
    pub fn missing_param(param_name: &str) -> Self {
        // 构造错误信息，指出缺失的参数名称
        SignError::MissingParameter(param_name.to_string())
    }
}

/// 为SignError实现From<std::io::Error>转换
/// 允许将IO错误自动转换为SignError
impl From<std::io::Error> for SignError {
    fn from(err: std::io::Error) -> Self {
        // 将IO错误包装为Other类型的SignError
        SignError::Other(err.to_string())
    }
}

/// 为SignError实现From<serde_json::Error>转换
/// 允许将JSON错误自动转换为SignError
impl From<serde_json::Error> for SignError {
    fn from(err: serde_json::Error) -> Self {
        // 将JSON错误包装为JsonParseError类型的SignError
        SignError::JsonParseError(err.to_string())
    }
}
