use core::time;
use std::thread;

mod cpu;
mod font;

fn main() {
    let mut cpu = cpu::CPU::new();

    let refresh_rate = time::Duration::from_millis(1000 / 700);

    loop {
        if !(true) {
            break;
        }

        cpu.execute_tick().expect("error during tick");
        thread::sleep(refresh_rate)
    }
}
