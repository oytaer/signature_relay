//! 响应模型模块 - 定义HTTP响应的数据结构
//!
//! 本模块定义签名和验签响应的数据结构
//! 所有响应都以JSON格式返回

// 引入serde的Serialize宏
use serde::Serialize;

/// 通用响应结构体
/// 所有API响应的基础结构
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// 响应状态码
    /// 0表示成功，非0表示失败
    #[serde(rename = "code")]
    pub code: i32,

    /// 响应消息
    /// 成功时为"success"，失败时为错误信息
    #[serde(rename = "message")]
    pub message: String,

    /// 响应数据
    /// 成功时包含具体数据，失败时为None
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

/// 签名响应数据结构体
/// 包含签名结果和调试信息
#[derive(Debug, Serialize)]
pub struct SignData {
    /// 签名结果
    /// 这是最终生成的签名字符串，需要传递给星驿支付API
    #[serde(rename = "sign")]
    pub sign: String,

    /// 待签名的原始字符串（调试用）
    /// 这是参数排序后拼接的字符串
    #[serde(rename = "signString", skip_serializing_if = "Option::is_none")]
    pub sign_string: Option<String>,

    /// SHA256哈希值（调试用）
    /// 这是对signString进行SHA256哈希后的十六进制表示
    #[serde(rename = "hashHex", skip_serializing_if = "Option::is_none")]
    pub hash_hex: Option<String>,
}

/// 验签响应数据结构体
/// 包含验签结果和调试信息
#[derive(Debug, Serialize)]
pub struct VerifyData {
    /// 验签结果
    /// true表示签名有效，false表示签名无效
    #[serde(rename = "valid")]
    pub valid: bool,

    /// 待验签的原始字符串（调试用）
    /// 这是参数排序后拼接的字符串（不包含sign）
    #[serde(rename = "verifyString", skip_serializing_if = "Option::is_none")]
    pub verify_string: Option<String>,
}

/// 实现ApiResponse的构造方法
impl<T: Serialize> ApiResponse<T> {
    /// 创建成功响应
    ///
    /// # 参数
    ///
    /// * `data` - 响应数据
    ///
    /// # 返回值
    ///
    /// 返回成功状态的ApiResponse实例
    pub fn success(data: T) -> Self {
        ApiResponse {
            // 状态码0表示成功
            code: 0,
            // 成功消息
            message: "success".to_string(),
            // 响应数据
            data: Some(data),
        }
    }

    /// 创建失败响应
    ///
    /// # 参数
    ///
    /// * `code` - 错误码（非0）
    /// * `message` - 错误消息
    ///
    /// # 返回值
    ///
    /// 返回失败状态的ApiResponse实例
    pub fn error(code: i32, message: String) -> Self {
        ApiResponse {
            // 错误码
            code,
            // 错误消息
            message,
            // 失败时数据为None
            data: None,
        }
    }
}
