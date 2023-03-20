use std::{
	path::Path,
	fs::read_dir,
};
use image::Pixel;

//aliases for a single pixel and an image buffer
pub type RGB = image::Rgb<u8>;
pub type ImgBuffer = image::ImageBuffer<RGB, Vec<u8>>;

//map where red, blue and green color channels are the key and an image is the value
pub type BlockMap = Vec<([u8; 3], ImgBuffer)>;

//block should be 16 pixels wide and 16 pixels high
pub const BLOCK_SIZE: u32 = 16;

//generates a Minecraft BlockMap with images in ./blocks
pub fn get_blocks_map() -> BlockMap {
	let files = read_dir(Path::new("./blocks"))
		.expect("You must download Minecraft blocks textures into ./block directory");

	let mut map = Vec::with_capacity(177);

	for i in files {
		let i = match i {
			Ok(x) => x,
			Err(_) => continue
		};

		let img = match image::open(i.path()) {
			Ok(x) => x.to_rgb8(),
			Err(_) => continue
		};

		let mut colors = [0u32, 0, 0];

		for j in img.pixels() {
			for k in 0..3 {
				colors[k] += j.channels()[k] as u32;
			}
		}
		map.push((colors.map(|c| (c/(BLOCK_SIZE*BLOCK_SIZE)) as u8), img));
	}
	map
}

//matches a pixel with a block
pub fn get_best_block(map: &BlockMap, pixel: &RGB) -> ImgBuffer {
	let colors = pixel.channels();
	let mut diffrence = u32::max_value();
	let mut img = &ImgBuffer::default();

	for (key, picture) in map {
		let mut tmp_sum = 0u32;
		for i in 0..3 {
			tmp_sum += (colors[i] as i32 - key[i] as i32).abs() as u32;
		}
		if tmp_sum < diffrence {
			img = &picture;
			diffrence = tmp_sum;
		}
	}
	img.to_owned()
}