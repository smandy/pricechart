use rand::Rng;

struct OHLC {
    open : f64,
    high : f64,
    low : f64,
    close : f64
}

fn main() {
    let sdl_context = sdl2::init()?;
    let ev = sdl_context.event().unwrap();
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("A Knights Tour", 960, 960)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;

    makePrices();
}

fn makePrices() {
    let mut price = 100.0;
    let mut rng = rand::thread_rng();

    for i in 0..1_000_000 {
        price *= rng.gen_range(0.9, 1.1);
        println!("Price is {}", price);
    }
}
