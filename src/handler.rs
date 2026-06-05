//! HTTP处理模块 - 处理签名和验签的HTTP请求
//!
//! 本模块提供HTTP API接口，接收用户的签名和验签请求
//! 并返回JSON格式的响应结果

// 引入axum框架的extract模块，用于提取请求体和状态
use axum::extract::State;

// 引入axum框架的Json类型，用于JSON请求和响应
use axum::Json;

// 引入标准库的sync::Arc，用于线程安全的共享状态
use std::sync::Arc;

// 引入标准库的time::Instant，用于计算处理耗时
use std::time::Instant;

// 引入签名器
use crate::signer::Signer;

// 引入验签器
use crate::verifier::Verifier;

// 引入数据库
use crate::database::Database;

// 引入请求模型
use crate::models::request::{SignRequest, VerifyRequest};

// 引入响应模型
use crate::models::response::{ApiResponse, SignData, VerifyData};

// 引入日志宏
use tracing::{error, info};

/// 应用状态结构体
/// 包含签名器、验签器和数据库的共享引用
#[derive(Clone)]
pub struct AppState {
    /// 签名器实例
    /// 用于生成签名
    pub signer: Arc<Signer>,

    /// 验签器实例
    /// 用于验证签名
    pub verifier: Arc<Verifier>,

    /// 数据库实例
    /// 用于记录请求日志
    pub database: Arc<Database>,
}

/// 签名接口处理函数
///
/// 接收用户的签名请求，生成签名并返回结果
///
/// # 请求路径
///
/// POST /sign
///
/// # 请求体格式
///
/// ```json
/// {
///   "params": {
///     "key1": "value1",
///     "key2": "value2",
///     ...
///   }
/// }
/// ```
///
/// # 响应体格式
///
/// ```json
/// {
///   "code": 0,
///   "message": "success",
///   "data": {
///     "sign": "签名结果字符串",
///     "signString": "待签名的原始字符串（调试用）",
///     "hashHex": "SHA256哈希值（调试用）"
///   }
/// }
/// ```
///
/// # 参数
///
/// * `state` - 应用状态，包含签名器实例
/// * `request` - 签名请求，包含需要签名的参数
///
/// # 返回值
///
/// 返回JSON格式的响应
pub async fn sign_handler(
    // 提取应用状态
    State(state): State<Arc<AppState>>,
    // 提取JSON请求体
    Json(request): Json<SignRequest>,
) -> Json<ApiResponse<SignData>> {
    // 记录开始时间，用于计算处理耗时
    let start_time = Instant::now();

    // 记录请求日志
    info!("收到签名请求，参数数量: {}", request.params.len());

    // 获取参数数量
    let param_count = request.params.len() as i32;

    // 调用签名器生成签名
    match state.signer.sign(&request.params) {
        // 签名成功
        Ok(sign) => {
            // 记录成功日志
            info!("签名生成成功");

            // 计算处理耗时（毫秒）
            let duration_ms = start_time.elapsed().as_millis() as i64;

            // 记录到数据库
            // 如果记录失败，只记录错误日志，不影响主流程
            if let Err(e) = state.database.record_request(
                "sign",           // 请求类型
                param_count,      // 参数数量
                "success",        // 处理结果
                duration_ms,      // 处理耗时
                "unknown",        // 客户端IP（暂无法获取）
            ) {
                error!("记录请求日志失败: {}", e);
            }

            // 获取调试信息（待签名字符串）
            let sign_string = Signer::get_sign_string(&request.params).ok();

            // 获取调试信息（SHA256哈希值）
            let hash_hex = Signer::get_hash_hex(&request.params).ok();

            // 构造响应数据
            let data = SignData {
                // 签名结果
                sign,
                // 待签名字符串（调试用）
                sign_string,
                // SHA256哈希值（调试用）
                hash_hex,
            };

            // 返回成功响应
            Json(ApiResponse::success(data))
        }
        // 签名失败
        Err(e) => {
            // 记录错误日志
            error!("签名生成失败: {}", e);

            // 计算处理耗时（毫秒）
            let duration_ms = start_time.elapsed().as_millis() as i64;

            // 记录到数据库
            if let Err(db_err) = state.database.record_request(
                "sign",           // 请求类型
                param_count,      // 参数数量
                "failed",         // 处理结果
                duration_ms,      // 处理耗时
                "unknown",        // 客户端IP
            ) {
                error!("记录请求日志失败: {}", db_err);
            }

            // 返回错误响应
            Json(ApiResponse::error(1, format!("签名失败: {}", e)))
        }
    }
}

