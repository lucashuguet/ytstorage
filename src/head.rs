use crate::convert::{byte_to_bits, bits_to_byte, bin_to_dec, dec_to_bin};

use itertools::Itertools;

pub fn gen_headpage(file: &str, pixel: u32, remain: u32, pages: u32) -> Vec<bool>{
    let mut out = Vec::new();

    let mut filebin: Vec<bool> = (0..(256 - file.len() * 8)).map(|_| false).collect();

    for c in file.chars() {
	filebin.append(&mut byte_to_bits(&(c as u8)))
    }

    let mut pixelbin = dec_to_bin(pixel, 8);
    let mut remainbin = dec_to_bin(remain, 32);
    let mut pagesbin = dec_to_bin(pages, 32);

    out.append(&mut filebin);
    out.append(&mut pixelbin);
    out.append(&mut remainbin);
    out.append(&mut pagesbin);

    assert!(out.len() == 328);
    
    return out
}

pub fn parse_headpage(head_raw: &Vec<bool>) -> (String, u32, u32, u32) {
    let head: Vec<bool> = (0..328).map(|i| head_raw[i]).collect();
    let mut head_iter = head.iter();

    let mut name = Vec::new();
    for _ in 0..256 {
	if let Some(b) = head_iter.next() {
	    name.push(b.clone());
	}
    }

    let mut filename = String::new();
    for c in &name.into_iter().chunks(8) {
	let byte = c.collect::<Vec<bool>>();

	if byte != vec![false; 8] {
	    filename = format!("{}{}", filename, bits_to_byte(&byte) as char);
	}
    }

    let mut pixelbin = Vec::new();
    for _ in 0..8 {
	if let Some(b) = head_iter.next() {
	    pixelbin.push(b.clone());
	}
    }

    let mut remainbin = Vec::new();
    for _ in 0..32 {
	if let Some(b) = head_iter.next() {
	    remainbin.push(b.clone());
	}
    }

    let mut pagesbin = Vec::new();
    for _ in 0..32 {
	if let Some(b) = head_iter.next() {
	    pagesbin.push(b.clone());
	}
    }

    let pixel = bin_to_dec(&pixelbin).expect("failed to convert bin to dec");
    let remain = bin_to_dec(&remainbin).expect("failed to convert bin to dec");
    let pages = bin_to_dec(&pagesbin).expect("failed to convert bin to dec");

    (filename, pixel, remain, pages)
}
