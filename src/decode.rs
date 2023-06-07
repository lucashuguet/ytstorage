use std::fs::OpenOptions;
use std::os::unix::prelude::FileExt;

use crate::{HEAD_LENGHT, VideoType, max_pixel_size, error, parse_headpage, pages_to_bytes};

use indicatif::{ProgressBar, ProgressStyle, ProgressIterator};

use opencv::prelude::MatTraitConst;
use opencv::videoio::{self, VideoCapture, VideoCaptureTrait, VideoCaptureTraitConst};
use opencv::core::Mat;

pub fn decode_video(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut video = VideoCapture::from_file(filename, videoio::CAP_ANY)?;
    let mut frame = Mat::default();

    let width = video.get(videoio::CAP_PROP_FRAME_WIDTH)? as u32;
    let height = video.get(videoio::CAP_PROP_FRAME_HEIGHT)? as u32;

    let pixel_size = max_pixel_size(HEAD_LENGHT, width, height) as u32;

    video.read(&mut frame)?;

    let bits = decode_black_and_white(&frame, width, height, pixel_size)?;
    let mut info = parse_headpage(&bits);

    println!("extracting {}", info.filename());
    
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(info.filename())?;

    let mut offset = 0u64;
    let bits_per_page = (width / info.pixel_size as u32) * (height / info.pixel_size as u32);

    let pb = ProgressBar::new(info.total_frames() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
        )?,
    );

    for i in (0..info.total_frames()).progress_with(pb) {
	video.read(&mut frame)?;

	let mut buffer = match info.video_type {
	    VideoType::BlackNWhite => decode_black_and_white(&frame, width, height, info.pixel_size as u32)?,
	    _ => error("not yet implemented")
	};

	assert!(buffer.len() == bits_per_page as usize);

	if i == info.total_frames() -1 {
	    buffer.truncate(buffer.len() - (info.unused_bytes() * 8) as usize);
	}

	file.write_all_at(&pages_to_bytes(&buffer), offset)?;
	offset += bits_per_page as u64 /8;
    }

    let filename = info.filename();
    let path = std::path::Path::new(&filename);
    let new_checksum = match sha256::try_digest(path) {
	Ok(s) => s,
	Err(_) => error("failed to get extracted file's checksum")
    };

    if new_checksum == info.checksum.unwrap() {
	println!("extracted file is intact");
    } else {
	println!("extracted file is corrupted");
    }

    Ok(())
}

fn decode_black_and_white(frame: &Mat, width: u32, height: u32, pixel_size: u32) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
    let mut bits = Vec::new();
    for i in 0..(height / pixel_size) {
	for j in 0..(width / pixel_size) {
	    let x = (i * pixel_size + pixel_size /2) as i32;
	    let y = (j * pixel_size + pixel_size /2) as i32;
	    let bgr = frame.at_2d::<opencv::core::Vec3b>(x, y)?;
	    
	    let avg: u32 = bgr.iter().map(|c| *c as u32).sum::<u32>() / 3;
	    
	    if avg >= 128 {
		bits.push(true);
	    } else {
		bits.push(false);
	    }
	}
    }

    Ok(bits)
}
