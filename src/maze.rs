const WIDTH: u8 = 32;
const HEIGHT: u8 = 16;

extern crate rand;
use rand::{Rng, SeedableRng};
use cortex_m::asm::nop;

#[derive(Copy, Clone)]
pub struct Point {
	pub x: u8,
	pub y: u8,
}
impl Point {
	fn dir(&self, direction: Direction) -> Point {
		match direction {
			Direction::Right => self.right(),
			Direction::Left => self.left(),
			Direction::Top => self.top(),
			Direction::Bottom => self.bottom(),
		}
	}
	fn right(&self) -> Point {
		Point{ x: self.x+1, y: self.y }
	}
	fn left(&self) -> Point {
		Point{ x: self.x-1, y: self.y }
	}
	fn top(&self) -> Point {
		Point{ x: self.x, y: self.y-1 }
	}
	fn bottom(&self) -> Point {
		Point{ x: self.x, y: self.y+1 }
	}
}

pub struct BitMap {
	content: [[u8; (WIDTH/8) as usize]; HEIGHT as usize]
}
impl BitMap {
	pub(crate) fn new(value: bool) -> BitMap {
		let value = if value {core::u8::MAX} else {0};
		let maze = BitMap {
			content: [[value; (WIDTH/8) as usize]; HEIGHT as usize]
		};
		maze
	}
	pub(crate) fn get(&self, location: Point) -> bool {
		let actual_index = location.x >> 3;
		let index_within_byte = location.x & 0b111;
		let byte = self.content[location.y as usize][actual_index as usize];
		return (byte >> index_within_byte) & 0b1 == 0b1
	}
	fn set(&mut self, location: Point, value: bool) {
		let actual_index = location.x >> 3;
		let index_within_byte = location.x & 0b111;
		let val = 1 << index_within_byte;
		let byte = &mut self.content[location.y as usize][actual_index as usize];
		if value {
			*byte |= val;
		} else {
			*byte &= !val;
		}
	}
	pub fn iter(&self) -> BitMapIterator {
		return BitMapIterator{ map: self, row: 0 }
	}
	pub fn row_iter(&self, row: u8) -> BitMapRowIterator {
		BitMapRowIterator {
			data: &self.content[row as usize],
			col: 0,
			buf: 0,
			counter: 0
		}
	}
}

pub struct BitMapIterator<'a> {
	map: &'a BitMap,
	row: u8,
}
pub struct BitMapRowIterator<'a> {
	data: &'a [u8; (WIDTH/8) as usize],
	col: u8,
	buf: u8,
	counter: u8,
}
impl<'a> Iterator for  BitMapIterator<'a>   {
	type Item = BitMapRowIterator<'a>;
	fn next(&mut self) -> Option<Self::Item> {
		self.row += 1;
		if self.row == HEIGHT {
			return None;
		} else {
			return Some(BitMapRowIterator {
				data: &self.map.content[self.row as usize],
				col: 0,
				buf: 0,
				counter: 0
			});
		}
	}
}

impl<'a> Iterator for  BitMapRowIterator<'a>   {
	type Item = bool;
	fn next(&mut self) -> Option<Self::Item> {
		if self.col == WIDTH/8 {
			return None;
		}
		if self.counter == 0 {
			self.counter = 8;
			self.buf = self.data[self.col as usize];
			self.col += 1;
		}
		self.counter -= 1;
		let digit: bool = self.buf & 0b1 == 1;
		self.buf = self.buf >> 1;
		Some(digit)
	}
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Direction {
	Left = 0, Right, Top, Bottom
}

pub struct Maze {
	pub bitmap_left: BitMap,
	pub bitmap_top: BitMap,
	pub start: Point,
	pub end: Point,
}
impl Maze {
	pub fn new() -> Maze {
		Maze {
			bitmap_top: BitMap::new(true),
			bitmap_left: BitMap::new(true),
			start: Point { x: 0, y: 0 },
			end: Point { x: 0, y: 0 },
		}
	}
	pub fn break_wall(&mut self, location: Point, dir: Direction) {
		match dir {
			Direction::Top => self.bitmap_top.set(location, false),
			Direction::Left => self.bitmap_left.set(location, false),
			Direction::Bottom => self.bitmap_top.set(location.bottom(), false),
			Direction::Right => self.bitmap_left.set(location.right(), false)
		}
	}
	pub fn grid_iter<'a>(&'a self) -> impl Iterator<Item = impl Iterator<Item = (bool, bool)> + 'a> + 'a {
		self.bitmap_left.iter().zip(self.bitmap_top.iter())
			.map(|(left, top)| left.zip(top))
	}
	pub fn row_iter(&self) -> impl Iterator<Item = (BitMapRowIterator, BitMapRowIterator)> {
		self.bitmap_left.iter().zip(self.bitmap_top.iter())
	}
}


