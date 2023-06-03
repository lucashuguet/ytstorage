mod convert;
mod encode;
mod decode;
mod frame;
mod head;

use crate::head::{gen_headpage, parse_headpage};
use crate::convert::{byte_to_bits, bin_to_dec, pages_to_bytes};
use crate::encode::create_video;
use crate::decode::decode_video;
use crate::frame::Frame;

use std::fs::File;
use std::path::Path;
use clap::Parser;
use std::process;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: String,

    #[arg(long, default_value_t = 1280)]
    width: u32,

    #[arg(long, default_value_t = 720)]
    height: u32,

    #[arg(long, default_value_t = 5)]
    pixel: u32,

    #[arg(long, default_value_t = 24)]
    fps: u32,

    #[arg(long, default_value_t = false)]
    rgb: bool,

    #[arg(short, long, default_value = None)]
    name: Option<String>,

    #[arg(short, long, default_value_t = false)]
    extract: bool
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let path  = Path::new(&args.file);
    if !path.exists() {
	eprintln!("File does not exists");
	process::exit(1);
    }

    let file = File::open(path).expect("Error opening file");
    let filename = path.file_name().expect("Error while getting file name").to_str().expect("There may be weird symbols in your file name");

    if !args.extract {
	if let Some(name) = args.name {
	    create_video(file, filename, args.width, args.height, args.pixel, &name, args.fps);
	} else {
	    eprintln!("Please specify a name for the output file");
	    process::exit(1);
	}
    } else {
	decode_video(&args.file)?;
    }

    Ok(())
}
