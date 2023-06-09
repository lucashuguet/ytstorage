mod convert;
mod encode;
mod decode;
mod frame;
mod head;

use crate::convert::{byte_to_bits, bin_to_dec, dec_to_bin, bin_to_str, pages_to_bytes};
use crate::head::{gen_headpage, parse_headpage};
use crate::encode::create_video;
use crate::decode::decode_video;
use crate::frame::Frame;

use std::fs::File;
use std::path::Path;
use clap::Parser;
use std::process;

const HEAD_LENGHT: u32 = 844;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// input file path
    #[arg(short, long)]
    file: String,

    /// video width
    #[arg(long, default_value_t = 1280)]
    width: u32,

    /// video height
    #[arg(long, default_value_t = 720)]
    height: u32,

    /// bigger pixel_size means less corruption on youtube but very large files
    #[arg(long, default_value_t = 10)]
    pixel_size: u8,

    /// video file fps
    #[arg(long, default_value_t = 24)]
    fps: u32,

    /// output path (only when compressing)
    #[arg(short, long, default_value = None)]
    output: Option<String>,

    /// extract the file from the video
    #[arg(short, long, default_value_t = false)]
    extract: bool,

    /// black, grayscale, color, color8 or color16
    #[arg(long, default_value = "black")]
    video_type: String
}

#[derive(Debug, Clone)]
pub struct VideoInfo {
    video_type: VideoType,
    checksum: Option<String>,
    filename: Option<String>,
    path: Option<String>,
    pixel_size: u8,
    unused_bytes: Option<u32>,
    total_frames: Option<usize>,
    file_size: Option<u64>,
    fps: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
}

impl VideoInfo {
    fn new(video_type: VideoType, path: &str, pixel_size: u8, fps: u32, width: u32, height: u32) -> Self {
	Self {
	    video_type,
	    checksum: None,
	    filename: None,
	    path: Some(path.to_string()),
	    pixel_size,
	    unused_bytes: None,
	    total_frames: None,
	    file_size: None,
	    fps: Some(fps),
	    width: Some(width),
	    height: Some(height),
	}
    }

    fn from_parse(video_type: VideoType, checksum: String, filename: String, pixel_size: u8, unused_bytes: u32, total_frames: usize) -> Self {
	Self {
	    video_type,
	    checksum: Some(checksum),
	    filename: Some(filename),
	    path: None,
	    pixel_size,
	    unused_bytes: Some(unused_bytes),
	    total_frames: Some(total_frames),
	    file_size: None,
	    fps: None,
	    width: None,
	    height: None
	}
    }

    fn total_pixels(&self) -> u32 {
	if self.width.is_none() || self.height.is_none() {
	    error("can't get total pixels without a width and a height");
	}

	self.width.unwrap() * self.height.unwrap()
    }

    fn bytes_per_frame(&self) -> u32 {
	match self.video_type {
	    VideoType::BlackNWhite => get_bytes_per_frame(self.total_pixels(), self.pixel_size as u32, (1, 8)),
	    VideoType::GrayScale => get_bytes_per_frame(self.total_pixels(), self.pixel_size as u32, (3, 8)),
	    VideoType::Color => get_bytes_per_frame(self.total_pixels(), self.pixel_size as u32, (3, 8)),
	    VideoType::Color8 => get_bytes_per_frame(self.total_pixels(), self.pixel_size as u32, (1, 2)),
	    VideoType::Color16 => get_bytes_per_frame(self.total_pixels(), self.pixel_size as u32, (3, 2)),
	}
    }

    fn load_file(&self) -> File {
	let path_str = match &self.path {
	    Some(p) => p,
	    None => error("need a path to load the file")
	};

	let path  = Path::new(&path_str);
	if !path.exists() {
	    error("File does not exists");
	}

	File::open(path).expect("Error opening file")
    }

    fn checksum(&mut self) -> String {
	match &self.checksum {
	    Some(s) => s.clone(),
	    None => {
		let path_str = match &self.path {
		    Some(p) => p,
		    None => error("cannot get checksum without a path")
		};

		let path = Path::new(&path_str);
		let checksum = match sha256::try_digest(path) {
		    Ok(c) => c,
		    Err(_) => error("cannot get checksum of the file")
		};

		self.checksum = Some(checksum.clone());
		checksum
	    },
	}
    }

