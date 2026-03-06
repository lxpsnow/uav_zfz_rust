use serde::{Deserialize, Serialize};
use actix_web::HttpResponse;
use sqlx::FromRow; // 用于从数据库行自动映射到结构体
use chrono::NaiveDateTime; //用于映射数据库事件类型
use serde_json::Value;

//关联无人机项目uav_log表类型，日志数据存储调用
#[derive(Serialize,FromRow)]
pub struct Log{
    pub id : i32,
    pub device_name : String,
    pub event : String,
    pub pattern :i32,
    pub create_time : NaiveDateTime,
}
//上传日志数据
#[derive(Deserialize,Debug)]
pub struct FormLog{
    pub device_name : String, //设备名称
    pub event : String, //事件
    pub pattern :i32,//类型1普通，2警告
}

//缓存数据处理
#[derive(Serialize,Deserialize,FromRow)]
pub struct CacheLog{
    pub flight_time : String, //飞行时间
    pub electric : String, //剩余电量
    pub uav_status : String, //无人机状态
    pub flight_status : String, //飞行状态
}

//BroadcastBody->data模型
#[derive(Serialize,Deserialize, Debug)]
pub struct BroadcastData{
    pub data_type : String,
    pub val : Value,
}

//广播消息post参数模型
#[derive(Deserialize, Debug)]
pub struct BroadcastBody {
    pub action: String, //更新事件update uav(无人机) 温度传感器...
    pub object: String, //更新对象
    pub data : BroadcastData,
}

//心跳传值模型
#[derive(Deserialize, Debug)]
pub struct FormHeartbeat{
    pub device_name : String,
    pub device_id : String,
    pub device_type : i32,//设备类型1传感器 2 载荷
    pub val : String, //携带数据 心跳 : {type:"heart"} 携带数据 :{type:"data",val:123}
}

//查询传感器返回数值
#[derive(FromRow,Serialize,Deserialize, Debug)]
pub struct SensorData{
    pub device_id : String,
    pub device_name : String,
    pub val : String,
}


//SELECT device_name, CASE WHEN MAX(create_time) > DATE_SUB(NOW(), INTERVAL 20 SECOND) THEN '在线' ELSE '离线' END as status FROM uav_heart where device_type = 2 GROUP BY device_name ORDER BY status DESC;
//设备在线数据
#[derive(FromRow,Serialize,Debug)]
pub struct DeviceOnlineData{
    pub device_name : String, //设备名称
    pub status : String,    //在线状态
}

//统一返回结果
#[derive(Serialize,Debug)]
pub struct ApiResponse<T>{
    pub code : i32,
    pub msg : String,
    pub data : Option<T>,
}
impl<T> ApiResponse<T>
where
    T: Serialize,
{
   pub fn success(data:T)->HttpResponse{
       HttpResponse::Ok().insert_header(("Content-Type", "application/json; charset=utf-8")).json(ApiResponse{
           code:200,
           msg:"success".to_string(),
           data:Some(data),
       })
   }
   pub fn err(msg:&str)->HttpResponse{
       HttpResponse::Ok().insert_header(("Content-Type", "application/json; charset=utf-8")).json(ApiResponse::<T>{
           code:500,
           msg:msg.to_string(),
           data:None,
       })
   }
   pub fn notice(code:i32,msg:&str)->HttpResponse{
       HttpResponse::Ok().insert_header(("Content-Type", "application/json; charset=utf-8")).json(ApiResponse::<T>{
           code:code,
           msg:msg.to_string(),
           data:None,
       })
   }
}