use std::{path::Path, sync::mpsc::channel, thread, time::Instant};
use image::Pixel;
use std::fs::read_dir;

//aliases for a single pixel and an image buffer
type RGB = image::Rgb<u8>;
type ImgBuffer = image::ImageBuffer<RGB, Vec<u8>>;

//map where red, blue and green color channels are the key and an image is the value
type BlockMap = Vec<([u8; 3], ImgBuffer)>;

//block should be 16 pixels wide and 16 pixels high
const BLOCK_SIZE: u32 = 16;

#[inline]
fn trim_func(c: char) -> bool {
    " \t\'\"\n".contains(c)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_image(input_name: String, output_name: String, scale: u32) {
    dbg!(&input_name, &output_name, &scale);
    let map = get_blocks_map();

    if input_name.is_empty() {
        panic!("No input file has been given!");
    }
    let input_name = Path::new(input_name.trim_matches(trim_func));

    let output_name = Path::new(output_name.trim_matches(trim_func));

    let input_img = image::open(input_name)
        .expect("Unable to open the input image")
        .to_rgb8();

    //benchmark timer
    let now = Instant::now();

    let mut output_img = ImgBuffer::new(
        input_img.width() * BLOCK_SIZE / scale,
        input_img.height() * BLOCK_SIZE / scale,
    );
    let (sender, receiver) = channel();

    thread::spawn(move || {
        for y in (0..(input_img.height() / scale * scale)).step_by(scale as _) {
            for x in (0..(input_img.width()) / scale * scale).step_by(scale as _) {
                let mut colors = [0u32, 0, 0];
                for i in 0..scale {
                    for j in 0..scale {
                        for k in 0..3 {
                            colors[k] += input_img.get_pixel(x + j, y + i).channels()[k] as u32;
                        }
                    }
                }
                sender
                    .send((
                        x,
                        y,
                        get_best_block(&map, colors.map(|c| (c / (scale * scale)) as u8)),
                    ))
                    .expect("Error occured during sending block");
            }
        }
    });

    for (_x, _y, block) in receiver {
        for x in 0..BLOCK_SIZE {
            for y in 0..BLOCK_SIZE {
                *output_img
                    .get_pixel_mut(BLOCK_SIZE * _x / scale + x, BLOCK_SIZE * _y / scale + y) =
                    *block.get_pixel(x, y);
            }
        }
    }

    let time1 = now.elapsed();
    println!("Processing completed in {:.2?}.", time1);

    let now = Instant::now();
    output_img
        .save(output_name)
        .expect("Cannot save the output image");

    let time2 = now.elapsed();
    println!("Saving image took {:.2?}.", time2);
    println!("Total time: {:.2?}.", time1 + time2);
}

//generates a Minecraft BlockMap with images in blocks
fn get_blocks_map() -> BlockMap {
    let files = read_dir(format!("{}/blocks", env!("CARGO_MANIFEST_DIR")))
        .expect("You must download Minecraft blocks textures into block directory");

    let mut map = Vec::with_capacity(177);

    for i in files {
        let i = match i {
            Ok(x) => x,
            Err(_) => continue,
        };

        let img = match image::open(i.path()) {
            Ok(x) => x.to_rgb8(),
            Err(_) => continue,
        };

        let mut colors = [0u32, 0, 0];

        for j in img.pixels() {
            for k in 0..3 {
                colors[k] += j.channels()[k] as u32;
            }
        }
        map.push((colors.map(|c| (c / (BLOCK_SIZE * BLOCK_SIZE)) as u8), img));
    }
    map
}

//matches a pixel with a block
pub fn get_best_block(map: &BlockMap, colors: [u8; 3]) -> ImgBuffer {
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
