use opencv::core::{Mat, MatTrait, Size, CV_8UC3};

pub struct Frame {
    pub image: Mat,
    pub data: Vec<bool>,
    pub size: u32,
    pub width: u32,
    pub height: u32
}

impl Frame {
    pub fn new(data: Vec<bool>, size: u32, width: u32, height: u32) -> Self {
	assert!(data.len() as u32 <= (width / size) * (height / size));
	unsafe {
	    Frame {
		image: Mat::new_rows_cols(height as i32, width as i32, CV_8UC3).unwrap(),
		data,
		size,
		width,
		height
	    }
	}
    }

    pub fn get_frame_size(&self) -> Size {
	Size::new(self.width as i32, self.height as i32)
    }

    pub fn compute_colors(&mut self) {
	let max_bits = (self.width / self.size) * (self.height / self.size);
	assert!(self.data.len() as u32 <= max_bits);

	let mut coords = Vec::new();
	for j in 0..(self.height / self.size) {
	    for i in 0..(self.width / self.size) {
		coords.push((i, j))
	    }
	}

	for (i, &b) in self.data.iter().enumerate() {
	    let (i, j) = (coords[i].0, coords[i].1);

	    for y in (j * self.size)..(j * self.size + self.size) {
		for x in (i * self.size)..(i * self.size + self.size) {
		    let bgr = self.image.at_2d_mut::<opencv::core::Vec3b>(y as i32, x as i32).unwrap();

		    if b {
			bgr[0] = 255;
			bgr[1] = 255;
			bgr[2] = 255;
		    } else {
			bgr[0] = 0;
			bgr[1] = 0;
			bgr[2] = 0;
		    }
		}
	    }
	}
    }
}
