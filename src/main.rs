use std::{
	path::Path,
	sync::mpsc::channel,
	env::args,
	thread,
	time::Instant,
	process::exit,
};

mod block_map;
use block_map::*;
use image::Pixel;
mod help;

#[inline]
fn trim_func(c: char) -> bool {
	" \t\'\"\n".contains(c)
}

fn main() {
	let map = get_blocks_map();

	let mut it = args().skip(1);
	let mut output_name = String::new();
	let mut scale: u32 = 0;
	let mut input_name = String::new();

	while let Some(i) = it.next() {
		if i.as_bytes()[0] == b'-' {
			let a = it.next().unwrap_or_default();
			match i.as_str() {
				"-o" => output_name = a,
				"-s" => scale = a.parse().expect("The scale must be a non-negative integer"),
				"-h" | "--help" => {
					println!("{}", help::HELP);
					exit(0);
				},
				"-v" | "--version" => {
					println!("Version: {}", env!("CARGO_PKG_VERSION"));
					exit(0);
				},
				_ => eprintln!("Invalid flag: `{i}` (ignored)")
			}
		} else {
			input_name = i;
		}
	}

	if input_name.is_empty() {
		panic!("No input file has been given!");
	}
	let input_name = Path::new(input_name.trim_matches(trim_func));

	if output_name.is_empty() {
		output_name = format!("./pixelart_{}", input_name.file_name().unwrap_or_default().to_str().expect("Unable to convert to an UTF-8 string"));
	}
	let output_name = Path::new(output_name.trim_matches(trim_func));

	if scale == 0 {
		scale = 1;
	}

	let input_img = image::open(input_name).expect("Unable to open the input image").to_rgb8();

	//benchmark timer
	let now = Instant::now();

	let mut output_img = ImgBuffer::new(input_img.width()*BLOCK_SIZE/scale, input_img.height()*BLOCK_SIZE/scale);
	let (sender, receiver) = channel();

	thread::spawn(move || {
		for y in (0..(input_img.height()/scale * scale)).step_by(scale as _) {
			for x in (0..(input_img.width())/scale * scale).step_by(scale as _) {
				let mut colors = [0u32, 0, 0];
				for i in 0..scale {
					for j in 0..scale {
						for k in 0..3 {
							colors[k] += input_img.get_pixel(x+j, y+i).channels()[k] as u32;
						}
					}
				}
				sender.send((x, y, get_best_block(&map, colors.map(|c| (c / (scale*scale)) as u8)))).expect("Error occured during sending block");
			}
		}
	});

	for (_x, _y, block) in receiver {
		for x in 0..BLOCK_SIZE {
			for y in 0..BLOCK_SIZE {
				*output_img.get_pixel_mut(BLOCK_SIZE * _x / scale + x, BLOCK_SIZE * _y / scale + y) = *block.get_pixel(x, y);
			}
		}
	}

	let time1 = now.elapsed();
	println!("Processing completed in {:.2?}.", time1);

	let now = Instant::now();
	output_img.save(output_name).expect("Cannot save the output image");

	let time2 = now.elapsed();
	println!("Saving image took {:.2?}.", time2);
	println!("Total time: {:.2?}.", time1 + time2);
}