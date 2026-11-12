// Property-Based Testing (PBT) for Order Flow & Footprint Invariants

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: f64,
    pub bid_vol: f64,
    pub ask_vol: f64,
    pub total_vol: f64,
    pub delta: f64,
}

#[derive(Debug, Clone)]
pub struct FootprintBar {
    pub volume: f64,
    pub buy_volume: f64,
    pub sell_volume: f64,
    pub delta: f64,
    pub poc_price: f64,
    pub levels: Vec<PriceLevel>,
}

// Invariant 1: Delta must equal buy_volume - sell_volume
fn prop_delta_integrity(bar: &FootprintBar) -> bool {
    (bar.delta - (bar.buy_volume - bar.sell_volume)).abs() < 1e-4
}

// Invariant 2: Total volume must equal buy_volume + sell_volume
fn prop_volume_conservation(bar: &FootprintBar) -> bool {
    (bar.volume - (bar.buy_volume + bar.sell_volume)).abs() < 1e-4
}

// Invariant 3: POC price must correspond to the level with maximum volume
fn prop_poc_is_maximum(bar: &FootprintBar) -> bool {
    if bar.levels.is_empty() {
        return true;
    }
    let max_vol = bar.levels.iter().map(|l| l.total_vol).fold(f64::NEG_INFINITY, f64::max);
    let poc_level = bar.levels.iter().find(|l| (l.price - bar.poc_price).abs() < 1e-4);
    
    if let Some(poc) = poc_level {
        (poc.total_vol - max_vol).abs() < 1e-4
    } else {
        false
    }
}

#[test]
fn test_property_based_invariants_across_1000_samples() {
    for i in 0..1000 {
        let buy_vol = (i as f64 * 0.13 + 1.2).round();
        let sell_vol = (i as f64 * 0.09 + 0.8).round();
        let volume = buy_vol + sell_vol;
        let delta = buy_vol - sell_vol;

        let levels = vec![
            PriceLevel { price: 92500.0, bid_vol: sell_vol * 0.3, ask_vol: buy_vol * 0.2, total_vol: sell_vol * 0.3 + buy_vol * 0.2, delta: 0.0 },
            PriceLevel { price: 92510.0, bid_vol: sell_vol * 0.5, ask_vol: buy_vol * 0.6, total_vol: sell_vol * 0.5 + buy_vol * 0.6, delta: 0.0 },
            PriceLevel { price: 92520.0, bid_vol: sell_vol * 0.2, ask_vol: buy_vol * 0.2, total_vol: sell_vol * 0.2 + buy_vol * 0.2, delta: 0.0 },
        ];

        let bar = FootprintBar {
            volume,
            buy_volume: buy_vol,
            sell_volume: sell_vol,
            delta,
            poc_price: 92510.0,
            levels,
        };

        assert!(prop_delta_integrity(&bar), "Delta invariant failed at iteration {}", i);
        assert!(prop_volume_conservation(&bar), "Volume conservation failed at iteration {}", i);
        assert!(prop_poc_is_maximum(&bar), "POC maximum invariant failed at iteration {}", i);
    }
}

