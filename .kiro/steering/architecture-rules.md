# Steering Rules: Rust Async Orderflow & Front-end Guidelines

1. **Rust Backend Conventions**:
   - Use Axum 0.7+ and Tokio asynchronous runtime.
   - Zero blocking calls in async tasks; offload heavy computations or simulation loops.
   - Enforce data serializability with serde for sub-millisecond JSON payloads.

2. **Frontend Charting Rules**:
   - Utilize TradingView Lightweight Charts (v4.2+).
   - Bundle dependencies locally under `/static` to support offline execution.
   - Keep theme dark (#131722) for institutional consistency.