/// 验签接口处理函数
///
/// 接收用户的验签请求，验证签名并返回结果
///
/// # 请求路径
///
/// POST /verify
///
/// # 请求体格式
///
/// ```json
/// {
///   "params": {
///     "key1": "value1",
///     "key2": "value2",
///     "sign": "签名字符串",
///     ...
///   }
/// }
/// ```
///
/// # 响应体格式
///
/// ```json
/// {
///   "code": 0,
///   "message": "success",
///   "data": {
///     "valid": true,
///     "verifyString": "待验签的原始字符串（调试用）"
///   }
/// }
/// ```
///
/// # 参数
///
/// * `state` - 应用状态，包含验签器实例
/// * `request` - 验签请求，包含需要验签的参数（必须包含sign）
///
/// # 返回值
///
/// 返回JSON格式的响应
pub async fn verify_handler(
    // 提取应用状态
    State(state): State<Arc<AppState>>,
    // 提取JSON请求体
    Json(request): Json<VerifyRequest>,
) -> Json<ApiResponse<VerifyData>> {
    // 记录开始时间，用于计算处理耗时
    let start_time = Instant::now();

    // 记录请求日志
    info!("收到验签请求，参数数量: {}", request.params.len());

    // 获取参数数量
    let param_count = request.params.len() as i32;

    // 调用验签器验证签名
    match state.verifier.verify(&request.params) {
        // 验签成功（签名有效或无效）
        Ok(valid) => {
            // 记录验签结果日志
            if valid {
                info!("验签成功，签名有效");
            } else {
                info!("验签失败，签名无效");
            }

            // 计算处理耗时（毫秒）
            let duration_ms = start_time.elapsed().as_millis() as i64;

            // 记录到数据库
            if let Err(e) = state.database.record_request(
                "verify",         // 请求类型
                param_count,      // 参数数量
                "success",        // 处理结果
                duration_ms,      // 处理耗时
                "unknown",        // 客户端IP
            ) {
                error!("记录请求日志失败: {}", e);
            }

            // 获取调试信息（待验签字符串）
            let verify_string = Verifier::get_verify_string(&request.params).ok();

            // 构造响应数据
            let data = VerifyData {
                // 验签结果
                valid,
                // 待验签字符串（调试用）
                verify_string,
            };

            // 返回成功响应
            Json(ApiResponse::success(data))
        }
        // 验签过程出错
        Err(e) => {
            // 记录错误日志
            error!("验签过程出错: {}", e);

            // 计算处理耗时（毫秒）
            let duration_ms = start_time.elapsed().as_millis() as i64;

            // 记录到数据库
            if let Err(db_err) = state.database.record_request(
                "verify",         // 请求类型
                param_count,      // 参数数量
                "failed",         // 处理结果
                duration_ms,      // 处理耗时
                "unknown",        // 客户端IP
            ) {
                error!("记录请求日志失败: {}", db_err);
            }

            // 返回错误响应
            Json(ApiResponse::error(1, format!("验签失败: {}", e)))
        }
    }
}

/// 健康检查接口处理函数
///
/// 用于检查服务是否正常运行
///
/// # 请求路径
///
/// GET /health
///
/// # 响应体格式
///
/// ```json
/// {
///   "code": 0,
///   "message": "success",
///   "data": "ok"
/// }
/// ```
///
/// # 返回值
///
/// 返回JSON格式的响应
pub async fn health_handler() -> Json<ApiResponse<String>> {
    // 记录健康检查日志
    info!("收到健康检查请求");

    // 返回成功响应
    Json(ApiResponse::success("ok".to_string()))
}
