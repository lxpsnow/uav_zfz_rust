mod models;
mod pool;
mod handlers;
mod ws_actor;
//mod server_actor;

use actix_cors::Cors;
use dotenvy::dotenv;
use pool::{create_pool,AppState};
use actix_web::{App,HttpServer,web};
use handlers::{get_log_list,save_log,get_cache_log,ws_route, broadcast_message, send_fixed_message,save_cache_log,send_heartbeat,get_online_data,get_sensor_data};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://127.0.0.1:8080");
    dotenv().ok();
    
    let database_url = match std::env::var("DATABASE_URL") {
    Ok(url) => {
        println!("使用环境变量中的数据库连接");
        url
    },
    Err(_) => {
        let default_url = "mysql://uav:rZeNrPmWE6HasMwE@192.168.96.171:3306/uav?ssl-mode=DISABLED".to_string();
        println!("未找到 DATABASE_URL 环境变量，使用默认连接: {}", default_url);
        default_url
    }
};

    // 初始化连接池
    let pool = create_pool(&database_url)
        .await
        .expect("Failed to create DB pool");


    HttpServer::new(move || {
        let cors = Cors::permissive();  // 允许所有来源，适合开发
        App::new()
            .wrap(cors)  // 应用 CORS 中间件
            .app_data(web::Data::new(AppState { db: pool.clone() })) 
            .service(
                web::scope("/api")
                .route("/getLogList/{pattern}",web::get().to(get_log_list))
                .route("/getSensorData/{deviceid}",web::get().to(get_sensor_data))
                .route("/getCacheLog",web::get().to(get_cache_log))
                .route("/getOnlineData",web::get().to(get_online_data))
                .route("/saveCacheLog",web::post().to(save_cache_log))
                .route("/saveLog",web::post().to(save_log))
                .route("/saveHeartbeat",web::post().to(send_heartbeat))
                // 新增：广播消息API（带自定义消息）
                .route("/broadcast", web::post().to(broadcast_message))
                // 新增：发送固定消息API
                .route("/sendFixedMessage", web::get().to(send_fixed_message))
            )
            // --- WebSocket 路由 ---
            .route("/ws", web::get().to(ws_route))
            
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
//106服务器使用39016