pub struct MazeGenerator {
	state: [[u8; (WIDTH) as usize]; HEIGHT as usize],
	visited: BitMap,
	rng: rand::rngs::SmallRng
}
impl MazeGenerator {
	pub fn new() -> MazeGenerator {
		let seed: [u8; 16] = [0,12,0,25,0,0,0,0,0,0,0,1,0,0,0,15];
		let mut rng = rand::rngs::SmallRng::from_seed(seed);

		MazeGenerator {
			state: [[0; WIDTH as usize]; HEIGHT as usize],
			visited: BitMap::new(false),
			rng
		}
	}

	// Maze gen algorithm
	// States: for each cell, there're 8 bits available.
	// First four bits: incoming edge direction. Up, Down, Left, Right. 0b0000 means this is starting cell
	// Last four bits: outgoing edge direction. Same definition.

	// When no more ways to go, backtrack by going to the grid incoming edge is pointing to.
	fn bin_to_dir(bindir: u8) -> Direction {
		match bindir {
			0b1000 => Direction::Top,
			0b0100 => Direction::Bottom,
			0b0010 => Direction::Left,
			0b0001 | _ => Direction::Right,
		}
	}

	fn bin_dir_opposite(bindir: u8) -> u8 {
		if bindir & 0b1010 == 0 {
			// up or left
			bindir << 1
		} else {
			bindir >> 1
		}
	}

	pub fn generate(&mut self, maze: &mut Maze) {
		let x: u8 = self.rng.gen();
		let y: u8 = self.rng.gen();
		maze.start = Point { x: x >> 3, y: y >> 4 };
		let mut current = maze.start;
		loop {
			let current_state: &mut u8 = &mut self.state[current.y as usize][current.x as usize];
			let incoming_edges = *current_state >> 4; // first four bits
			let outgoing_edges = *current_state & 0b1111; // last four bits

			let mut available_edges_to_go = !(incoming_edges | outgoing_edges) & 0b1111;

			if current.x == 0 {
				available_edges_to_go &= !0b0010; // dont go left
			} else if current.x == WIDTH-1 {
				available_edges_to_go &= !0b0001; // dont go right
			}
			if current.y == 0 {
				available_edges_to_go &= !0b1000; // dont go up
			} else if current.y == HEIGHT - 1 {
				available_edges_to_go &= !0b0100; // dont go down
			}

			for i in 0 .. 4 {
				let dir: u8 = 1 << i;
				if available_edges_to_go & dir == 0 {
					// This direction was already blocked
					continue;
				}
				let point = current.dir(MazeGenerator::bin_to_dir(dir));
				if self.visited.get(point) {
					// The block on this direction was already visited. Skip it.
					available_edges_to_go &= !dir;
				}
			}

			if available_edges_to_go == 0 {
				// backtrace
				self.visited.set(current, true);
				if incoming_edges == 0 {
					// incoming edges equals 0, meaning this is the starting location
					return;
				}
				current = current.dir(MazeGenerator::bin_to_dir(incoming_edges));
				continue;
			}

			// determine the direction to go
			let mut dir_to_go: u8 = 0;
			while dir_to_go == 0 {
				let rand_num: u8 = self.rng.gen();
				let rand_num: u8 = 1_u8 << (rand_num & 0b11_u8);
				dir_to_go = available_edges_to_go & rand_num;
			}
			let direction = MazeGenerator::bin_to_dir(dir_to_go);
			maze.break_wall(current, direction);
			self.visited.set(current, true);

			*current_state |= dir_to_go;
			current = current.dir(direction);

			let opposite_dir = MazeGenerator::bin_dir_opposite(dir_to_go);
			self.state[current.y as usize][current.x as usize] |= opposite_dir << 4; // set the incoming edges
		}
	}
}
