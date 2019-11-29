const WIDTH: u8 = 32;
const HEIGHT: u8 = 16;

extern crate rand;
use rand::{Rng, SeedableRng};

#[derive(Copy, Clone)]
pub struct Point {
	x: u8,
	y: u8,
}
impl Point {
	fn dir(&self, direction: Direction) -> Option<Point>{
		match direction {
			Direction::Right => self.right(),
			Direction::Left => self.left(),
			Direction::Top => self.top(),
			Direction::Bottom => self.bottom(),
		}
	}
	fn right(&self) -> Option<Point> {
		if self.x+1 == WIDTH {None} else {Some(Point{ x: self.x+1, y: self.y })}
	}
	fn left(&self) -> Option<Point> {
		if self.x == 0 {None} else {Some(Point{ x: self.x-1, y: self.y })}
	}
	fn top(&self) -> Option<Point> {
		if self.y == 0 {None} else {Some(Point{ x: self.x, y: self.y-1 })}
	}
	fn bottom(&self) -> Option<Point> {
		if self.y+1 == HEIGHT {None} else {Some(Point{ x: self.x, y: self.y+1 })}
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
	fn get(&self, location: Point) -> bool {
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
}
impl Maze {
	pub fn new() -> Maze {
		Maze {
			bitmap_top: BitMap::new(true),
			bitmap_left: BitMap::new(true),
		}
	}
	pub fn break_wall(&mut self, location: Point, dir: Direction) {
		match dir {
			Direction::Top => self.bitmap_top.set(location, false),
			Direction::Left => self.bitmap_left.set(location, false),
			Direction::Bottom => if let Some(point) = location.bottom() { self.bitmap_top.set(point, false)},
			Direction::Right => if let Some(point) = location.right() {self.bitmap_left.set(point, false)}
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
	start: Point,
	end: Point,
	visited: BitMap,
}
impl MazeGenerator {
	pub fn random() -> u8 {
		let seed: [u8; 16] = [0; 16];
		let mut rng = rand::rngs::SmallRng::from_seed(seed);
		rng.gen()
	}
	pub fn new() -> MazeGenerator {
		let x: u8 = MazeGenerator::random();
		let y: u8 = MazeGenerator::random();
		MazeGenerator {
			start: Point { x: x >> 5, y: y >> 5 },
			end: Point{ x: 0, y: 0 },
			visited: BitMap::new(false),
		}
	}

	pub fn generate(&mut self) -> Maze {
		let mut maze = Maze::new();
		self.generate_at_point(&mut maze, self.start);
		maze
	}


	pub fn dummy_generate(&mut self) -> Maze {
		let mut maze = Maze::new();
		for i in 0 .. 10 {
			let x: u8 = MazeGenerator::random();
			let y: u8 = MazeGenerator::random();
			maze.bitmap_left.set(Point { x: x >> 5, y: y >> 5}, false);
		}
		maze
	}

	pub fn generate_at_point(&mut self, maze: &mut Maze, location: Point) {
		self.visited.set(location, true);
		let mut tried_dirs: u8 = 0;
		while tried_dirs != 0b1111 {
			let mut rand_num: u8 = MazeGenerator::random();
			rand_num = rand_num >> 6;
			let rand_dir = match rand_num{
				0 => Direction::Right,
				1 => Direction::Left,
				2 => Direction::Top,
				_ => Direction::Bottom
			};
			tried_dirs |= 1 << rand_num;
			if let Some(new_location) = location.dir(rand_dir) {
				if !self.visited.get(new_location) {
					maze.break_wall(location, rand_dir);
					self.generate_at_point(maze, new_location);
				}
			}
		}
	}
}
