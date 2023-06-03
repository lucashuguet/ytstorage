use crate::{Frame, gen_headpage, byte_to_bits};

use std::io::Read;
use std::io::BufReader;

use std::fs::File;

use std::process;

use indicatif::{ProgressIterator, ProgressBar, ProgressStyle};

use crossbeam::thread;

use opencv::videoio::{VideoWriter, VideoWriterTrait};
use opencv::core::Size;

pub fn create_video(file: File, filename: &str, width: u32, height: u32, pixel: u32, output: &str, fps: u32) {
    if width % pixel != 0 || height % pixel != 0 {
	eprintln!("pixel size, width or height are incompatible");
	process::exit(1);
    }

    if (width * height) % 8 != 0 {
	eprintln!("can't store bytes on a frame");
	process::exit(1);
    }

    let bytes_per_frame = (width / pixel) * (height / pixel) / 8;
    let file_size = file.metadata().unwrap().len();

    let frames_count = (file_size as f64 / bytes_per_frame as f64).ceil() as usize;
    let remain = bytes_per_frame * frames_count as u32 - file_size as u32;

    let buf = BufReader::new(file);
    let mut bytes = buf.bytes();

    let fourcc = VideoWriter::fourcc('a', 'v', 'c', '1').unwrap();
    let mut video = VideoWriter::new(output, fourcc, fps as f64, Size::new(1280, 720), true).unwrap();

    let head = gen_headpage(filename, pixel, remain, frames_count as u32);

    let mut headframe = Frame::new(head, 40, 1280, 720);
    headframe.compute_colors();

    video.write(&headframe.image).unwrap();

    let pb = ProgressBar::new(frames_count as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
        )
        .unwrap(),
    );

    for _ in (0..frames_count).progress_with(pb) {
	thread::scope(|s| {
	    let mut data = Vec::new();
	    for _ in 0..bytes_per_frame {
		match bytes.next() {
		    Some(b) => data.append(&mut byte_to_bits(&(b.unwrap() as u8))),
		    _ => data.append(&mut vec![false; 8])
		}
	    }

	    s.spawn(|_| {
		let mut frame = Frame::new(data, pixel, width, height);
		frame.compute_colors();
		video.write(&frame.image).unwrap();
	    });

	}).unwrap();
    }
}
