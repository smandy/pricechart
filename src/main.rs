#![feature(allocator_api)]

use rand::prelude::SliceRandom;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::TextureQuery;
use std::alloc::Global;
use std::path::Path;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 960;

macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

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

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    //let ev = sdl_context.event().unwrap();
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Price chart", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;
    let (all_prices, prices) = make_prices();

    /*    println!("allprices is {:?}", all_prices);
    println!(
        "prices is {:#?} {:#?}",
        &prices[..5],
        &prices[prices.len() - 5..]
    );*/

    let mut last_event = None;

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'mainloop,
                /*                Event::User { .. } => {
                                    //println!("Woot have a solution {:?}", event);
                                }
                */
                _ => {
                    //    println!("event is {:?}", event);
                }
            }
            last_event = Some(event)
        }

        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        canvas.clear();

        let white = Color::RGBA(255, 255, 255, 255);
        let red = Color::RGBA(255, 0, 0, 255);
        let green = Color::RGBA(0, 255, 0, 255);

        let scale_x = |x: i32| {
            let f = (x as f64) / (prices.len() as f64);
            return (f * WIDTH as f64) as i32;
        };

        let half_tickwidth = {
            let ret = (0.45 * WIDTH as f64) / prices.len() as f64;
            ret as u32
        };

        let OHLC {
            high: all_high,
            low: all_low,
            ..
        } = all_prices.clone().unwrap();

        let yrange = all_high - all_low;

        let font =
            ttf_context.load_font(Path::new("/usr/share/fonts/JetBrainsMono-Medium.ttf"), 12)?;
        let texture_creator = canvas.texture_creator();

        let mut draw_string = |x, y, s| -> Result<(), String> {
            let surface = font
                .render(s)
                .blended(Color::RGBA(0, 255, 255, 255))
                .map_err(|e| e.to_string())
                .unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())
                .unwrap();
            let TextureQuery { width, height, .. } = texture.query();
            //println("Query ")
            canvas
                .copy(&texture, None, Some(rect!(x, y, width, height)))
                .unwrap();
            Ok(())
        };

        let s1 = &std::format!("ah={} al={}", all_high, all_low);
        let s2 = &std::format!("yrange={}", yrange);
        draw_string(20, 20, s1)?;
        draw_string(20, 40, s2)?;

        if let Some(event) = &last_event {
            //println!("Last event is {:#?}", event);
            let s = &std::format!("evt is {:#?}", event);
            draw_string(20, 60, s)?;
        }

        for tmp in &prices {
            if let (
                i,
                Some(OHLC {
                    open,
                    high,
                    low,
                    close,
                }),
            ) = tmp
            {
                let scale_y = |px: f64| {
                    //let range = all_high - all_how;
                    let f = (all_high - px) / yrange;
                    let ret = (f * (HEIGHT as f64)) as i32;
                    //println!("scale {} -> {}", px, ret);
                    return ret;
                    //low + dif * range;
                };
                let x = scale_x(*i);
                let o = scale_y(*open);
                let h = scale_y(*high);
                let l = scale_y(*low);
                let c = scale_y(*close);
                let s = Point::new(x, l);
                let f = Point::new(x, h);
                /*            println!(
                    "s={:?} f={:?} tmp={:?} ah={} al={} yrange={}",
                    s, f, tmp, all_high, all_low, yrange
                );*/

                canvas.set_draw_color(white);
                canvas.draw_line(s, f)?;

                {
                    let rect = Rect::new(
                        x - half_tickwidth as i32,
                        o,
                        2 * half_tickwidth,
                        (c - o).abs() as u32,
                    );
                    canvas.set_draw_color(if close > open { green } else { red });
                    canvas.fill_rect(rect)?;
                    canvas.set_draw_color(white);
                    canvas.draw_rect(rect)?;
                }
            }
        }
        canvas.present();
    }

    Ok(())
}

fn make_prices() -> (Option<OHLC>, Vec<(i32, Option<OHLC>), Global>) {
    let mut price = 100.0;
    let mut rng = rand::thread_rng();
    let mut prices = Vec::new();

    let mut ohlc: Option<OHLC> = None;
    let mut all = None;
    println!("Ohlc is {:?}", ohlc);
    let mut bar = 0;

    let ticks = &[-0.01, -0.01, 0.0, 0.0, 0.0, 0.01, 0.01];

    fn max(x: f64, y: f64) -> f64 {
        if x > y {
            x
        } else {
            y
        }
    }

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
            if prices.len() == 50 {
                return (all, prices);
            }
        }
    }
    //return (all, prices);
}
