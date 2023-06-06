use crate::VideoType;
use opencv::core::{Mat, MatTrait, CV_8UC3};

struct Pixel {
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
	assert!(data.len() as u32 <= (width / pixel_size as u32) * (height / pixel_size as u32));
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

	let mut coords = Vec::new();
	for j in 0..(self.height / self.pixel_size) {
	    for i in 0..(self.width / self.pixel_size) {
		coords.push((i, j))
	    }
	}

	let pixels: Vec<Pixel> = match video_type {
	    VideoType::BlackNWhite => black_and_white(&self.data),
	    VideoType::GrayScale => todo!(),
	    VideoType::Color8 => todo!(),
	    VideoType::Color16 => todo!()
	};

	for (i, pixel) in pixels.iter().enumerate() {
	    let (i, j) = (coords[i].0, coords[i].1);

	    for y in (j * self.pixel_size)..(j * self.pixel_size + self.pixel_size) {
		for x in (i * self.pixel_size)..(i * self.pixel_size + self.pixel_size) {
		    let bgr = self.image.at_2d_mut::<opencv::core::Vec3b>(y as i32, x as i32).unwrap();
		    bgr[0] = pixel.b;
		    bgr[1] = pixel.g;
		    bgr[2] = pixel.r;
		}
	    }
	}
    }
}

fn black_and_white(data: &[bool]) -> Vec<Pixel> {
    todo!()
}
