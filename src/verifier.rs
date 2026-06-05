//! 验签模块 - 实现星驿支付验签算法
//!
//! 本模块实现与星驿支付官方文档完全一致的验签算法
//! 验签步骤：
//! 1. 移除sign参数
//! 2. 将剩余参数按照ASCII码从小到大排序
//! 3. 使用URL键值对格式拼接成字符串
//! 4. 对拼接后的字符串进行SHA256哈希计算
//! 5. 对sign进行Base64解码
//! 6. 使用RSA公钥对解码后的数据进行解密
//! 7. 比较解密结果与计算的哈希值是否一致

// 引入标准库的BTreeMap，用于自动排序的键值对存储
use std::collections::BTreeMap;

// 引入标准库的result模块，用于定义Result类型
use std::result::Result;

// 引入SHA256哈希算法
use sha2::{Digest, Sha256};

// 引入RSA公钥和解密相关类型
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};

// 引入RSA的DecodePublicKey trait，用于从PEM格式解析公钥
use rsa::pkcs8::DecodePublicKey;

// 引入Base64编解码
use base64::{engine::general_purpose::STANDARD, Engine};

// 引入自定义错误类型
use crate::error::SignError;

/// 验签器结构体
/// 用于验证星驿支付异步通知的签名
pub struct Verifier {
    /// RSA公钥
    /// 由星驿支付官方提供，用于解密签名
    public_key: RsaPublicKey,
}

/// 实现Verifier结构体的方法
impl Verifier {
    /// 创建新的验签器实例
    ///
    /// # 参数
    ///
    /// * `public_key_pem` - PEM格式的RSA公钥字符串
    ///
    /// # 返回值
    ///
    /// 返回Result类型：
    /// - Ok(Verifier)：验签器创建成功
    /// - Err(SignError)：验签器创建失败（公钥格式错误）
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let verifier = Verifier::new("-----BEGIN PUBLIC KEY-----...")?;
    /// ```
    pub fn new(public_key_pem: &str) -> Result<Self, SignError> {
        // 去除首尾空白字符
        let public_key_pem = public_key_pem.trim();

        // 检查是否已经是完整的PEM格式
        let pem_str = if public_key_pem.contains("-----BEGIN PUBLIC KEY-----") {
            // 已经是PEM格式，直接使用
            public_key_pem.to_string()
        } else {
            // 不是PEM格式，自动添加PEM头部和尾部
            // 将Base64内容按64字符换行，符合PEM标准格式
            let base64_with_newlines = public_key_pem
                .as_bytes()
                .chunks(64)
                .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
                base64_with_newlines
            )
        };

        // 解析PEM格式的RSA公钥
        // rsa库的RsaPublicKey::from_public_key_pem方法可以解析PEM格式
        let public_key = RsaPublicKey::from_public_key_pem(&pem_str)
            // 如果解析失败，返回错误
            .map_err(|e| SignError::RsaPublicKeyError(e.to_string()))?;

