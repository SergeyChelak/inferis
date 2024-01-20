use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::WindowCanvas, EventPump,
};

const TARGET_FPS: usize = 60;
const WINDOW_WIDTH: u32 = 1600;
const WINDOW_HEIGHT: u32 = 900;

type RenderJob = dyn Fn(&mut WindowCanvas) -> Result<(), String>;

fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;
    let window = video_subsystem
        .window("Inferis", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|err| err.to_string())?;
    let mut event_pump = sdl.event_pump()?;
    let mut canvas = window
        .into_canvas()
        .accelerated()
        // .target_texture()
        .present_vsync()
        .build()
        .map_err(|err| err.to_string())?;
    let target_frame_duration = (1000 / TARGET_FPS) as u128;
    let mut frames = 0usize;
    let mut time = Instant::now();
    loop {
        let frame_start = Instant::now();
        // handle control events
        if !process_events(&mut event_pump) {
            break;
        }
        // draw scene
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        for job in get_draw_pool() {
            job(&mut canvas)?;
        }
        canvas.present();
        // calculate fps (optionally)
        let elapsed = time.elapsed();
        if elapsed.as_millis() > 1000 {
            time = Instant::now();
            let title = format!("FPS: {frames}");
            canvas
                .window_mut()
                .set_title(&title)
                .map_err(|err| err.to_string())?;
            frames = 0;
        } else {
            frames += 1;
        }
        // sleep the rest of the time
        let suspend_ms = target_frame_duration.saturating_sub(frame_start.elapsed().as_millis());
        if suspend_ms > 0 {
            let duration = Duration::from_millis(suspend_ms as u64);
            ::std::thread::sleep(duration);
        }
    }
    Ok(())
}

fn process_events(event_pump: &mut EventPump) -> bool {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => return false,
            _ => {}
        }
    }
    true
}

fn get_draw_pool() -> Vec<Box<RenderJob>> {
    let mut arr: Vec<Box<RenderJob>> = Vec::new();
    let color = [
        Color::BLUE,
        Color::CYAN,
        Color::GREEN,
        Color::GRAY,
        Color::MAGENTA,
        Color::RED,
        Color::WHITE,
        Color::YELLOW,
    ];
    let mut time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .unwrap()
        .as_millis();
    let mut k = 1;
    for i in 0..1500 {
        let clr = color[i % 8];
        let x = (time & 0b11111111) as i32;
        let y = (time >> 4 & 0b11111111) as i32;
        let w = (time >> 5 & 0b1111111111) as u32; // % WINDOW_WIDTH;
        let h = (time >> 6 & 0b1111111111) as u32; // % WINDOW_HEIGHT;
        let draw = move |c: &mut WindowCanvas| -> Result<(), String> {
            c.set_draw_color(clr);
            let ofs = (i + 1) % WINDOW_WIDTH as usize;
            let rect = Rect::new(x * k + ofs as i32, y, w, h);
            c.fill_rect(rect)?;
            Ok(())
        };
        arr.push(Box::new(draw));
        time += time % (i + 1) as u128;
        k = -k;
    }
    arr
}
