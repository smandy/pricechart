#![feature(allocator_api)]

pub const WIDTH: u32 = 960;
pub const HEIGHT: u32 = 960;
pub const ZOOM_FACT: f64 = 20.0;
use rand::prelude::SliceRandom;

pub fn max(x: f64, y: f64) -> f64 {
    if x > y {
        x
    } else {
        y
    }
}

#[derive(Debug, Clone)]
pub struct OHLC {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

trait RecvPrice<T> {
    fn apply(&mut self, px: f64);
}

impl RecvPrice<f64> for Option<OHLC> {
    fn apply(&mut self, px: f64) {
        match self {
            None => {
                *self = Some(OHLC {
                    open: px,
                    high: px,
                    low: px,
                    close: px,
                })
            }
            Some(x) => {
                if px > x.high {
                    x.high = px
                }
                if px < x.low {
                    x.low = px
                }
                x.close = px;
            }
        }
    }
}

pub fn make_prices() -> (Option<OHLC>, Vec<(i32, Option<OHLC>)>) {
    let mut price = 100.0;
    let mut rng = rand::thread_rng();
    let mut prices = Vec::new();

    let mut ohlc: Option<OHLC> = None;
    let mut all = None;
    println!("Ohlc is {:?}", ohlc);
    let mut bar = 0;

    let ticks = &[-0.01, -0.01, 0.0, 0.0, 0.0, 0.01, 0.01];

    let mut tix = 0;
    loop {
        tix += 1;
        if tix > 500 {
            ohlc.apply(price);
        }
        all.apply(price);
        let new_p = price + ticks.choose(&mut rng).unwrap();
        price = max(10.0, new_p);
        if tix == 10000 {
            tix = 0;
            bar += 1;
            prices.push((bar, ohlc));
            ohlc = None;
            if prices.len() == 200 {
                return (all, prices);
            }
        }
    }
}
