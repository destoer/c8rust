extern crate chip8;
extern crate sdl2;
use std::env;
use std::process;
use std::{thread, time};
use chip8::{Cpu};
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use sdl2::rect::Rect;

const SCALE_FACTOR: u32 = 20;
const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;


// https://github.com/Rust-SDL2/rust-sdl2
// https://github.com/arnsa/Chip-8-Emulator/blob/master/chip8.c
// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.4
// https://rust-sdl2.github.io/rust-sdl2/sdl2/
// https://github.com/starrhorne/chip8-rust/blob/master/src/drivers/input_driver.rs
fn main() 
{ 
	// get program args and load in the rom
	let args: Vec<String> = env::args().collect(); // collect args
	
	// not enough args
	if args.len() < 2
	{
		println!("usage: {} <rom to open>",args[0]);
		process::exit(1);
	}

	// input the cpu state
	let mut cpu = Cpu::new(); 

	// read the rom into main memory starting at 0x200
	// and copy the font into main memory while we are at it
	cpu.load_rom(&args[1]);

	
	// init sdl 
	let sdl = sdl2::init().unwrap();
	let video_subsystem = sdl.video().unwrap();
	let window = video_subsystem
		.window("chip8",WIDTH*SCALE_FACTOR,HEIGHT*SCALE_FACTOR)
		.position_centered()
		.opengl()
		.build()
		.unwrap();

	let mut event_pump = sdl.event_pump().unwrap();
	
	// more window stuff...
	let mut canvas = window.into_canvas().build().unwrap();
	canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
	canvas.clear();
	canvas.present();
	
	// begin the emulation loop
	'main: loop
	{
		// update the key state
		for event in event_pump.poll_iter()
		{
			match event
			{
				sdl2::event::Event::Quit {..} => break 'main,
				
				sdl2::event::Event::KeyDown { keycode: Some(keycode), .. } =>
				{
					//println!("{}", keycode);
					
					match keycode
					{
						Keycode::Num1 => cpu.keys[0x1] = true,
						Keycode::Num2 => cpu.keys[0x2] = true,
						Keycode::Num3 => cpu.keys[0x3] = true,
						Keycode::Num4 => cpu.keys[0xc] = true,
						Keycode::Q => cpu.keys[0x4] = true,
						Keycode::W => cpu.keys[0x5] = true,
						Keycode::E => cpu.keys[0x6] = true,
						Keycode::R => cpu.keys[0xd] = true,
						Keycode::A => cpu.keys[0x7] = true,
						Keycode::S => cpu.keys[0x8] = true,
						Keycode::D => cpu.keys[0x9] = true,
						Keycode::F => cpu.keys[0xe] = true,
						Keycode::Z => cpu.keys[0xa] = true,
						Keycode::C => cpu.keys[0x0] = true,
						Keycode::V => cpu.keys[0xf] = true,
						
						_ => ()
					}
					
				},
				
				// dont know if required
				sdl2::event::Event::KeyUp { keycode: Some(keycode), .. } =>
				{
					//println!("{}", keycode);
					
					match keycode
					{
						Keycode::Num1 => cpu.keys[0x1] = false,
						Keycode::Num2 => cpu.keys[0x2] = false,
						Keycode::Num3 => cpu.keys[0x3] = false,
						Keycode::Num4 => cpu.keys[0xc] = false,
						Keycode::Q => cpu.keys[0x4] = false,
						Keycode::W => cpu.keys[0x5] = false,
						Keycode::E => cpu.keys[0x6] = false,
						Keycode::R => cpu.keys[0xd] = false,
						Keycode::A => cpu.keys[0x7] = false,
						Keycode::S => cpu.keys[0x8] = false,
						Keycode::D => cpu.keys[0x9] = false,
						Keycode::F => cpu.keys[0xe] = false,
						Keycode::Z => cpu.keys[0xa] = false,
						Keycode::C => cpu.keys[0x0] = false,
						Keycode::V => cpu.keys[0xf] = false,						
						_ => ()
					}
					
				},
				
				_=> {},
			}
		}
		
		// execute a cpu cycle
		cpu.step();
		cpu.timers();
		
		
		// draw the window 
		for y in 0..HEIGHT
		{
			for x in 0..WIDTH
			{
				canvas.set_draw_color( color( cpu.graphics[ ((y*WIDTH) + x) as usize]) );
				//canvas.set_draw_color(pixels::Color::RGB(255,255,255));
				let _ = canvas
					.fill_rect(Rect::new((x * SCALE_FACTOR)as i32, (y * SCALE_FACTOR) as i32,SCALE_FACTOR, SCALE_FACTOR));
			}
		}
		canvas.present();
		thread::sleep(time::Duration::from_millis(1)); // add emulation delay
														// for playabilitly 
	}
}



fn color(value: u8) -> pixels::Color {
    if value == 0 {
        pixels::Color::RGB(0, 0, 0)
    } else {
        pixels::Color::RGB(255, 255, 255)
    }
}

