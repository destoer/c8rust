extern crate rand;

use std::fmt;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::process;
//use std::io::Seek;
//use std::io::SeekFrom;

// LEARN SDL AND IMPLEMENT DISPLAY AND KEYMAP
// AND WE ARE GOOD TO GO
// FINISH THE STUB INSTRUCTIONS
/* implement the stub instructions and
		cpu.timers(); // update the timers <-- do next as its easy then do the instructions(leave keys) and finally the sdl stuff
		cpu.draw(); // update the screen
		cpu.keys(); // get the keystate <--- may not even need? just do it in the instruciton loop
	functions and we are done */

// may need to enable overflows on addition and subtraction instructions
// with wrapping_sub and wrapping_add


// refactoring
// pull out all fields at start to improve code cleanliness
// change sp and pc to usize to avoid lots of casting
// add display and keyboard as part of the cpu
// find a better way to do the initial memory copys
// do for loops using iter() instead of manually



// Chip 8 fontset
static FONTSET: [u8; 80] = [
	0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
	0x20, 0x60, 0x20, 0x20, 0x70, // 1
	0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
	0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
	0x90, 0x90, 0xF0, 0x10, 0x10, // 4
	0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
	0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
	0xF0, 0x10, 0x20, 0x40, 0x40, // 7
	0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
	0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
	0xF0, 0x90, 0xF0, 0x90, 0x90, // A
	0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
	0xF0, 0x80, 0x80, 0x80, 0xF0, // C
	0xE0, 0x90, 0x90, 0x90, 0xE0, // D
	0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
	0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];


const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

pub struct Cpu
{
	// registers
	opcode: u16,
	v: [u8; 16], // main registers
	i: u16,
	sp: u8,
	stack: [u16; 16], // stack memory 
	pub graphics: [u8; 64 * 32], // graphics memory 64 wide 32 height
	pc: u16,
	delay: u8, // delay timer
	sound: u8, // sound timer
	pub keys: [bool; 16], // stores the keypress state
	mem: [u8; 4096] // main memory
}

impl Cpu
{
	pub fn new() -> Cpu
	{
		Cpu {
			opcode: 0,
			v: [0;16],
			i: 0x200, // chip 8 reset vector
			delay: 0,
			sound: 0,
			pc: 0x200, // same
			sp: 0,
			stack: [0;16],
			mem: [0; 4096],
			graphics: [0; 64 * 32],
			keys: [false; 16]
		}
	}
	
	pub fn timers(&mut self)
	{
		if self.delay > 0
		{
			self.delay -= 1;
		}
		
		// think supposed to produce beeps for sound timer here
		// but its not a big deal yet
		if self.sound > 0
		{
			self.sound -= 1;
		}	
	}
		
	pub fn load_rom(&mut self, filename: &String)
	{
		// open the file
		let fp = match File::open(&filename) { // open the file
			Err(why) => { 
				eprintln!("couldn't open file {}: {}",filename,
					why.description());
				process::exit(1)  
			}, 	Ok(fp) => fp,
		};

		// read the rom out
		// is there memcpy function for this?
		for(i, byte) in fp.bytes().enumerate()
		{
			self.mem[i+0x200] = byte.unwrap();
		}
		
		
		// set the first 80 bytes to the fontset
		for(i, &byte) in FONTSET.iter().enumerate()
		{
			self.mem[i] = byte;
		}
		
	}
	
