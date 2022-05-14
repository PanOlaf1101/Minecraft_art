use std::io::*;
use std::path::Path;
use std::fs::read_dir;
use std::sync::mpsc;
use std::env::args;
use std::thread;
use std::time::Instant;
use image::Pixel;
use std::collections::HashMap;

//alias for a single pixel
type RGB = image::Rgb<u8>;

type ImgBuffer = image::ImageBuffer<RGB, Vec<u8>>;

//map where red, blue and green color channels are the key and an image is the value
type BlockMap = HashMap<[u8; 3], ImgBuffer>;

//block shall be 16 pixel wide and 16 high
const BLOCK_SIZE: u32 = 16;

//generates a Minecraft BlockMap with images in ./blocks
fn get_blocks_map() -> BlockMap {
	let files = read_dir(Path::new("./blocks"))
		.expect("You must download Minecraft blocks textures into ./block directory");
	let mut map = HashMap::new();

	for i in files {
		let i = i.unwrap();
		let img = match image::open(i.path()) {
			Ok(x) => x.to_rgb8(),
			Err(_) => continue
		};
		let mut colors: [u32; 3] = [0, 0, 0];
		for j in img.pixels() {
			for k in 0..3 {
				colors[k] += j.channels()[k] as u32;
			}
		}
		map.insert(colors.map(|c| (c/256) as u8), img);
	}
	map
}

//matches a pixel with a block
fn get_best_block(map: &BlockMap, pixel: &RGB) -> ImgBuffer {
	let colors = pixel.channels();
	let mut diffrence = 10000u32;
	let mut img = &ImgBuffer::default();

	for (key, picture) in map {
		let mut tmp_sum = 0u32;
		for i in 0..3 {
			tmp_sum += (colors[i] as i16 - key[i] as i16).abs() as u32;
		}

		if tmp_sum < diffrence {
			img = picture;
			diffrence = tmp_sum;
		}
	}
	img.to_owned()
}

fn main() {
	let map = get_blocks_map();

	let name = if args().len() >= 2 {
		args().collect::<Vec<_>>()[1].clone()
	} else {
		print!("Enter image file name: ");
		stdout().flush().unwrap();
		let mut name = String::new();
		stdin().read_line(&mut name).unwrap();
		name
	};
	let name = Path::new(name.trim());

	let input_img = image::open(name).unwrap().to_rgb8();
	println!("Path loaded");

	//benchmarking timer
	let now = Instant::now();

	let mut output_img = ImgBuffer::new(input_img.width()*BLOCK_SIZE, input_img.height()*BLOCK_SIZE);
	let (sender, receiver) = mpsc::channel();

	thread::spawn(move || {
		for (x, y, p) in input_img.enumerate_pixels() {
			sender.send((x, y, get_best_block(&map, p))).expect("Error during sending block");
		}
	});
	for (_x, _y, block) in receiver {
		for x in 0..BLOCK_SIZE {
			for y in 0..BLOCK_SIZE {
				*(output_img.get_pixel_mut(BLOCK_SIZE * _x + x, BLOCK_SIZE * _y + y)) = *(block.get_pixel(x, y));
			}
		}
	}

	let time1 = now.elapsed();
	println!("Processing completed in {:.2?}", time1);

	//suprisingly saving an image on hard drive takes more time than calling ``
	let now = Instant::now();
	output_img.save("./minecraft_art.jpeg").unwrap();

	let time2 = now.elapsed();
	println!("Saving image took {:.2?}", time2);
	println!("Summary: {:.2?}", time1 + time2);
}