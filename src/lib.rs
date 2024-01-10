#![feature(allocator_api)]

pub mod lib {

    const WIDTH: u32 = 960;
    const HEIGHT: u32 = 960;
    const ZOOM_FACT: f64 = 20.0;
    use rand::prelude::SliceRandom;

    macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
    );

    fn max(x: f64, y: f64) -> f64 {
        if x > y {
            x
        } else {
            y
        }
    }

    #[derive(Debug, Clone)]
    struct OHLC {
        open: f64,
        high: f64,
        low: f64,
        close: f64,
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

    fn make_prices() -> (Option<OHLC>, Vec<(i32, Option<OHLC>)>) {
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
}