	// emulate a single cycle
	pub fn step(&mut self)
	{
		// read in an opcode
		self.opcode = u16_from_u8(&self.mem, self.pc);
		
		let op = (self.opcode & 0xf000) >> 12; // get top nibble
		
		//println!("{:x} :opcode 0x{:x} : op 0x{:x}", self.pc,self.opcode,op);
		
		// each instruction is a u16 goto next instruction
		self.pc += 2;
		
		match op 
		{
			// CLS OR RET (both are implicit and take no args)
			0x0 =>
			{
				// CLS ( clear the screen )
				if self.opcode == 0x00E0
				{
					self.graphics = [0; (WIDTH * HEIGHT) as usize]; // memset the screen buffer
				}
				
				// RET (not sure if proper way to read off the stack)
				else if self.opcode == 0x00EE
				{
					self.pc = self.stack[(self.sp & 0xF) as usize]; self.sp -= 1;
				}
				
				// anything else is an invalid opcode
				else
				{
					self.invalid_opcode()
				}
				
			}
			
			// JP addr
			0x1 =>
			{
				// set pc to lower 12 bits of opcode
				self.pc = self.opcode & 0x0fff;
			
			}
			
			// CALL addr
			0x2 =>
			{
				self.sp += 1;
				self.stack[self.sp as usize] = self.pc;
				self.pc = self.opcode & 0x0fff;			
			}

			// SE Vx, byte (3xkk)
			0x3 =>
			{
				// skip next instruction if vx == kk
				if self.v[( (self.opcode & 0x0F00) >> 8 ) as usize] == (self.opcode & 0x00ff) as u8
				{
					self.pc += 2;
				} 
			}
			
			// SNE Vx, byte (3xkk)
			0x4 =>
			{
				// skip next instruction if vx != kk
				if self.v[( (self.opcode & 0x0F00) >> 8 ) as usize] != (self.opcode & 0x00ff) as u8
				{
					self.pc += 2;
				} 
			}			
			
			// SE Vx, Vy (5xy0)
			0x5 =>
			{
				// if vx = vy skip current instruction
				if self.v[ ((self.opcode & 0x0F00) >> 8) as usize] == self.v[ ((self.opcode & 0x00F0) >> 4) as usize]
				{
					self.pc += 2;
				}
			}
			
			// LD vx, byte (6xkk) 
			0x6 =>
			{
				// vx = kk
				self.v[ ((self.opcode & 0x0f00) >> 8) as usize] = (self.opcode & 0x00ff) as u8
			}
			
			// add Vx, byte (7xkk)
			0x7 =>
			{
				// vx += kk
				let x = ((self.opcode & 0xf00) >> 8) as usize;
				//self.v[ ((self.opcode & 0xf00) >> 8) as usize] += (self.opcode & 0x00ff) as u8;
				self.v[x] = self.v[x].wrapping_add((self.opcode & 0x00ff) as u8);
			}
			
			// bitshifts logical operators and subtraction
			0x8 =>
			{
				// get new opcode
				let op = self.opcode & 0x000f;
				
				match op
				{
					// LD vx, vy (8xy0)
					0x0 =>
					{
						// vx = vy
						self.v[((self.opcode & 0x0f00) >> 8) as usize] = self.v[((self.opcode & 0x00f0) >> 4) as usize];
					}
					
					// OR vx, vy (8xy1)
					0x1 =>
					{
						// vx |= vy
						self.v[((self.opcode & 0x0f00) >> 8) as usize] |= self.v[((self.opcode & 0x00f0) >> 4) as usize];
					}
					
					// AND vx, vy (8xy2)
					0x2 =>
					{
						self.v[((self.opcode & 0x0f00) >> 8) as usize] &= self.v[((self.opcode & 0x00f0) >> 4) as usize];
					}
				
					// XOR vx, vy (8xy3)
					0x3 =>
					{
						self.v[((self.opcode & 0x0f00) >> 8) as usize] ^= self.v[((self.opcode & 0x00f0) >> 4) as usize];
					}
					
					// ADD vx, vy (8xy4) <-- check for error
					0x4 =>
					{
						let ans: usize = self.v[((self.opcode & 0x0f00) >> 8) as usize] as usize + self.v[((self.opcode & 0x00f0) >> 4) as usize] as usize;
						// if there is a carry (result of addition > 255)
						if ans > 255 {
							self.v[15] = 1;

						} else {
							self.v[15] = 0;
						}
						self.v[((self.opcode & 0x0f00) >> 8) as usize] = self.v[((self.opcode & 0x0f00) >> 8) as usize]
							.wrapping_add(self.v[((self.opcode & 0x00f0) >> 4) as usize]);						
					}
					
					// SUB Vx, Vy
					0x5 =>
					{
						//r.y = r.y.wrapping_sub(1);
						let x: usize = ((self.opcode & 0x0f00) >> 8) as usize;
						let y: usize = ((self.opcode & 0x00f0) >> 4) as usize;
						
						if self.v[x] > self.v[y]
						{
							self.v[15] = 1;
						} else {
							self.v[15] = 0;
						}
						
						// dont know if subtraction should underflow
						self.v[x] = self.v[x].wrapping_sub(self.v[y]);
					}
					
					// SHR VX {, VY}
					0x6 =>
					{
						// v[f] =  lsb of vx
						let x = ((self.opcode & 0x0f00) >> 8) as usize;
						self.v[15] =  self.v[x] & 1;
						
						self.v[x] >>= 1;
					}
					
					// SUBN vx, vy
					0x7 =>
					{
						let x: usize = ((self.opcode & 0x0f00) >> 8) as usize;
						let y: usize = ((self.opcode & 0x00f0) >> 4) as usize;					
						
						if self.v[y] > self.v[x]
						{
							self.v[15] = 1;
						} else {
							self.v[15] = 0;
						}
						
						self.v[x] = self.v[y].wrapping_sub(self.v[x]);
						
					}
					
					// SHL VX {, VY}
					0xE =>
					{
						let x = ((self.opcode & 0x0f00) >> 8) as usize;
						self.v[15] =  (self.v[x] & 128) >> 7;
						self.v[x] <<= 1;
					}
					
					// invalid opcode
					_=>
					{
						self.invalid_opcode();
					}
				}
			
			}
			
			// SNE vx, vy
			0x9 =>
			{
				// 0 as lowest nibble only valid encoding
				if self.opcode & 0x000f == 0
				{
					// if vx != vy skip current instruction
					if self.v[ ((self.opcode & 0x0f00) >> 8) as usize] != self.v[ ((self.opcode & 0x00f0) >> 4) as usize]
					{
						self.pc += 2;
					}
				} else {
					self.invalid_opcode();
				}	
			}
			
			// LD I, addr
			0xA =>
			{
				// i = nnn
				self.i = self.opcode & 0x0fff;
			}
			
			// JP V0 , addr ( bnnn )
			0xB =>
			{
				// jump to v0 + nnn
				self.pc = (self.v[0]) as u16 + (self.opcode & 0x0fff);
			}

			// RND Vx, byte
			0xC =>
			{
				// x = random byte & kk
				let num: u8 =  rand::random();
				self.v[((self.opcode & 0x0f00) >> 8) as usize] = num & (self.opcode & 0x00ff) as u8;
			}
			
			// DRW vxm vy, nibble
			// stub for now
			0xD => // <--- needs to be verified
			{
				// display n-byte sprite starting at memory location I at (vx,vy) set vf = collision
				let height = self.opcode & 0x000f;
				self.v[0xf] = 0;
				
				// draw the sprite // <--- needs verification
				
				// fix all the casting
				let x = ((self.opcode &0x0f00) >> 8) as u16;
				let y = ((self.opcode &0x00f0) >> 4) as u16;
				for byte in 0..height 
				{
					let y = (self.v[y as usize] as usize + byte as usize) % HEIGHT as usize;
					for bit in 0..8 
					{
						let x = (self.v[x as usize] as usize + bit) % WIDTH as usize;
						let color = (self.mem[(self.i as usize + byte as usize) as usize] >> (7 - bit)) & 1;
						self.v[0xf] |= color & self.graphics[(y*WIDTH as usize) + x];
						self.graphics[(y*WIDTH as usize)+x] ^= color;

					}
				}
				
			}
			
			// SKP vx
			0xE =>
			{
				if self.opcode & 0x00ff == 0x9E
				{
					// skip next instr if key pressed value = vx
					if self.keys[self.v[((self.opcode & 0x0f00) >> 8) as usize] as usize]
					{
						self.pc += 2;
					}
				}

				else if self.opcode & 0x00ff == 0xA1
				{
					// skip next instruction if key press value != vx
					if !(self.keys[self.v[((self.opcode & 0x0f00) >> 8) as usize] as usize])
					{
						self.pc += 2;
					}					
				}
				
				else 
				{
					self.invalid_opcode();
				}
			}
			
			// operations involving I and tiemr
			0xF =>
			{
				let op = self.opcode & 0x00ff;
				
				match op
				{
					// LD vx, DT (Fx07)
					0x07 =>
					{
						// vx = delay timer
						self.v[((self.opcode & 0x0f00) >> 8) as usize] = self.delay;
					
					}
					
					// LD vx, K
					0x0A => // <--- verify
					{
						// wait for a key press, store the value of the key in vx
						for x in 0..16
						{
							if self.keys[x]
							{
								self.v[((self.opcode & 0x0f00) >> 8) as usize] = x as u8;
							}
						}
					}
					
					// LD DT, vx
					0x15 =>
					{
						// delay timer = vx
						self.delay = self.v[((self.opcode & 0x0f00) >> 8) as usize];
					}
					
					//  LD ST, vx
					0x18 =>
					{
						// sound timer = vx
						self.sound = self.v[((self.opcode & 0x0f00) >> 8) as usize];
					}
					
					// ADD I, Vx
					0x1E =>
					{
						// I += vx
						self.i += self.v[((self.opcode & 0x0f00) >> 8) as usize] as u16;
					}
					
					// LD F, Vx
					0x29 =>
					{
						// I = location of sprite for digit Vx
						self.i = (self.v[((self.opcode & 0x0f00) >> 8) as usize] * 5) as u16;
					}
					
					// LD B, vx
					0x33 =>  // <---- needs verifying
					{
						// store bcd represenation of vx in I, I+1, and I+2
						let vx = self.v[((self.opcode & 0x0f00) >> 8) as usize];
						
						self.mem[self.i as usize] = vx / 100; // hundreds digit
						self.mem[(self.i+1) as usize] = (vx / 10) % 10;  // tens digit
						self.mem[(self.i+2) as usize] = vx % 10; // units digit
					}
					
					
					// LD [I], VX
					0x55 =>
					{
						let x = (self.opcode & 0x0f00) >> 8;
						// store V0 trhough vx in memory starting at I
						for j in 0..x // <-- verify
						{
							self.mem[(self.i + j) as usize] = self.v[j as usize];
						}
					}
					
					// LD Vx, [I]
					0x65 =>
					{
						// read registers v0 to vx from I
						let x = (self.opcode & 0x0f00) >> 8;
						for j in 0..x // <--- verify
						{
							self.v[j as usize] = self.mem[(self.i + j) as usize];
						}
					}
					
					_=>
					{
						self.invalid_opcode();
					}
				
				
				}
			
			
			}

			
			// unknown opcode
			_ =>
			{
				println!("Unknown opcode, Cpu state {}",self);
				process::exit(1);
			
			}
			

		
		}		
	}
	
	// execution of an invalid opcode
	pub fn invalid_opcode(&mut self)
	{
		println!("Error invalid opcode, Cpu state: {}",self);
		process::exit(1);
	}
	
	
	
}

impl fmt::Display for Cpu
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		// awful code
		write!(f, "V0:{:x}, V1:{:x}, V2:{:x}, V3:{:x}, V4:{:x}, V5:{:x}, V6:{:x}, V7:{:x}
V8:{:x}, V9:{:x}, VA:{:x}, VB:{:x}, VC:{:x}, VD:{:x}, VE:{:x}, VF:{:x}
SP:{:x}, PC:{:x}, DELAY:{:x}, SOUND:{:x}",
			self.v[0],self.v[1],self.v[2],self.v[3],self.v[4],self.v[5],self.v[6],
			self.v[7],self.v[8],self.v[9],self.v[10],self.v[11],self.v[12],self.v[13],self.v[14],self.v[15],
			self.sp, self.pc, self.sound, self.delay)
	}
}




// chip 8 is big endian
pub fn u16_from_u8(buf: &[u8], address: u16) -> u16 { // address is a num cast to usize
    let mut num: u16 = (buf[address as usize] as u16) << 8;
    num += buf[(address+1) as usize] as u16; 
    return num;
}


