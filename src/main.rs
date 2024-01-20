use std::time::{Duration, Instant};

use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::WindowCanvas, EventPump,
};

const TARGET_FPS: usize = 60;

fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video_sybsystem = sdl.video()?;
    let window = video_sybsystem
        .window("Inferis", 800, 600)
        .position_centered()
        .build()
        .map_err(|err| err.to_string())?;
    let mut event_pump = sdl.event_pump()?;
    let mut canvas = window
        .into_canvas()
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

fn get_draw_pool() -> Vec<Box<dyn Fn(&mut WindowCanvas) -> Result<(), String>>> {
    let draw1 = |c: &mut WindowCanvas| -> Result<(), String> {
        c.set_draw_color(Color::RED);
        let rect = Rect::new(10, 10, 50, 100);
        c.fill_rect(rect)?;
        Ok(())
    };

    let draw2 = |c: &mut WindowCanvas| -> Result<(), String> {
        c.set_draw_color(Color::BLUE);
        let rect = Rect::new(60, 110, 50, 100);
        c.fill_rect(rect)?;
        Ok(())
    };

    vec![Box::new(draw1), Box::new(draw2)]
}
