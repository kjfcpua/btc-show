@echo off
echo ============================================================
echo   BTC Orderflow & Footprint (TradingView + Rust)
echo ============================================================
echo 正在启动 Rust 后端服务并打开网页...
start http://127.0.0.1:8080
cargo run --release
pause

