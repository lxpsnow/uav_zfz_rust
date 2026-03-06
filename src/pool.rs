use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::time::Duration;

/// 初始化数据库连接池
pub async fn create_pool(database_url: &str) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(100)
        .min_connections(20)
        .acquire_timeout(Duration::from_secs(30))  // 获取连接超时
        .idle_timeout(Duration::from_secs(300))    // 空闲超时
        .max_lifetime(Duration::from_secs(1800))   // 生命周期
        .test_before_acquire(true)                  // 连接测试
        .connect(database_url)
        .await
}

// 定义应用状态结构
pub struct AppState {
    pub db: MySqlPool,
}