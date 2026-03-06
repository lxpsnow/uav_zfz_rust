use crate::models::{ApiResponse, BroadcastBody, CacheLog, DeviceOnlineData, FormHeartbeat, FormLog, Log,SensorData};
use crate::pool::AppState;
use actix_web::{web,Responder};
//use sqlx::MySqlPool;
use actix_web_actors::ws;
use crate::ws_actor::MyWebSocket; // 引入刚才写的 Actor
use serde_json::json;
use sqlx::Error as SqlxError;

//获取日志列表
pub async fn get_log_list(path:web::Path<i32>,data:web::Data<AppState>) -> impl Responder{
    let pattern = path.into_inner(); 
    match sqlx::query_as::<_,Log>("select id,device_name,event,pattern,create_time from uav_log where pattern = ? order by id asc limit 30")
    .bind(pattern)
    .fetch_all(&data.db)
    .await
    {
        Ok(list) => {
            ApiResponse::<Vec<Log>>::success(list)
        },
        Err(e) => {
            ApiResponse::<Vec<Log>>::err(&format!("Database error: {}", e))
        }
    }
}

//上传日志数据
pub async fn save_log(form:web::Json<FormLog>,data:web::Data<AppState>) -> impl Responder{ 
    match sqlx::query!("insert into uav_log(device_name,event,pattern) values(?,?,?)",
    form.device_name,
    form.event,
    form.pattern
    )
    .execute(&data.db)
    .await{
        Ok(_) => {
            let fixed_message = &format!(r#"{{"action":"update","object":"log","data":"reload{}"}}"#, form.pattern);
            MyWebSocket::broadcast_message(fixed_message);
            return ApiResponse::<()>::notice(200, "日志上传成功")
        },
        Err(e) => {
            return ApiResponse::<()>::err(&format!("Database error: {}", e))
        }
    };
}

//获取缓存数据
pub async fn get_cache_log(data:web::Data<AppState>) -> impl Responder{
    match sqlx::query_as::<_,CacheLog>("select flight_time,electric,uav_status,flight_status from uav_cache order by id desc limit 1")
    .fetch_one(&data.db)
    .await
    {
        Ok(list) => {
            ApiResponse::<CacheLog>::success(list)
        },
        Err(e) => {
            ApiResponse::<CacheLog>::err(&format!("Database error: {}", e))
        }
    }
}

//上传缓存日志
pub async fn save_cache_log(form:web::Json<CacheLog>,data:web::Data<AppState>) -> impl Responder{ 
    match sqlx::query!("insert into uav_cache(flight_time,electric,uav_status,flight_status) values(?,?,?,?)",
        form.flight_time,
        form.electric,
        form.uav_status,
        form.flight_status
    )
    .execute(&data.db)
    .await{
        Ok(_) => {
            return ApiResponse::<()>::notice(200, "缓存日志上传成功")
        },
        Err(e) => {
            return ApiResponse::<()>::err(&format!("Database error: {}", e))
        }
    };
}

//发送心跳/携带数据
pub async fn send_heartbeat(form:web::Json<FormHeartbeat>,data:web::Data<AppState>)->impl Responder{ 
    match sqlx::query!("insert into uav_heart(device_name,device_id,device_type,val) values(?,?,?,?)",
        form.device_name,
        form.device_id,
        form.device_type,
        form.val
    )
    .execute(&data.db)
    .await
    {
        Ok(_) => {
            return ApiResponse::<()>::notice(200, "心跳日志上传成功")
        },
        Err(e) => {
            return ApiResponse::<()>::err(&format!("Database error: {}", e))
        }
    };
}

//获取实时在线模块数据
pub async fn get_online_data(data:web::Data<AppState>)->impl Responder{
    match sqlx::query_as::<_,DeviceOnlineData>(
        "SELECT device_name, CASE WHEN MAX(create_time) > DATE_SUB(NOW(), INTERVAL 60 SECOND) THEN '在线' ELSE '离线' END as status FROM uav_heart where device_type = 2 GROUP BY device_name ORDER BY status ASC"
    )
    .fetch_all(&data.db)
    .await
    {
        Ok(list) => {
            ApiResponse::<Vec<DeviceOnlineData>>::success(list)
        },
        Err(e) => {
            ApiResponse::<Vec<DeviceOnlineData>>::err(&format!("Database error: {}", e))
        }
    }
}

//获取传感器实时数据
pub async fn get_sensor_data(deviceid: web::Path<String>,data: web::Data<AppState>) -> impl Responder { 
    let device_id = deviceid.into_inner();
    match sqlx::query_as::<_, SensorData>("select device_name,device_id,val from uav_heart where device_id = ? AND create_time >= NOW() - INTERVAL 30 SECOND order by id desc limit 1")
    .bind(device_id)
    .fetch_one(&data.db)
    .await
    {
        Ok(data) => {
            return ApiResponse::<SensorData>::success(data);
        },
        Err(SqlxError::RowNotFound) => {
            // 404 Not Found
            ApiResponse::<SensorData>::err("未查找到30秒内的最新数据")
        },
        Err(e) => {
            return ApiResponse::<SensorData>::err(&format!("Database error: {}", e));
        }
    }
}

//创建webSocket服务
pub async fn ws_route(req: actix_web::HttpRequest, stream: web::Payload) -> impl Responder {
    // start() 方法会处理握手协议，如果成功则启动 MyWebSocket Actor
    ws::start(MyWebSocket::default(), &req, stream)
}

// 新增：广播消息到所有WebSocket客户端
pub async fn broadcast_message(body: web::Json<BroadcastBody>) -> impl Responder {
    let data = body.into_inner();
    let message= json!({
        "action" : data.action,
        "object" : data.object,
        "data" : data.data,
    });
    let message_str = message.to_string();
    // 广播消息给所有连接的客户端
    let client_count = MyWebSocket::broadcast_message(&message_str);
    
    ApiResponse::<serde_json::Value>::notice(
        200, 
        &format!("消息已广播给 {} 个客户端: {}", client_count, message)
    )
}

// 新增：发送固定消息的API
pub async fn send_fixed_message() -> impl Responder {
    let fixed_message = r#"{"type":"update"}"#;
    let client_count = MyWebSocket::broadcast_message(fixed_message);
    
    ApiResponse::<serde_json::Value>::notice(
        200, 
        &format!("固定消息已发送给 {} 个客户端", client_count)
    )
}