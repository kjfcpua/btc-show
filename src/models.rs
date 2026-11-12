use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub bid_vol: f64,    // 主动卖出成交在买一 (Taker Sell / Bid Hit)
    pub ask_vol: f64,    // 主动买入成交在卖一 (Taker Buy / Ask Hit)
    pub total_vol: f64,  // 本档总成交量
    pub delta: f64,      // ask_vol - bid_vol
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootprintBar {
    pub time: i64,          // 秒级时间戳
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,        // 总成交量
    pub buy_volume: f64,    // 主动买量 (Total Taker Buy)
    pub sell_volume: f64,   // 主动卖量 (Total Taker Sell)
    pub delta: f64,         // 净买卖差 = buy_volume - sell_volume
    pub cum_delta: f64,     // 累计 Delta (CVD)
    pub poc_price: f64,     // Point of Control (成交量最大档位价格)
    pub levels: Vec<PriceLevel>, // 该根K线内的每个价格档位足迹
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    #[serde(rename = "snapshot")]
    Snapshot {
        symbol: String,
        interval: String,
        history: Vec<FootprintBar>,
    },
    #[serde(rename = "bar_update")]
    BarUpdate {
        symbol: String,
        bar: FootprintBar,
    },
    #[serde(rename = "bar_new")]
    BarNew {
        symbol: String,
        bar: FootprintBar,
    },
}

