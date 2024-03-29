use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::TextureQuery;
use std::path::Path;

use lib::*;

macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    );
    );

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
    let zoomx = 1.0;
    let zoomy = 1.0;
    let OHLC {
        high: all_high,
        low: all_low,
        ..
    } = all_prices.clone().unwrap();

    let height = HEIGHT as f64;
    let width = WIDTH as f64;

    let price_range = all_high - all_low;

    let mut my = height / -price_range;
    // Solve y = mx + c for HEIGHT = (HEIGHT / (all_low - all_high) ) * all_high + c
    let mut cy: i32 = (height * all_high / price_range) as i32;
    let mut cx: i32 = 0;

    let mut mx = width / prices.len() as f64;

    //let mut dx: i32 = 0;
    //let mut dy: i32 = 0;

    #[derive(Debug)]
    enum MouseState {
        Up,
        Zooming {
            anchor_x: f64,
            anchor_y: f64,
            //initial_zoom_x: f64,
            //initial_zoom_y: f64,
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
            let inv_x = |x: i32| -> f64 { (x - cx) as f64 / mx };
            let inv_y = |y: i32| -> f64 { (y - cy) as f64 / my };

            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'mainloop,
                Event::MouseMotion { x, y, .. } => match mouse_state {
                    MouseState::Zooming { anchor_x, anchor_y } => {
                        // Need to transform mx

                        let px = inv_x(x);
                        let py = inv_y(y);

                        let new_mx = ZOOM_FACT * (px - anchor_x) / prices.len() as f64;
                        let new_my = ZOOM_FACT * (py - anchor_y) / price_range;

                        cx = cx + (px * (mx - new_mx)) as i32;
                        cy = cy + (py * (my - new_my)) as i32;

                        mx = new_mx;
                        my = new_my;
                    }
                    MouseState::Panning { x: px, y: py } => {
                        cx = x - px;
                        cy = y - py;
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
                        anchor_x: inv_x(x),
                        anchor_y: inv_y(y),
                    }
                }
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    mouse_state = MouseState::Panning {
                        x: x - cx as i32,
                        y: y - cy as i32,
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
            let ret = mx * 0.45;
            ret as u32
        };

        let font = ttf_context.load_font(
            Path::new("/usr/share/fonts/TTF/JetBrainsMono-Medium.ttf"),
            12,
        )?;
        let texture_creator = canvas.texture_creator();
        let scale_x = |x: i32| -> i32 { (mx * (x as f64)) as i32 + cx };
        let scale_y = |px: f64| -> i32 { (my * px) as i32 + cy };

        // Simple scaling
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
        draw_string(500, 60, s6)?;

        canvas.present();
    }

    Ok(())
}