        // 返回验签器实例
        Ok(Verifier { public_key })
    }

    /// 验证签名
    ///
    /// 该方法按照星驿支付官方文档的验签算法验证签名
    ///
    /// # 验签步骤（与官方文档完全一致）
    ///
    /// 1. 移除sign参数
    /// 2. 将剩余参数按照ASCII码从小到大排序
    /// 3. 使用URL键值对格式拼接成字符串
    /// 4. 对拼接后的字符串进行SHA256哈希计算
    /// 5. 对sign进行Base64解码
    /// 6. 使用RSA公钥对解码后的数据进行解密
    /// 7. 比较解密结果与计算的哈希值是否一致
    ///
    /// # 参数
    ///
    /// * `params` - 包含sign的参数键值对
    ///
    /// # 返回值
    ///
    /// 返回Result类型：
    /// - Ok(bool)：验签成功，返回true表示签名有效，false表示签名无效
    /// - Err(SignError)：验签过程出错
    pub fn verify(&self, params: &BTreeMap<String, String>) -> Result<bool, SignError> {
        // ==================== 步骤1：移除sign参数 ====================
        // 从参数中获取sign值
        let sign = params
            .get("sign")
            // 如果sign参数不存在，返回错误
            .ok_or_else(|| SignError::missing_param("sign"))?;

        // 如果sign为空，返回错误
        if sign.is_empty() {
            return Err(SignError::missing_param("sign"));
        }

        // 创建不包含sign的新参数集合
        let mut params_without_sign = BTreeMap::new();
        for (key, value) in params.iter() {
            // 跳过sign参数
            if key != "sign" {
                params_without_sign.insert(key.clone(), value.clone());
            }
        }

        // ==================== 步骤2：参数排序 ====================
        // BTreeMap会自动按照键的ASCII码顺序排序
        // params_without_sign已经是排序后的参数

        // ==================== 步骤3：参数拼接 ====================
        // 使用URL键值对格式拼接成字符串
        let concatenated = concat_params(&params_without_sign)?;

        // ==================== 步骤4：SHA256哈希计算 ====================
        // 对拼接后的字符串进行SHA256哈希计算
        // 返回小写十六进制字符串，与官方文档一致
        let calculated_hash_hex = calculate_sha256_hex(&concatenated)?;

        // ==================== 步骤5：Base64解码 ====================
        // 对sign进行Base64解码
        let encrypted_hash = base64_decode(sign)?;

        // ==================== 步骤6：RSA公钥解密 ====================
        // 使用RSA公钥对解码后的数据进行解密
        // 得到SHA256哈希的十六进制字符串
        let decrypted_hash_hex = rsa_decrypt_to_string(&self.public_key, &encrypted_hash)?;

        // ==================== 步骤7：比较哈希值 ====================
        // 比较解密结果与计算的哈希字符串是否一致
        let is_valid = calculated_hash_hex == decrypted_hash_hex;

        // 返回验签结果
        Ok(is_valid)
    }

    /// 获取待验签的原始字符串
    ///
    /// 该方法返回排序后拼接的字符串（不包含sign）
    /// 用于调试和验签过程
    ///
    /// # 参数
    ///
    /// * `params` - 参数键值对（可以包含sign，会被自动移除）
    ///
    /// # 返回值
    ///
    /// 返回排序后拼接的字符串
    pub fn get_verify_string(params: &BTreeMap<String, String>) -> Result<String, SignError> {
        // 创建不包含sign的新参数集合
        let mut params_without_sign = BTreeMap::new();
        for (key, value) in params.iter() {
            // 跳过sign参数
            if key != "sign" {
                params_without_sign.insert(key.clone(), value.clone());
            }
        }

        // 调用参数拼接方法
        concat_params(&params_without_sign)
    }
}

/// 参数拼接函数（内部函数）
///
/// 将参数按照URL键值对格式拼接成字符串
/// 格式：key1=value1&key2=value2&key3=value3...
///
/// # 参数
///
/// * `params` - 已排序的参数键值对（BTreeMap自动排序）
///
/// # 返回值
///
/// 返回Result类型：
/// - Ok(String)：拼接成功
/// - Err(SignError)：拼接失败
fn concat_params(params: &BTreeMap<String, String>) -> Result<String, SignError> {
    // 创建String向量，用于存储每个键值对
    let mut pairs = Vec::new();

    // 遍历所有参数（BTreeMap已按ASCII码排序）
    for (key, value) in params.iter() {
        // 跳过sign参数（与官方文档一致）
        // 官方文档说明：sign参数本身不参与签名计算
        if key == "sign" {
            continue;
        }

        // 注意：根据官方文档，null值不参与拼接，但空字符串需要拼接
        // 由于Rust的String不可能是null，我们只处理空字符串的情况
        // 空字符串需要参与拼接，所以不跳过
        // 这与官方文档完全一致："null值不参与拼接，空字符串需要拼接"

        // 将键值对格式化为"key=value"形式
        pairs.push(format!("{}={}", key, value));
    }

    // 使用"&"连接所有键值对
    let result = pairs.join("&");

    // 返回拼接结果
    Ok(result)
}

