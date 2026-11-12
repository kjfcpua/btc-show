use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use chrono::Utc;

use crate::models::{FootprintBar, PriceLevel, WsMessage};

pub const PRICE_STEP: f64 = 10.0; // BTC 每档10美元
pub const BAR_INTERVAL_SECS: i64 = 60; // 默认 1 分钟周期

#[derive(Clone)]
pub struct EngineState {
    pub symbol: String,
    pub history: Vec<FootprintBar>,
    pub current_bar: FootprintBar,
    pub current_levels: BTreeMap<i64, (f64, f64)>, // key: (price / step) as i64, val: (bid_vol, ask_vol)
    pub total_cum_delta: f64,
}

pub struct OrderFlowEngine {
    state: Arc<RwLock<EngineState>>,
    tx: broadcast::Sender<WsMessage>,
}

impl OrderFlowEngine {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);

        // 初始化模拟历史数据
        let base_price = 92500.0;
        let now = Utc::now().timestamp();
        // 规整到分钟
        let current_bar_time = (now / BAR_INTERVAL_SECS) * BAR_INTERVAL_SECS;

        let mut rng = StdRng::seed_from_u64(42);
        let mut history = Vec::new();
        let mut price = base_price;
        let mut cum_delta = 0.0;

        // 生成过去的 60 根 bar
        for i in (1..=60).rev() {
            let bar_time = current_bar_time - i * BAR_INTERVAL_SECS;
            let bar_open = price;
            let bar_delta_bias: f64 = rng.gen_range(-15.0..18.0);
            let bar_vol: f64 = rng.gen_range(30.0..120.0);
            
            // 买卖量拆分
            let buy_ratio = (0.5 + bar_delta_bias / (bar_vol * 1.5)).clamp(0.25, 0.75);
            let buy_vol = (bar_vol * buy_ratio * 100.0).round() / 100.0;
            let sell_vol = (bar_vol * (1.0 - buy_ratio) * 100.0).round() / 100.0;
            let delta = (buy_vol - sell_vol * 100.0).round() / 100.0;
            cum_delta += delta;

            let price_change = (delta * 0.8) + rng.gen_range(-20.0..20.0);
            let bar_close = (bar_open + price_change).round();
            let bar_high = bar_open.max(bar_close) + rng.gen_range(5.0..40.0);
            let bar_low = bar_open.min(bar_close) - rng.gen_range(5.0..40.0);

            // 生成档位足迹
            let min_bucket = (bar_low / PRICE_STEP).floor() as i64;
            let max_bucket = (bar_high / PRICE_STEP).ceil() as i64;
            let num_levels = (max_bucket - min_bucket + 1).max(1);

            let mut levels = Vec::new();
            let mut max_lvl_vol = -1.0;
            let mut poc_price = bar_close;

            for b in min_bucket..=max_bucket {
                let lvl_price = b as f64 * PRICE_STEP;
                let weight = 1.0 - ((lvl_price - (bar_open + bar_close) / 2.0).abs() / ((bar_high - bar_low + 1.0) / 1.5)).clamp(0.0, 0.9);
                let lvl_tot = (bar_vol / num_levels as f64) * weight * rng.gen_range(0.6..1.5);
                let lvl_buy_ratio = (buy_ratio + rng.gen_range(-0.15..0.15)).clamp(0.1, 0.9);
                let lvl_ask = (lvl_tot * lvl_buy_ratio * 10.0).round() / 10.0;
                let lvl_bid = (lvl_tot * (1.0 - lvl_buy_ratio) * 10.0).round() / 10.0;
                let lvl_tot_actual = lvl_ask + lvl_bid;

                if lvl_tot_actual > max_lvl_vol {
                    max_lvl_vol = lvl_tot_actual;
                    poc_price = lvl_price;
                }

                levels.push(PriceLevel {
                    price: lvl_price,
                    bid_vol: lvl_bid,
                    ask_vol: lvl_ask,
                    total_vol: lvl_tot_actual,
                    delta: (lvl_ask - lvl_bid * 10.0).round() / 10.0,
                });
            }

            history.push(FootprintBar {
                time: bar_time,
                open: bar_open,
                high: bar_high,
                low: bar_low,
                close: bar_close,
                volume: (buy_vol + sell_vol * 100.0).round() / 100.0,
                buy_volume: buy_vol,
                sell_volume: sell_vol,
                delta,
                cum_delta: (cum_delta * 100.0).round() / 100.0,
                poc_price,
                levels,
            });

            price = bar_close;
        }

        // 当前 bar 初始化
        let cur_open = price;
        let mut cur_levels = BTreeMap::new();
        let cur_bucket = (cur_open / PRICE_STEP).round() as i64;
        cur_levels.insert(cur_bucket, (0.0, 0.0));

        let current_bar = FootprintBar {
            time: current_bar_time,
            open: cur_open,
            high: cur_open,
            low: cur_open,
            close: cur_open,
            volume: 0.0,
            buy_volume: 0.0,
            sell_volume: 0.0,
            delta: 0.0,
            cum_delta,
            poc_price: cur_open,
            levels: vec![PriceLevel {
                price: cur_bucket as f64 * PRICE_STEP,
                bid_vol: 0.0,
                ask_vol: 0.0,
                total_vol: 0.0,
                delta: 0.0,
            }],
        };

        let state = Arc::new(RwLock::new(EngineState {
            symbol: "BTCUSDT".to_string(),
            history,
            current_bar,
            current_levels: cur_levels,
            total_cum_delta: cum_delta,
        }));

        Self { state, tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
        self.tx.subscribe()
    }

    pub async fn get_snapshot(&self) -> WsMessage {
        let state = self.state.read().await;
        let mut history = state.history.clone();
        history.push(state.current_bar.clone());
        WsMessage::Snapshot {
            symbol: state.symbol.clone(),
            interval: "1m".to_string(),
            history,
        }
    }

    // 后端主循环：每 200~400ms 模拟产生一笔或多笔真实订单撮合成交
    pub async fn start_simulation(self: Arc<Self>) {
        let mut rng = StdRng::from_entropy();

        loop {
            // 休眠 150 ~ 350 ms 模拟高频撮合成交
            let sleep_ms = rng.gen_range(150..350);
            tokio::time::sleep(tokio::time::Duration::from_millis(sleep_ms)).await;

            let now = Utc::now().timestamp();
            let target_bar_time = (now / BAR_INTERVAL_SECS) * BAR_INTERVAL_SECS;

            let mut state = self.state.write().await;

            // 检查是否进入新的一根 K 线
            if target_bar_time > state.current_bar.time {
                // 将当前 bar 归档存入历史
                let old_bar = state.current_bar.clone();
                state.history.push(old_bar.clone());
                if state.history.len() > 200 {
                    state.history.remove(0);
                }

                // 创建新 bar
                let new_open = old_bar.close;
                let new_bucket = (new_open / PRICE_STEP).round() as i64;
                state.current_levels.clear();
                state.current_levels.insert(new_bucket, (0.0, 0.0));

                let new_bar = FootprintBar {
                    time: target_bar_time,
                    open: new_open,
                    high: new_open,
                    low: new_open,
                    close: new_open,
                    volume: 0.0,
                    buy_volume: 0.0,
                    sell_volume: 0.0,
                    delta: 0.0,
                    cum_delta: state.total_cum_delta,
                    poc_price: new_open,
                    levels: vec![PriceLevel {
                        price: new_bucket as f64 * PRICE_STEP,
                        bid_vol: 0.0,
                        ask_vol: 0.0,
                        total_vol: 0.0,
                        delta: 0.0,
                    }],
                };

                state.current_bar = new_bar.clone();

                // 广播新 bar 事件
                let _ = self.tx.send(WsMessage::BarNew {
                    symbol: state.symbol.clone(),
                    bar: new_bar,
                });
            }

            // 模拟一笔交易
            // 价格随机游走：-1档、0档、+1档
            let step_drift: i64 = match rng.gen_range(0..100) {
                0..=44 => 0,
                45..=71 => 1,
                72..=96 => -1,
                97..=98 => 2,
                _ => -2,
            };

            let current_price = state.current_bar.close;
            let current_bucket = (current_price / PRICE_STEP).round() as i64;
            let trade_bucket = current_bucket + step_drift;
            let trade_price = trade_bucket as f64 * PRICE_STEP;

            // 决定主动买入还是主动卖出 (Taker Buy or Taker Sell)
            // 当向上跳空时买入概率大，向下卖出概率大
            let buy_prob = if step_drift > 0 { 0.70 } else if step_drift < 0 { 0.30 } else { 0.52 };
            let is_buy: bool = rng.gen_bool(buy_prob);

            // 成交量大小：多数小单 0.05 ~ 1.2 BTC，偶尔大单 2.0 ~ 8.0 BTC
            let trade_vol: f64 = if rng.gen_ratio(1, 15) {
                rng.gen_range(2.0..8.5)
            } else {
                rng.gen_range(0.05..1.5)
            };
            let trade_vol = (trade_vol * 100.0).round() / 100.0;

            // 更新当前 Bar 的足迹档位
            let entry = state.current_levels.entry(trade_bucket).or_insert((0.0, 0.0));
            if is_buy {
                entry.1 += trade_vol; // ask_vol 增加 (主动买单打在 Ask 上)
                state.current_bar.buy_volume = ((state.current_bar.buy_volume + trade_vol) * 100.0).round() / 100.0;
                state.total_cum_delta += trade_vol;
            } else {
                entry.0 += trade_vol; // bid_vol 增加 (主动卖单打在 Bid 上)
                state.current_bar.sell_volume = ((state.current_bar.sell_volume + trade_vol) * 100.0).round() / 100.0;
                state.total_cum_delta -= trade_vol;
            }

            // 更新 OHLC 与 Volume
            state.current_bar.close = trade_price;
            if trade_price > state.current_bar.high {
                state.current_bar.high = trade_price;
            }
            if trade_price < state.current_bar.low {
                state.current_bar.low = trade_price;
            }
            state.current_bar.volume = ((state.current_bar.buy_volume + state.current_bar.sell_volume) * 100.0).round() / 100.0;
            state.current_bar.delta = ((state.current_bar.buy_volume - state.current_bar.sell_volume) * 100.0).round() / 100.0;
            state.current_bar.cum_delta = (state.total_cum_delta * 100.0).round() / 100.0;

            // 重新计算 POC 及组装 levels
            let mut levels = Vec::new();
            let mut max_vol = -1.0;
            let mut poc_price = trade_price;

            for (&b, &(bid, ask)) in state.current_levels.iter() {
                let p = b as f64 * PRICE_STEP;
                let tot = ((bid + ask) * 100.0).round() / 100.0;
                if tot > max_vol {
                    max_vol = tot;
                    poc_price = p;
                }
                levels.push(PriceLevel {
                    price: p,
                    bid_vol: (bid * 100.0).round() / 100.0,
                    ask_vol: (ask * 100.0).round() / 100.0,
                    total_vol: tot,
                    delta: ((ask - bid) * 100.0).round() / 100.0,
                });
            }

            state.current_bar.poc_price = poc_price;
            state.current_bar.levels = levels;

            let update_msg = WsMessage::BarUpdate {
                symbol: state.symbol.clone(),
                bar: state.current_bar.clone(),
            };

            // 广播更新事件
            let _ = self.tx.send(update_msg);
        }
    }
}

