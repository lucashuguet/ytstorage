use crate::{HEAD_LENGHT, VideoInfo, VideoType, error, byte_to_bits, bin_to_dec, dec_to_bin, bin_to_str};
use std::process;

fn parse_head_iterator<'a, I>(mut iterator: I, total_iter: usize) -> Vec<bool> 
where
    I: Iterator<Item = &'a bool>,
{
    (0..total_iter).map(|_| {
	if let Some(b) = iterator.next() {
	    *b
	} else {
	    error("no more values in iterator");
	}
    }).collect()
}

pub fn gen_headpage(mut info: VideoInfo) -> Vec<bool>{
    let filename = info.filename();
    let checksum = info.checksum();

    assert!(filename.len() <= 32);
    assert!(checksum.len() == 64);

    let mut out = Vec::new();

    let mut video_type_b = match info.video_type {
	VideoType::BlackNWhite => dec_to_bin(0u8, 4),
	VideoType::GrayScale => dec_to_bin(1u8, 4),
	VideoType::Color => dec_to_bin(2u8, 4),
	VideoType::Color8 => dec_to_bin(3u8, 4),
	VideoType::Color16 => dec_to_bin(4u8, 4)
    };

    let mut checksum_b: Vec<bool> = checksum.chars().flat_map(|c| byte_to_bits(&(c as u8))).collect();
    assert!(checksum_b.len() == 512);

    let mut filename_b: Vec<bool> = (0..(256 - filename.len() * 8)).map(|_| false).collect();

    for c in filename.chars() {
	filename_b.append(&mut byte_to_bits(&(c as u8)))
    }
    assert!(filename_b.len() == 256);

    let mut pixel_size_b = dec_to_bin(info.pixel_size, 8);
    let mut unused_bytes_b = dec_to_bin(info.unused_bytes(), 32);
    let mut total_frames_b = dec_to_bin(info.total_frames() as u32, 32);

    out.append(&mut video_type_b);   // 4 bits
    out.append(&mut checksum_b);     // 512 bits
    out.append(&mut filename_b);     // 256 bits
    out.append(&mut pixel_size_b);   // 8 bits
    out.append(&mut unused_bytes_b); // 32 bits
    out.append(&mut total_frames_b); // 32 bits
    // => total of 844 bits

    assert!(out.len() == HEAD_LENGHT as usize);
    
    out
}

pub fn parse_headpage(head_raw: &[bool]) -> VideoInfo {
    let head: Vec<bool> = (0..(HEAD_LENGHT as usize)).map(|i| head_raw[i]).collect();
    let mut head_iter = head.iter();

    let video_type_b = parse_head_iterator(&mut head_iter, 4);
    let checksum_b = parse_head_iterator(&mut head_iter, 512);
    let filename_b = parse_head_iterator(&mut head_iter, 256);
    let pixel_size_b = parse_head_iterator(&mut head_iter, 8);
    let unused_bytes_b = parse_head_iterator(&mut head_iter, 32);
    let total_frames_b = parse_head_iterator(&mut head_iter, 32);

    assert!(head_iter.next().is_none());

    let checksum = bin_to_str(&checksum_b);
    let filename = bin_to_str(&filename_b);

    let video_type_d = bin_to_dec(&video_type_b);

    let pixel_size = bin_to_dec(&pixel_size_b);
    let unused_bytes = bin_to_dec(&unused_bytes_b);
    let total_frames = bin_to_dec(&total_frames_b);

    let video_type = match video_type_d {
	0 => VideoType::BlackNWhite,
	1 => VideoType::GrayScale,
	2 => VideoType::Color,
	3 => VideoType::Color8,
	4 => VideoType::Color16,
	_ => {
	    eprintln!("unreconized video type");
	    process::exit(1);
	}
    };

    VideoInfo::from_parse(video_type, checksum, filename, pixel_size, unused_bytes, total_frames)
}
