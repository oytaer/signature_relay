//! 请求模型模块 - 定义HTTP请求的数据结构
//!
//! 本模块定义签名和验签请求的数据结构
//! 所有字段都与星驿支付官方文档保持一致

// 引入serde的Serialize和Deserialize宏
use serde::Deserialize;

// 引入标准库的BTreeMap，用于存储参数键值对
use std::collections::BTreeMap;

/// 签名请求结构体
/// 用于接收签名请求的JSON数据
#[derive(Debug, Deserialize)]
pub struct SignRequest {
    /// 需要签名的参数键值对
    /// 所有参数都按照星驿支付官方文档的要求传递
    /// 签名器会自动处理参数排序、拼接、哈希、加密和编码
    #[serde(rename = "params")]
    pub params: BTreeMap<String, String>,
}

/// 验签请求结构体
/// 用于接收验签请求的JSON数据
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    /// 需要验签的参数键值对
    /// 必须包含sign字段，否则验签会失败
    /// 验签器会自动移除sign参数后进行验签
    #[serde(rename = "params")]
    pub params: BTreeMap<String, String>,
}
