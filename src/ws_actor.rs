use actix::{Actor, StreamHandler, AsyncContext, ActorContext, Addr};
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 定义 WebSocket 会话结构
pub struct MyWebSocket {
    hb: Instant, // 上次收到消息/心跳的时间戳
    pub id: usize, // 会话ID
}

// 全局连接管理器
lazy_static::lazy_static! {
    static ref CONNECTIONS: Arc<Mutex<HashMap<usize, Addr<MyWebSocket>>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

impl MyWebSocket {
    // 获取下一个可用的会话ID
    fn get_next_id() -> usize {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }
    
    // 广播消息给所有连接的客户端
    pub fn broadcast_message(message: &str) -> usize {
        let connections = CONNECTIONS.lock().unwrap();
        let count = connections.len();
        
        for (_, addr) in connections.iter() {
            addr.do_send(WebSocketMessage(message.to_string()));
        }
        
        count
    }
}

impl Default for MyWebSocket {
    fn default() -> Self {
        Self { 
            hb: Instant::now(),
            id: Self::get_next_id(),
        }
    }
}

// 定义发送给Actor的消息类型
#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct WebSocketMessage(pub String);

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WebSocket 客户端已连接 - ID: {}", self.id);
        
        // 将当前连接添加到全局Map中
        CONNECTIONS.lock().unwrap().insert(self.id, ctx.address());
        
        // 启动心跳检测
        self.hb(ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("WebSocket 客户端已断开 - ID: {}", self.id);
        
        // 从全局Map中移除
        CONNECTIONS.lock().unwrap().remove(&self.id);
    }
}

// 处理发送给Actor的消息
impl actix::Handler<WebSocketMessage> for MyWebSocket {
    type Result = ();

    fn handle(&mut self, msg: WebSocketMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

/// 处理 WebSocket 消息
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                println!("收到消息(ID:{}): {}", self.id, text);
                self.hb = Instant::now();
                
                if text.contains("\"action\":\"ping\"") || text.contains("\"action\": \"ping\"") {
                    ctx.text(r#"{"action":"pong"}"#); 
                } else {
                    ctx.text(format!("Server received: {}", text));
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                self.hb = Instant::now();
                ctx.binary(bin);
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Err(_) => {
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl MyWebSocket {
    /// 心跳检查逻辑
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_secs(15), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::from_secs(30) {
                println!("客户端心跳超时 (超过30秒无活动)，断开连接 - ID: {}", act.id);
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}