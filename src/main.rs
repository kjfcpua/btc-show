mod models;
mod engine;

use std::net::SocketAddr;
use std::sync::Arc;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use futures_util::{SinkExt, StreamExt};

use engine::OrderFlowEngine;

#[derive(Clone)]
struct AppState {
    engine: Arc<OrderFlowEngine>,
}

#[tokio::main]
async fn main() {
    let engine = Arc::new(OrderFlowEngine::new());
    let state = AppState {
        engine: engine.clone(),
    };

    // 启动后台高频订单流与足迹撮合模拟任务
    let engine_worker = engine.clone();
    tokio::spawn(async move {
        engine_worker.start_simulation().await;
    });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/snapshot", get(snapshot_handler))
        .route("/ws", get(ws_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("============================================================");
    println!("   🚀 BTC TradingView & Footprint / Orderflow Server");
    println!("   📊 BTC K线、足迹图(Footprint)、成交量、买量/卖量展示系统");
    println!("   🌐 Web 服务已启动: http://127.0.0.1:8080");
    println!("   ⚡ WebSocket 实时流: ws://127.0.0.1:8080/ws");
    println!("============================================================");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index_handler() -> impl IntoResponse {
    let html = match tokio::fs::read_to_string("static/index.html").await {
        Ok(content) => content,
        Err(_) => include_str!("../static/index.html").to_string(),
    };
    Html(html)
}

async fn snapshot_handler(State(state): State<AppState>) -> impl IntoResponse {
    let snapshot = state.engine.get_snapshot().await;
    Json(snapshot)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // 客户端连入后，立即推送历史快照
    let snapshot = state.engine.get_snapshot().await;
    if let Ok(json_str) = serde_json::to_string(&snapshot) {
        if sender.send(Message::Text(json_str)).await.is_err() {
            return;
        }
    }

    // 订阅后续实时更新广播
    let mut rx = state.engine.subscribe();

    // 发送任务
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json_str) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json_str)).await.is_err() {
                    break;
                }
            }
        }
    });

    // 接收任务（保持连接活跃或心跳）
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
    });

    // 任意一端断开即退出
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
