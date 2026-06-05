//! 星驿支付签名验签中转服务主入口
//!
//! 本服务作为星驿支付API的签名验签中转器
//! 解决下级商户无法获取RSA公钥的问题
//! 由顶级服务商统一管理RSA公钥，提供签名和验签服务

// 引入标准库的sync::Arc，用于线程安全的共享状态
use std::sync::Arc;

// 引入标准库的net::SocketAddr，用于定义监听地址
use std::net::SocketAddr;

// 引入axum框架的路由模块
use axum::{
    // 路由器，用于定义HTTP路由
    routing::{get, post},
    // Router构建器
    Router,
};

// 引入tokio的信号模块，用于优雅关闭
use tokio::signal;

// 引入tower-http的CORS中间件
use tower_http::cors::{Any, CorsLayer};

// 引入tracing的日志宏
use tracing::{error, info, warn};

// 引入配置模块
mod config;

// 引入错误处理模块
mod error;

// 引入签名模块
mod signer;

// 引入验签模块
mod verifier;

// 引入模型模块
mod models;

// 引入HTTP处理模块
mod handler;

// 引入数据库模块
mod database;

// 引入监控模块
mod monitor;

/// 主函数入口
///
/// 初始化配置、签名器、验签器和HTTP服务器
/// 启动服务并监听请求
#[tokio::main]
async fn main() {
    // ==================== 步骤1：加载配置 ====================
    // 加载.env文件中的环境变量
    config::load_dotenv();

    // 初始化日志系统
    // 设置日志级别为info，输出到标准输出
    // 使用tracing_subscriber的fmt模块初始化
    tracing_subscriber::fmt()
        // 设置日志级别过滤器，默认为info级别
        .with_max_level(tracing::Level::INFO)
        // 初始化日志订阅者
        .init();

    // 记录服务启动日志
    info!("星驿支付签名验签中转服务启动中...");

    // 从环境变量加载配置
    let app_config = match config::Config::from_env() {
        // 配置加载成功
        Ok(cfg) => {
            // 记录配置信息
            info!(
                "配置加载成功 - 监听地址: {}",
                cfg.listen_address()
            );
            cfg
        }
        // 配置加载失败
        Err(e) => {
            // 记录错误日志
            error!("配置加载失败: {}", e);
            // 退出程序
            return;
        }
    };

    // ==================== 步骤2：初始化签名器和验签器 ====================
    // 创建签名器实例
    let signer = match signer::Signer::new(&app_config.rsa_public_key) {
        // 签名器创建成功
        Ok(s) => {
            info!("签名器初始化成功");
            s
        }
        // 签名器创建失败
        Err(e) => {
            error!("签名器初始化失败: {}", e);
            return;
        }
    };

    // 创建验签器实例
    let verifier = match verifier::Verifier::new(&app_config.rsa_public_key) {
        // 验签器创建成功
        Ok(v) => {
            info!("验签器初始化成功");
            v
        }
        // 验签器创建失败
        Err(e) => {
            error!("验签器初始化失败: {}", e);
            return;
        }
    };

    // ==================== 步骤2.5：初始化数据库 ====================
    // 创建数据库实例
    let database = match database::Database::new() {
        // 数据库创建成功
        Ok(db) => {
            info!("数据库初始化成功");
            db
        }
        // 数据库创建失败
        Err(e) => {
            error!("数据库初始化失败: {}", e);
            return;
        }
    };

    // 将数据库包装在Arc中，实现线程安全共享
    let database_arc = Arc::new(database);

    // ==================== 步骤3：创建应用状态 ====================
    // 将签名器、验签器和数据库包装在Arc中，实现线程安全共享
    let app_state = Arc::new(handler::AppState {
        // 签名器共享引用
        signer: Arc::new(signer),
        // 验签器共享引用
        verifier: Arc::new(verifier),
        // 数据库共享引用（与monitor_state共享同一个数据库实例）
        database: database_arc.clone(),
    });

    // 初始化模板引擎
    info!("初始化模板引擎...");
    let tera = monitor::init_templates();

    // 创建监控状态
    let monitor_state = Arc::new(monitor::MonitorState {
        // 监控密码
        monitor_password: app_config.monitor_password.clone(),
        // 数据库引用（与app_state共享同一个数据库实例）
        database: database_arc.clone(),
        // 模板引擎
        tera,
    });

    // ==================== 步骤4：配置CORS中间件 ====================
    // 允许跨域请求，方便前端调用
    let cors = CorsLayer::new()
        // 允许所有来源
        .allow_origin(Any)
        // 允许所有HTTP方法
        .allow_methods(Any)
        // 允许所有HTTP头
        .allow_headers(Any);

    // ==================== 步骤5：构建路由 ====================
    // 创建API路由器（签名验签接口）
    let api_routes = Router::new()
        // 签名接口：POST /sign
        .route("/sign", post(handler::sign_handler))
        // 验签接口：POST /verify
        .route("/verify", post(handler::verify_handler))
        // 健康检查接口：GET /health
        .route("/health", get(handler::health_handler))
        // 添加应用状态
        .with_state(app_state);

    // 创建监控路由器
    let monitor_routes = Router::new()
        // 监控页面（显示登录页面）：GET /monitor
        .route("/monitor", get(monitor::monitor_page_handler))
        // 监控登录处理（表单提交）：POST /monitor/login
        .route("/monitor/login", post(monitor::monitor_login_handler))
        // 添加监控状态
        .with_state(monitor_state);

    // 合并路由器
    let app = Router::new()
        // 合并API路由
        .merge(api_routes)
        // 合并监控路由
        .merge(monitor_routes)
        // 添加CORS中间件
        .layer(cors);

    // ==================== 步骤6：启动HTTP服务器 ====================
    // 解析监听地址
    let addr: SocketAddr = match app_config.listen_address().parse() {
        // 地址解析成功
        Ok(a) => a,
        // 地址解析失败
        Err(e) => {
            error!("监听地址解析失败: {}", e);
            return;
        }
    };

    // 记录服务启动信息
    info!("服务启动成功，监听地址: {}", addr);
    info!("可用接口:");
    info!("  POST /sign          - 签名接口");
    info!("  POST /verify        - 验签接口");
    info!("  GET  /health        - 健康检查接口");
    info!("  GET  /monitor       - 监控登录页面");
    info!("  POST /monitor/login - 监控登录验证（表单提交密码）");

    // 绑定TCP监听器
    let listener = match tokio::net::TcpListener::bind(addr).await {
        // 绑定成功
        Ok(l) => l,
        // 绑定失败
        Err(e) => {
            error!("TCP监听器绑定失败: {}", e);
            return;
        }
    };

    // 启动服务器，使用优雅关闭
    // 当收到Ctrl+C信号时，优雅关闭服务器
    axum::serve(listener, app)
        // 配置优雅关闭
        .with_graceful_shutdown(shutdown_signal())
        // 等待服务器运行
        .await
        // 处理服务器运行错误
        .unwrap_or_else(|e| {
            error!("服务器运行错误: {}", e);
        });

    // 记录服务关闭日志
    info!("服务已关闭");
}

/// 优雅关闭信号处理函数
///
/// 等待Ctrl+C信号，触发服务器优雅关闭
///
/// # 返回值
///
/// 返回一个Future，当收到信号时完成
async fn shutdown_signal() {
    // 等待Ctrl+C信号
    let ctrl_c = async {
        // 等待信号
        signal::ctrl_c()
            .await
            // 如果等待失败，记录错误
            .expect("无法安装Ctrl+C信号处理器");
    };

    // 等待终止信号（Unix系统）
    #[cfg(unix)]
    let terminate = async {
        // 等待SIGTERM信号
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("无法安装SIGTERM信号处理器")
            .recv()
            .await;
    };

    // Windows系统没有SIGTERM信号
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // 等待任一信号到达
    tokio::select! {
        // Ctrl+C信号
        _ = ctrl_c => {
            warn!("收到Ctrl+C信号，开始优雅关闭...");
        },
        // 终止信号
        _ = terminate => {
            warn!("收到终止信号，开始优雅关闭...");
        },
    }
}
