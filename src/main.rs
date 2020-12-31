#![feature(allocator_api)]

use rand::prelude::SliceRandom;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::TextureQuery;
use std::alloc::Global;
use std::path::Path;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 960;
const ZOOM_FACT: f64 = 20.0;

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

    let mut last_event = None;
    let mut zoomx = ZOOM_FACT;
    let mut zoomy = ZOOM_FACT;
    let OHLC {
        high: all_high,
        low: all_low,
        ..
    } = all_prices.clone().unwrap();

    let mut cx: f64 = prices.len() as f64 / 2.0;
    let mut cy: f64 = (all_high + all_low) / 2.0;

    let mut dx: i32 = 0;
    let mut dy: i32 = 0;

    #[derive(Debug)]
    enum MouseState {
        Up,
        Zooming {
            x: i32,
            y: i32,
            initial_zoom_x: f64,
            initial_zoom_y: f64,
        },
        Panning {
            x: i32,
            y: i32,
        },
    }

    impl MouseState {
        fn is_zoom(&self) -> bool {
            match self {
                MouseState::Zooming { .. } => true,
                _ => false,
            }
        }

        fn is_pan(&self) -> bool {
            match self {
                MouseState::Panning { .. } => true,
                _ => false,
            }
        }
    }

    let mut mouse_state = MouseState::Up;

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'mainloop,
                Event::MouseMotion { x, y, .. } => match mouse_state {
                    MouseState::Zooming {
                        x: cx,
                        y: cy,
                        initial_zoom_x,
                        initial_zoom_y,
                    } => {
                        zoomx = initial_zoom_x + 10.0 * (x as f64 - cx as f64) / WIDTH as f64;
                        zoomy = initial_zoom_y + 10.0 * (y as f64 - cy as f64) / HEIGHT as f64;
                    }
                    MouseState::Panning { x: px, y: py } => {
                        dx = x - px;
                        dy = y - py;
                    }
                    MouseState::Up => {}
                },
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } if mouse_state.is_pan() => mouse_state = MouseState::Up,
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Right,
                    ..
                } if mouse_state.is_zoom() => mouse_state = MouseState::Up,
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Right,
                    x,
                    y,
                    ..
                } => {
                    mouse_state = MouseState::Zooming {
                        x,
                        y,
                        initial_zoom_x: zoomx,
                        initial_zoom_y: zoomy,
                    }
                }
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    mouse_state = MouseState::Panning {
                        x: x - dx,
                        y: y - dy,
                    }
                }
                _ => {
                    //    println!("event is {:?}", event);
                }
            }
            last_event = Some(event)
        }

        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        canvas.clear();

        let white = Color::RGBA(255, 255, 255, 255);
        let red = Color::RGBA(200, 0, 0, 255);
        let green = Color::RGBA(0, 150, 0, 255);

        let half_tickwidth = {
            let ret = (zoomx * 0.45 * WIDTH as f64) / prices.len() as f64;
            ret as u32
        };

        //let yrange = all_high - all_low;

        let font =
            ttf_context.load_font(Path::new("/usr/share/fonts/JetBrainsMono-Medium.ttf"), 12)?;
        let texture_creator = canvas.texture_creator();

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
                let scale_x = |x: i32| {
                    //let cx = prices.len() as f64 / 2.0;
                    let rx = prices.len() as f64;
                    let ox = (x as f64 - cx) / rx;
                    let fw = WIDTH as f64;
                    let ret = dx + ((fw / 2.0) + fw * ox * zoomx) as i32;
                    //return (f * WIDTH as f64) as i32;
                    return ret;
                };

                let scale_y = |px: f64| {
                    //let range = all_high - all_how;
                    let ry = all_high - all_low;
                    //let cy = (all_high + all_low) / 2.0;
                    let oy = (px - cy) / ry;
                    //let f = (all_high - px) / yrange;
                    let ih = HEIGHT as f64;
                    let ret = dy + ((ih / 2.0) + ih * oy * zoomy) as i32;
                    //println!("scale {} -> {} oy={} cy={}", px, ret, oy, cy);
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

        let mut draw_string = |x, y, s: &String| -> Result<(), String> {
            let surface = font
                .render(s)
                .blended_wrapped(Color::RGBA(200, 0, 255, 255), 200)
                .map_err(|e| e.to_string())
                .unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())
                .unwrap();
            let TextureQuery { width, height, .. } = texture.query();
            canvas
                .copy(&texture, None, Some(rect!(x, y, width, height)))
                .unwrap();
            Ok(())
        };

        let s1 = &std::format!("ah={} al={}", all_high, all_low);
        //let s2 = &std::format!("yrange={}", yrange);
        draw_string(20, 20, s1)?;
        //draw_string(20, 40, s2)?;

        if let Some(event) = &last_event {
            //println!("Last event is {:#?}", event);
            let s3 = &std::format!("evt is {:#?}", *event);
            draw_string(20, 60, s3)?;
        }
        //let s4 =
        let s5 = &std::format!("zoomx={} zoomy={}", zoomx, zoomy);
        draw_string(500, 20, s5)?;

        let s6 = &std::format!("mouse_state={:?}", mouse_state);
        draw_string(500, 40, s6)?;

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
}
