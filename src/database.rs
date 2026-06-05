//! 数据库模块 - 管理SQLite数据库操作
//!
//! 本模块负责数据库的初始化、数据记录和统计查询
//! 使用SQLite嵌入式数据库，无需额外运行环境

// 引入rusqlite的Connection类型
use rusqlite::Connection;

// 引入rusqlite的Result类型
use rusqlite::Result as SqliteResult;

// 引入标准库的sync::Mutex，用于线程安全
use std::sync::Mutex;

// 引入chrono的Utc，用于处理时间戳
use chrono::Utc;

// 引入serde的Serialize宏，用于序列化统计结果
use serde::Serialize;

/// 数据库结构体
/// 封装SQLite连接，提供数据记录和查询功能
/// 使用Mutex包装Connection以支持多线程访问
pub struct Database {
    /// SQLite数据库连接（使用Mutex包装以支持多线程）
    conn: Mutex<Connection>,
}

/// 请求记录结构体
/// 表示一次签名或验签请求的完整信息
#[derive(Debug, Clone, Serialize)]
pub struct RequestRecord {
    /// 记录ID（自增主键）
    pub id: i64,

    /// 请求类型（sign或verify）
    pub request_type: String,

    /// 请求时间戳（UTC时间）
    pub timestamp: String,

    /// 请求参数数量
    pub param_count: i32,

    /// 处理结果（success或failed）
    pub result: String,

    /// 处理耗时（毫秒）
    pub duration_ms: i64,

    /// 客户端IP地址
    pub client_ip: String,
}

/// 统计信息结构体
/// 包含各种统计数据
#[derive(Debug, Serialize)]
pub struct Statistics {
    /// 总请求数
    pub total_requests: i64,

    /// 签名请求数
    pub sign_requests: i64,

    /// 验签请求数
    pub verify_requests: i64,

    /// 成功请求数
    pub success_requests: i64,

    /// 失败请求数
    pub failed_requests: i64,

    /// 平均处理耗时（毫秒）
    pub avg_duration_ms: f64,

    /// 今日请求数
    pub today_requests: i64,

    /// 今日成功请求数
    pub today_success: i64,

    /// 今日失败请求数
    pub today_failed: i64,

    /// 今日平均耗时（毫秒）
    pub today_avg_duration_ms: f64,
}

/// 实现Database结构体的方法
impl Database {
    /// 创建或打开数据库
    ///
    /// 如果数据库文件不存在，会自动创建
    /// 数据库文件位于程序运行目录下的relay.db
    ///
    /// # 返回值
    ///
    /// 返回SqliteResult类型：
    /// - Ok(Database)：数据库初始化成功
    /// - Err(SqliteError)：数据库初始化失败
    pub fn new() -> SqliteResult<Self> {
        // 数据库文件路径：当前目录下的relay.db
        let db_path = "relay.db";

        // 打开或创建数据库文件
        // 如果文件不存在，Connection::open会自动创建
        let conn = Connection::open(db_path)?;

        // 创建请求记录表
        conn.execute(
            // SQL语句：创建requests表
            "CREATE TABLE IF NOT EXISTS requests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                request_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                param_count INTEGER NOT NULL,
                result TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                client_ip TEXT NOT NULL
            )",
            // 无参数
            [],
        )?;

        // 返回Database实例，使用Mutex包装Connection
        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    /// 记录一次请求
    ///
    /// 将请求信息保存到数据库
    ///
    /// # 参数
    ///
    /// * `request_type` - 请求类型（sign或verify）
    /// * `param_count` - 参数数量
    /// * `result` - 处理结果（success或failed）
    /// * `duration_ms` - 处理耗时（毫秒）
    /// * `client_ip` - 客户端IP地址
    ///
    /// # 返回值
    ///
    /// 返回SqliteResult类型
    pub fn record_request(
        &self,
        request_type: &str,
        param_count: i32,
        result: &str,
        duration_ms: i64,
        client_ip: &str,
    ) -> SqliteResult<()> {
        // 获取当前UTC时间戳
        let timestamp = Utc::now().to_rfc3339();

        // 锁定Mutex以访问连接
        let conn = self.conn.lock().unwrap();

        // 插入记录到数据库
        conn.execute(
            // SQL语句：插入请求记录
            "INSERT INTO requests (request_type, timestamp, param_count, result, duration_ms, client_ip)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            // 参数列表
            [request_type, &timestamp, &param_count.to_string(), result, &duration_ms.to_string(), client_ip],
        )?;

        // 返回成功
        Ok(())
    }

    /// 获取统计信息
    ///
    /// 查询数据库获取各种统计数据
    ///
    /// # 返回值
    ///
    /// 返回SqliteResult类型：
    /// - Ok(Statistics)：统计信息
    /// - Err(SqliteError)：查询失败
    pub fn get_statistics(&self) -> SqliteResult<Statistics> {
        // 锁定Mutex以访问连接
        let conn = self.conn.lock().unwrap();

        // 查询总请求数
        let total_requests: i64 = conn.query_row(
            "SELECT COUNT(*) FROM requests",
            [],
            |row| row.get(0),
        )?;

        // 查询签名请求数
        let sign_requests: i64 = conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE request_type = 'sign'",
            [],
            |row| row.get(0),
        )?;

