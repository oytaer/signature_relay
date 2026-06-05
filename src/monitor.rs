//! 监控模块 - 提供监控页面和统计信息
//!
//! 本模块实现监控页面的访问控制和数据展示
//! 必须输入正确密码才能进入监控页面，无法绕过

// 引入axum框架的extract模块
use axum::extract::{Query, State};

// 引入axum框架的Html响应类型
use axum::{
    // 用于提取表单数据
    extract::Form,
    // 用于HTML响应
    response::Html,
};

// 引入标准库的sync::Arc，用于线程安全的共享状态
use std::sync::Arc;

// 引入serde的Deserialize宏，用于解析表单数据
use serde::Deserialize;

// 引入数据库模块
use crate::database::Database;

// 引入tera模板引擎
use tera::{Context, Tera};

// 引入rust_embed宏，用于将文件嵌入到二进制中
use rust_embed::RustEmbed;

/// 嵌入的HTML模板文件
/// 在编译时将templates目录下的所有HTML文件嵌入到二进制中
#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

/// 登录表单数据
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    /// 用户输入的密码
    password: String,
}

/// 数据查询参数
#[derive(Debug, Deserialize)]
pub struct QueryParams {
    /// 页码（从1开始）
    #[serde(default = "default_page")]
    pub page: u32,

    /// 每页数量
    #[serde(default = "default_page_size")]
    pub page_size: u32,

    /// 搜索关键词（搜索客户端IP）
    #[serde(default)]
    pub search: String,

    /// 筛选类型（sign或verify或空）
    #[serde(default)]
    pub filter_type: String,

    /// 筛选结果（success或failed或空）
    #[serde(default)]
    pub filter_result: String,
}

/// 默认页码
fn default_page() -> u32 {
    1
}

/// 默认每页数量
fn default_page_size() -> u32 {
    20
}

/// 初始化模板引擎
///
/// 从嵌入的模板文件中加载HTML模板
/// 模板文件在编译时已嵌入到二进制中
///
/// # 返回值
///
/// 返回Tera模板引擎实例
pub fn init_templates() -> Tera {
    // 创建模板引擎
    let mut tera = Tera::default();

    // 从嵌入的文件中读取模板并添加到模板引擎
    // 遍历所有嵌入的文件
    for file in Templates::iter() {
        // 获取文件名
        let filename = file.as_ref();

        // 只处理.html文件
        if filename.ends_with(".html") {
            // 从嵌入的文件中读取内容
            let content = Templates::get(filename)
                .expect("嵌入的模板文件不存在")
                .data;

            // 将内容转换为UTF-8字符串
            let content_str = std::str::from_utf8(&content)
                .expect("模板文件不是有效的UTF-8编码");

            // 将模板添加到模板引擎
            tera.add_raw_template(filename, content_str)
                .expect("添加模板失败");
        }
    }

    // 返回模板引擎
    tera
}

/// 监控页面HTML处理函数（GET请求）
///
/// 显示登录页面
/// 从嵌入的二进制文件中读取HTML模板
///
/// # 参数
///
/// * `_state` - 应用状态（未使用，但需要保持路由一致性）
///
/// # 返回值
///
/// 返回HTML响应
pub async fn monitor_page_handler(
    _state: State<Arc<MonitorState>>,
) -> Html<String> {
    // 从嵌入的模板文件中读取登录页面HTML
    let html = Templates::get("login.html")
        .map(|file| {
            // 将嵌入的文件内容转换为UTF-8字符串
            std::str::from_utf8(&file.data)
                .expect("登录页面模板不是有效的UTF-8编码")
                .to_string()
        })
        .unwrap_or_else(|| "登录页面加载失败".to_string());

    // 返回HTML响应
    Html(html)
}

/// 监控页面表单处理函数（POST请求）
///
/// 处理登录表单提交
/// 验证密码，如果正确则返回监控页面
/// 如果错误则返回错误提示
///
/// # 参数
///
/// * `State(state)` - 应用状态
/// * `Query(query)` - 查询参数（分页、搜索、筛选）
/// * `Form(form)` - 表单数据（包含密码）
///
/// # 返回值
///
/// 返回HTML响应
pub async fn monitor_login_handler(
    State(state): State<Arc<MonitorState>>,
    Query(query): Query<QueryParams>,
    Form(form): Form<LoginForm>,
) -> Html<String> {
    // 验证密码
    if form.password != state.monitor_password {
        // 密码错误，渲染错误页面
        let mut context = Context::new();
        context.insert("error_message", "密码错误，请重新输入");

        let html = state.tera
            .render("login_error.html", &context)
            .unwrap_or_else(|_| "登录错误页面渲染失败".to_string());

        return Html(html);
    }

    // 密码正确，获取监控数据
    // 获取统计信息
    let statistics = match state.database.get_statistics() {
        Ok(stats) => stats,
        Err(e) => {
            return Html(format!("获取统计信息失败: {}", e));
        }
    };

    // 查询请求记录（支持搜索、筛选和分页）
    let (records, total) = match state.database.query_requests(
        query.page,
        query.page_size,
        &query.search,
        &query.filter_type,
        &query.filter_result,
    ) {
        Ok(data) => data,
        Err(e) => {
            return Html(format!("获取请求记录失败: {}", e));
        }
    };

    // 准备模板上下文
    let mut context = Context::new();

    // 插入密码（用于表单提交）
    context.insert("password", &form.password);

    // 插入统计信息
    context.insert("total_requests", &statistics.total_requests);
    context.insert("sign_requests", &statistics.sign_requests);
    context.insert("verify_requests", &statistics.verify_requests);
    context.insert("success_requests", &statistics.success_requests);
    context.insert("failed_requests", &statistics.failed_requests);
    context.insert("avg_duration_ms", &statistics.avg_duration_ms);
    context.insert("today_requests", &statistics.today_requests);
    context.insert("today_success", &statistics.today_success);
    context.insert("today_failed", &statistics.today_failed);
    context.insert("today_avg_duration_ms", &statistics.today_avg_duration_ms);

    // 插入查询参数
    context.insert("search", &query.search);
    context.insert("filter_type", &query.filter_type);
    context.insert("filter_result", &query.filter_result);
    context.insert("page", &query.page);
    context.insert("page_size", &query.page_size);
    context.insert("total", &total);

    // 计算总页数
    let total_pages = (total as f64 / query.page_size as f64).ceil() as u32;
    context.insert("total_pages", &total_pages);

    // 计算页码列表（显示当前页前后各2页，最多5个页码）
    let mut page_numbers = Vec::new();
    let start = if query.page > 2 { query.page - 2 } else { 1 };
    let end = if start + 4 > total_pages { total_pages } else { start + 4 };
    for p in start..=end {
        page_numbers.push(p);
    }
    context.insert("page_numbers", &page_numbers);

    // 插入请求记录
    context.insert("records", &records);

    // 渲染模板
    let html = state.tera
        .render("monitor.html", &context)
        .unwrap_or_else(|_| "监控页面渲染失败".to_string());

    // 返回HTML响应
    Html(html)
}

/// 监控状态结构体
/// 包含监控页面所需的共享数据
pub struct MonitorState {
    /// 监控密码
    pub monitor_password: String,

    /// 数据库实例（使用Arc包装以共享）
    pub database: Arc<Database>,

    /// 模板引擎实例
    pub tera: Tera,
}
