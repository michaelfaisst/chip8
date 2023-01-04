use std::fs::read;

use sdl2::{event::Event, render::Canvas, video::Window, pixels::Color, rect::Rect};

mod cpu;
mod font;

const SCALE: u32 = 1;


fn draw_screen(cpu: &cpu::CPU, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let vram = cpu.vram;
    canvas.set_draw_color(Color::RGB(255, 255, 255));

    for y in 0..vram.len() {
        for x in 0..vram[y].len() {
            if vram[y][x] > 0 {
                let rect = Rect::new(((x as u32) * SCALE) as i32, ((y as u32) * SCALE) as i32, SCALE, SCALE);
                canvas.fill_rect(rect).unwrap();
            }
        } 
    }

    canvas.present();
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Chip8 in Rust", 64 * SCALE, 32 * SCALE).position_centered().opengl().build().unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let rom = read("./roms/space_invaders.ch8").unwrap();
    let mut cpu = cpu::CPU::new(&rom);

    'mainloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit{..} => {
                    break 'mainloop;
                },
                Event::KeyDown { keycode: Some(key), .. } => {
                    cpu.key_pressed(key);
                },
                Event::KeyUp { keycode: Some(key), .. } => {
                    cpu.key_released(key);
                }
                _ => ()
            }
        }

        for _ in 0..10 {
            cpu.execute_tick().expect("error during tick");
        }

        cpu.update_timers();
        draw_screen(&cpu, &mut canvas);
    }
}