    fn filename(&mut self) -> String {
	match &self.filename {
	    Some(s) => s.clone(),
	    None => {
		let path_str = match &self.path {
		    Some(p) => p,
		    None => error("cannot get filename without a path")
		};

		let path = Path::new(&path_str);
		let filename = match path.file_name() {
		    Some(f) => match f.to_str() {
			Some(s) => s.to_string(),
			None => error("there may be weird symbols in the file name")
		    },
		    None => error("failed to get file name"),
		};

		self.filename = Some(filename.clone());
		filename
	    },
	}
    }

    fn unused_bytes(&mut self) -> u32 {
	match self.unused_bytes {
	    Some(s) => s,
	    None => {
		let unused_bytes = self.bytes_per_frame() * self.total_frames() as u32 - self.file_size() as u32;
		self.unused_bytes = Some(unused_bytes);
		unused_bytes
	    },
	}
    }

    fn total_frames(&mut self) -> usize {
	match self.total_frames {
	    Some(s) => s,
	    None => {
		let total_frames = (self.file_size() as f64 / self.bytes_per_frame() as f64).ceil() as usize;
		self.total_frames = Some(total_frames);
		total_frames
	    },
	}
    }

    fn file_size(&mut self) -> u64 {
	match self.file_size {
	    Some(s) => s,
	    None => {
		let file_size = get_file_size(self.load_file());
		self.file_size = Some(file_size);
		file_size
	    },
	}
    }

    fn fps(&self) -> u32 {
	match self.fps {
	    Some(f) => f,
	    None => error("you need to specify a fps parameter to create a video")
	}
    }

    fn width(&self) -> u32 {
	match self.width {
	    Some(f) => f,
	    None => error("you need to specify a width parameter to create a video")
	}
    }

    fn height(&self) -> u32 {
	match self.height {
	    Some(f) => f,
	    None => error("you need to specify a height parameter to create a video")
	}
    }
}

#[derive(Debug, Copy, Clone)]
pub enum VideoType {
    BlackNWhite,
    GrayScale,
    Color,
    Color8,
    Color16
}

fn error(msg: &str) -> ! {
    eprintln!("{msg}");
    process::exit(1)
}

fn get_file_size(file: File) -> u64 {
    let meta = file.metadata();

    match meta {
	Ok(m) => m.len(),
	Err(_) => error("failed to get file size")
    }
}

fn get_bytes_per_frame(total_pixels: u32, pixel_size: u32, byte_per_pixel: (u32, u32)) -> u32 {
    let numerator = byte_per_pixel.0;
    let denominator = byte_per_pixel.1;
    
    let pixels_per_frame = if (total_pixels * numerator) % pixel_size.pow(2) == 0 {
	(total_pixels * numerator) / pixel_size.pow(2)
    } else {
	error("width, height, pixel_size and video_type are uncompatible");
    };
    if pixels_per_frame % denominator == 0 {
	pixels_per_frame / denominator
    } else {
	error("width, height, pixel_size and video_type are uncompatible");
    }
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

fn max_pixel_size(bytes_on_frame: u32, width: u32, height: u32) -> u8 {
    let mut pixel_size = gcd(width, height);
    while width % pixel_size != 0
	|| height % pixel_size != 0
	|| (width * height) / pixel_size.pow(2) <= bytes_on_frame
    {
	pixel_size -= 1;
    }

    pixel_size as u8
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let video_type = match args.video_type.as_str() {
	"black" => VideoType::BlackNWhite,
	"grayscale" => VideoType::GrayScale,
	"color" => VideoType::Color,
	"color8" => VideoType::Color8,
	"color16" => VideoType::Color16,
	_ => error("unreconized video type parameter"),
    };

    let info = VideoInfo::new(video_type, &args.file, args.pixel_size, args.fps, args.width, args.height);
    
    if !args.extract {
	if let Some(output) = args.output {
	    create_video(info, &output);
	} else {
	    error("Please specify a name for the output file");
	}
    } else {
	decode_video(&args.file)?;
    }

    Ok(())
}
