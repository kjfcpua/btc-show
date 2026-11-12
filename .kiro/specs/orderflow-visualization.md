# Specification: BTC Order Flow & Footprint Visualization Engine

## 1. System Goals
- Provide sub-second real-time BTC/USDT candlestick streaming.
- Deconstruct candles into discrete price level ladders ($10 per tier).
- Visualize dual-sided volume (Taker Buy / Ask Hit vs. Taker Sell / Bid Hit).
- Calculate and trace POC (Point of Control) per bar.
- Stream data over WebSocket to TradingView Lightweight Charts.

## 2. Invariants & Acceptance Criteria
- Conservation of Volume: Total Bar Volume == Buy Volume + Sell Volume.
- Delta Integrity: Delta == Buy Volume - Sell Volume.
- Footprint POC: POC Price level must have the maximum total traded volume in the bar.
- Time-sync: Crosshair and horizontal time-scales across all 4 sub-charts must be 100% synchronized.