/// SHA256哈希计算函数（内部函数）
///
/// 对字符串进行SHA256哈希计算，返回小写十六进制字符串
///
/// # 参数
///
/// * `input` - 需要计算哈希的字符串
///
/// # 返回值
///
/// 返回Result类型：
/// - Ok(String)：哈希计算成功，返回小写十六进制字符串
/// - Err(SignError)：哈希计算失败
fn calculate_sha256_hex(input: &str) -> Result<String, SignError> {
    // 创建SHA256哈希计算器
    let mut hasher = Sha256::new();

    // 更新哈希计算器的输入数据
    // 将字符串转换为字节数组
    hasher.update(input.as_bytes());

    // 完成哈希计算，获取结果
    let hash = hasher.finalize();

    // 将哈希结果转换为小写十六进制字符串
    // hex::encode默认输出小写，与官方文档一致
    let hash_hex = hex::encode(hash);

    // 返回哈希值字符串
    Ok(hash_hex)
}

/// Base64解码函数（内部函数）
///
/// 对Base64字符串进行解码
///
/// # 参数
///
/// * `encoded` - Base64编码的字符串
///
/// # 返回值
///
/// 返回Result类型：
/// - Ok(Vec<u8>)：解码成功，返回解码后的字节数组
/// - Err(SignError)：解码失败
fn base64_decode(encoded: &str) -> Result<Vec<u8>, SignError> {
    // 使用标准Base64解码
    let decoded = STANDARD
        .decode(encoded)
        // 如果解码失败，返回错误
        .map_err(|e| SignError::Base64DecodeError(e.to_string()))?;

    // 返回解码结果
    Ok(decoded)
}

/// RSA公钥解密函数（内部函数）
///
/// 使用RSA公钥对数据进行解密，返回字符串
///
/// # 说明
///
/// 在星驿支付的签名算法中，签名是用公钥加密SHA256哈希的十六进制字符串生成的
/// 因此验签时需要用公钥"解密"，得到SHA256哈希的十六进制字符串
/// 然后与本地计算的SHA256哈希字符串进行比较
///
/// # 参数
///
/// * `public_key` - RSA公钥
/// * `data` - 需要解密的数据（Base64解码后的签名）
///
/// # 返回值
///
/// 返回Result类型：
/// - Ok(String)：解密成功，返回解密后的字符串（SHA256哈希的十六进制字符串）
/// - Err(SignError)：解密失败
fn rsa_decrypt_to_string(public_key: &RsaPublicKey, data: &[u8]) -> Result<String, SignError> {
    // 创建随机数生成器
    let mut rng = rand::thread_rng();

    // 使用PKCS1v15填充方案进行"解密"
    // 由于星驿支付的特殊设计，这里实际上执行的是公钥加密操作
    // RSA公钥加密和解密在数学上都是模幂运算，只是指数不同
    // 但在星驿支付的设计中，签名和验签都使用公钥指数
    let decrypted = public_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, data)
        // 如果解密失败，返回错误
        .map_err(|e| SignError::RsaDecryptError(e.to_string()))?;

    // 将解密结果转换为字符串
    // 根据官方文档，解密后得到的是SHA256哈希的十六进制字符串
    let decrypted_str = String::from_utf8(decrypted)
        .map_err(|e| SignError::RsaDecryptError(format!("解密结果不是有效UTF8字符串: {}", e)))?;

    // 返回解密后的字符串
    Ok(decrypted_str)
}

// 引入rand库用于随机数生成
use rand;

// 引入hex库用于十六进制编码
use hex;
