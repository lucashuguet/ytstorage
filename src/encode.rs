use crate::{HEAD_LENGHT, VideoInfo, VideoType, Frame, max_pixel_size, error, gen_headpage, byte_to_bits};

use std::io::{Read, BufReader};

use indicatif::{ProgressIterator, ProgressBar, ProgressStyle};

use crossbeam::thread;

use opencv::videoio::{VideoWriter, VideoWriterTrait};
use opencv::core::Size;

pub fn create_video(mut info: VideoInfo, output: &str) {
    let total_pixels = info.total_pixels();
    let total_frames = info.total_frames();

    if total_pixels % info.pixel_size as u32 != 0 {
	error("pixel size, width or height are incompatible");
    }

    if total_pixels % 8 != 0 {
	error("can't store bytes on a frame");
    }

    let buf = BufReader::new(info.load_file());
    let mut bytes = buf.bytes();

    let fourcc = VideoWriter::fourcc('a', 'v', 'c', '1').unwrap();
    let mut video = VideoWriter::new(output, fourcc, info.fps() as f64, Size::new(info.width() as i32, info.height() as i32), true).unwrap();

    let head = gen_headpage(info.clone());

    let head_pixel = max_pixel_size(HEAD_LENGHT, info.width(), info.height());

    let mut headframe = Frame::new(head, head_pixel, info.width(), info.height());
    headframe.compute_colors(VideoType::BlackNWhite, (info.width() * info.height()) / (head_pixel as u32).pow(2));

    video.write(&headframe.image).unwrap();

    let pb = ProgressBar::new(total_frames as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
        )
        .unwrap(),
    );

    for _ in (0..total_frames).progress_with(pb) {
	thread::scope(|s| {
	    s.spawn(|_| {
		let mut data = Vec::new();
		for _ in 0..(info.bytes_per_frame()) {
		    match bytes.next() {
			Some(b) => data.append(&mut byte_to_bits(&b.unwrap())),
			_ => data.append(&mut vec![false; 8])
		    }
		}
		
		let mut frame = Frame::new(data, info.pixel_size, info.width(), info.height());
		frame.compute_colors(info.video_type, info.bytes_per_frame() * 8);
		video.write(&frame.image).unwrap();
	    });
	}).unwrap();
    }
}
