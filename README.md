<img width="2548" height="1263" alt="image" src="https://github.com/user-attachments/assets/4ae98ad7-072a-4a6b-9803-f7175c519756" />



BTC Order Flow & Footprint Visualization Terminal (Rust + TradingView)

1. Project Overview

This project is an institutional-grade, low-latency Order Flow and Footprint Visualization Terminal designed for Bitcoin (BTC/USDT) market microstructure analysis. The backend is implemented in Rust with an asynchronous concurrency stack (Tokio + Axum), driving a high-frequency tick simulation and footprint level-clustering engine that broadcasts via WebSocket. The frontend utilizes TradingView Lightweight Charts across multi-pane synchronized layouts, providing BTC Candlesticks, Price-Level Footprint Ladders, Total Volume, Taker Buy Volume, Taker Sell Volume, Net Delta, and Cumulative Volume Delta (CVD).

2. Key Features
TradingView Main Candlestick Pane (Candlestick & POC):
Built on TradingView Lightweight Charts with an institutional dark theme;
Ultra-smooth responsive zooming/panning with dynamic Point of Control (POC) level tracing.
Professional Footprint Price Ladder:
Dissects every candlestick into discrete price increments (e.g., $10 tick brackets);
Tabulates dual-sided order flow: Bid Volume (Taker Sells hitting bids) vs. Ask Volume (Taker Buys lifting offers);
Highlights the POC (Point of Control) with golden accent bars and star badges (★);
Detects and highlights market Imbalances (e.g., volume ratio > 3:1);
Interactive crosshair linkage: Hovering over any historical candle immediately unveils its microscopic footprint ladder.
Total Volume Pane (Volume & MA):
Volume histogram with bullish/bearish color coding, coupled with a 20-period Volume Moving Average (MA).
Buy Volume vs. Sell Volume Pane:
Distinct green (Taker Buy) and red (Taker Sell) volume histograms to visually assess aggressive buying vs. selling pressure.
Net Delta & CVD (Cumulative Volume Delta) Pane:
Calculates per-bar Net Delta (Buy Vol - Sell Vol) alongside a continuous CVD curve to pinpoint liquidity absorption and divergence patterns under Wyckoff/Order-Flow methodologies.
Multi-Chart Synchronization & Offline Capability:
All four chart panes share identical time coordinates and synced crosshairs;
Standalone TradingView JS runtime bundled locally under static/, enabling instant launch with zero third-party network dependencies.
