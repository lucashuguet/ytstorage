use std::fs::OpenOptions;
use std::os::unix::prelude::FileExt;

use crate::{parse_headpage, pages_to_bytes};

use indicatif::{ProgressBar, ProgressStyle, ProgressIterator};

use opencv::prelude::MatTraitConst;
use opencv::videoio::{self, VideoCapture, VideoCaptureTrait, VideoCaptureTraitConst};
use opencv::core::Mat;

pub fn decode_video(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut video = VideoCapture::from_file(filename, videoio::CAP_ANY)?;
    let mut frame = Mat::default();

    let width = video.get(videoio::CAP_PROP_FRAME_WIDTH)?;
    let height = video.get(videoio::CAP_PROP_FRAME_HEIGHT)?;

    video.read(&mut frame)?;

    let mut bits = Vec::new();
    
    for i in 0..(height as u32 / 40) {
	for j in 0..(width as u32 / 40) {
	    let bgr = frame.at_2d::<opencv::core::Vec3b>((i * 40 + 20) as i32, (j * 40 + 20) as i32)?;

	    let avg: u32 = bgr.iter().map(|c| c.clone() as u32).sum::<u32>() / 3;

	    if avg >= 128 {
		bits.push(true);
	    } else {
		bits.push(false);
	    }
	}
    }

    let (name, pixel, remain, pages) = parse_headpage(&bits);

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(name)?;

    let mut offset = 0u64;
    let bits_per_page = (width as u32 / pixel) * (height as u32 / pixel);

    let pb = ProgressBar::new(pages as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
        )
        .unwrap(),
    );

    for i in (0..pages).progress_with(pb) {
	let mut buffer = Vec::new();

	video.read(&mut frame)?;

	for i in 0..(height as u32 / pixel) {
	    for j in 0..(width as u32 / pixel) {
		let bgr = frame.at_2d::<opencv::core::Vec3b>((i * pixel + pixel/2) as i32, (j * pixel + pixel/2) as i32)?;

		let avg: u32 = bgr.iter().map(|c| c.clone() as u32).sum::<u32>() / 3;

		if avg >= 128 {
		    buffer.push(true);
		} else {
		    buffer.push(false);
	    }
	    }
	}

	assert!(buffer.len() == bits_per_page as usize);

	if i == pages -1 {
	    buffer.truncate(buffer.len() - (remain * 8) as usize);
	}

	file.write_all_at(&pages_to_bytes(&buffer), offset)?;
	offset += bits_per_page as u64 /8;
    }

    Ok(())
}
