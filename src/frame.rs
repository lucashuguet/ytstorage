use crate::{VideoType, error};
use opencv::core::{Mat, MatTrait, CV_8UC3};
use itertools::Itertools;

#[derive(Debug)]
struct Pixel {
    x: u32,
    y: u32,
    r: u8,
    g: u8,
    b: u8
}

pub struct Frame {
    pub image: Mat,
    pub data: Vec<bool>,
    pub pixel_size: u32,
    pub width: u32,
    pub height: u32
}

impl Frame {
    pub fn new(data: Vec<bool>, pixel_size: u8, width: u32, height: u32) -> Self {
	// assert!(data.len() as u32 <= (width / pixel_size as u32) * (height / pixel_size as u32));
	unsafe {
	    Frame {
		image: Mat::new_rows_cols(height as i32, width as i32, CV_8UC3).unwrap(),
		data,
		pixel_size: pixel_size as u32,
		width,
		height
	    }
	}
    }

    pub fn compute_colors(&mut self, video_type: VideoType, bits_per_page: u32) {
	assert!(self.data.len() as u32 <= bits_per_page);
	assert!(self.width % self.pixel_size == 0);
	assert!(self.height % self.pixel_size == 0);

	while (self.data.len() as u32) < bits_per_page {
	    self.data.push(false);
	}

	let mut coords = Vec::new();
	for j in 0..(self.height / self.pixel_size) {
	    for i in 0..(self.width / self.pixel_size) {
		coords.push((i, j))
	    }
	}

	let pixels: Vec<Pixel> = match video_type {
	    VideoType::BlackNWhite => encode_black_and_white(&self.data, coords, self.pixel_size),
	    VideoType::Color => encode_color(&self.data, coords, self.pixel_size),
	    _ => error("not yet implemented"),
	};

	for pixel in pixels.iter() {
	    let bgr = self.image.at_2d_mut::<opencv::core::Vec3b>(pixel.y as i32, pixel.x as i32).unwrap();
	    bgr[0] = pixel.b;
	    bgr[1] = pixel.g;
	    bgr[2] = pixel.r;
	}
    }
}

fn encode_black_and_white(data: &[bool], coords: Vec<(u32, u32)>, pixel_size: u32) -> Vec<Pixel> {
    let mut pixels = Vec::new();

    for (idx, &b) in data.iter().enumerate() {
	let (i, j) = (coords[idx].0, coords[idx].1);
	
	for y in (j * pixel_size)..(j * pixel_size + pixel_size) {
	    for x in (i * pixel_size)..(i * pixel_size + pixel_size) {
		let pixel = if b {
		    Pixel {x, y, r: 255, g: 255, b: 255}
		} else {
		    Pixel {x, y, r: 0, g: 0, b: 0}
		};

		pixels.push(pixel);
	    }
	}
    }

    pixels
}

fn encode_color(data: &[bool], coords: Vec<(u32, u32)>, pixel_size: u32) -> Vec<Pixel> {
    let mut pixels = Vec::new();

    for (idx, chunk) in (&data.iter().chunks(3)).into_iter().enumerate() {
	let (i, j) = (coords[idx].0, coords[idx].1);
	let bits: Vec<bool> = chunk.copied().collect();
	
	for y in (j * pixel_size)..(j * pixel_size + pixel_size) {
	    for x in (i * pixel_size)..(i * pixel_size + pixel_size) {
		let r = if bits[0] { 255 } else { 0 };
		let g = if bits[1] { 255 } else { 0 };
		let b = if bits[2] { 255 } else { 0 };

		pixels.push(Pixel {x, y, r, g, b});
	    }
	}
    }

    pixels
}