        // 查询验签请求数
        let verify_requests: i64 = conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE request_type = 'verify'",
            [],
            |row| row.get(0),
        )?;

        // 查询成功请求数
        let success_requests: i64 = conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE result = 'success'",
            [],
            |row| row.get(0),
        )?;

        // 查询失败请求数
        let failed_requests: i64 = conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE result = 'failed'",
            [],
            |row| row.get(0),
        )?;

        // 查询平均处理耗时
        let avg_duration_ms: f64 = conn.query_row(
            "SELECT COALESCE(AVG(duration_ms), 0) FROM requests",
            [],
            |row| row.get(0),
        )?;

        // 获取今日日期（UTC）
        let today = Utc::now().format("%Y-%m-%d").to_string();

        // 查询今日请求数
        let today_requests: i64 = conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE timestamp LIKE ?1",
            [&format!("{}%", today)],
            |row| row.get(0),
        )?;

        // 查询今日成功请求数
        let today_success: i64 = conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE timestamp LIKE ?1 AND result = 'success'",
            [&format!("{}%", today)],
            |row| row.get(0),
        )?;

        // 查询今日失败请求数
        let today_failed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM requests WHERE timestamp LIKE ?1 AND result = 'failed'",
            [&format!("{}%", today)],
            |row| row.get(0),
        )?;

        // 查询今日平均耗时
        let today_avg_duration_ms: f64 = conn.query_row(
            "SELECT COALESCE(AVG(duration_ms), 0) FROM requests WHERE timestamp LIKE ?1",
            [&format!("{}%", today)],
            |row| row.get(0),
        )?;

        // 返回统计信息
        Ok(Statistics {
            total_requests,
            sign_requests,
            verify_requests,
            success_requests,
            failed_requests,
            avg_duration_ms,
            today_requests,
            today_success,
            today_failed,
            today_avg_duration_ms,
        })
    }

    /// 查询请求记录（支持搜索、筛选和分页）
    ///
    /// # 参数
    ///
    /// * `page` - 页码（从1开始）
    /// * `page_size` - 每页数量
    /// * `search` - 搜索关键词（搜索客户端IP）
    /// * `filter_type` - 筛选类型（sign或verify或空）
    /// * `filter_result` - 筛选结果（success或failed或空）
    ///
    /// # 返回值
    ///
    /// 返回SqliteResult类型，包含请求记录列表和总数
    pub fn query_requests(
        &self,
        page: u32,
        page_size: u32,
        search: &str,
        filter_type: &str,
        filter_result: &str,
    ) -> SqliteResult<(Vec<RequestRecord>, i64)> {
        // 锁定Mutex以访问连接
        let conn = self.conn.lock().unwrap();

        // 构建WHERE条件
        let mut conditions = Vec::new();
        let mut params: Vec<String> = Vec::new();

        // 添加搜索条件
        if !search.is_empty() {
            conditions.push("client_ip LIKE ?".to_string());
            params.push(format!("%{}%", search));
        }

        // 添加类型筛选
        if !filter_type.is_empty() {
            conditions.push("request_type = ?".to_string());
            params.push(filter_type.to_string());
        }

        // 添加结果筛选
        if !filter_result.is_empty() {
            conditions.push("result = ?".to_string());
            params.push(filter_result.to_string());
        }

        // 构建WHERE子句
        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // 计算偏移量
        let offset = (page.saturating_sub(1)) * page_size;

        // 查询总数
        let count_sql = format!(
            "SELECT COUNT(*) FROM requests {}",
            where_clause
        );

        let total: i64 = if params.is_empty() {
            conn.query_row(&count_sql, [], |row| row.get(0))?
        } else {
            let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
            conn.query_row(&count_sql, params_refs.as_slice(), |row| row.get(0))?
        };

        // 查询数据
        let data_sql = format!(
            "SELECT id, request_type, timestamp, param_count, result, duration_ms, client_ip
             FROM requests {}
             ORDER BY id DESC
             LIMIT ? OFFSET ?",
            where_clause
        );

        // 添加分页参数
        params.push(page_size.to_string());
        params.push(offset.to_string());

        // 执行查询
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        let mut stmt = conn.prepare(&data_sql)?;
        let records = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(RequestRecord {
                id: row.get(0)?,
                request_type: row.get(1)?,
                timestamp: row.get(2)?,
                param_count: row.get(3)?,
                result: row.get(4)?,
                duration_ms: row.get(5)?,
                client_ip: row.get(6)?,
            })
        })?;

        // 收集结果到Vec
        let mut result = Vec::new();
        for record in records {
            result.push(record?);
        }

        // 返回结果和总数
        Ok((result, total))
    }
}
