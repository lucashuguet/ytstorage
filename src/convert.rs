use itertools::Itertools;
use std::process;

pub fn dec_to_bin(dec: u32, bits: usize) -> Vec<bool> {
    let mut binstr = format!("{:b}", dec);
    while binstr.len() < bits {
	binstr = format!("0{}", binstr);
    }

    binstr.chars().map(|c| {
	match c {
	    '0' => false,
	    '1' => true,
	    _ => process::exit(1)
	}
    }).collect()
}

pub fn bin_to_dec(bin: &Vec<bool>) -> Option<u32> {
    let mut binstr = String::new();
    for b in bin {
	match b {
	    false => binstr = format!("{}0", binstr),
	    true => binstr = format!("{}1", binstr)
	}
    }

    match u32::from_str_radix(&binstr, 2) {
	Ok(out) => Some(out),
	Err(_) => None
    }
}

pub fn byte_to_bits(byte: &u8) -> Vec<bool> {
    let mut cbyte = format!("{:b}", byte);
    while cbyte.len() < 8 {
	cbyte = format!("0{}", cbyte);
    }

    return cbyte.chars().map(|c| {
	match c {
	    '0' => false,
	    '1' => true,
	    _ => process::exit(1)
	}
    }).collect();
}

pub fn bits_to_byte(bits: &Vec<bool>) -> u8 {
    assert!(bits.len() == 8);
    let mut tmp_str = String::new();
    for b in bits.iter() {
	match b {
	    false => tmp_str = format!("{}0", tmp_str),
	    true => tmp_str = format!("{}1", tmp_str),
	}
    }
    return u8::from_str_radix(&tmp_str, 2).unwrap();
}

pub fn pages_to_bytes(bits: &Vec<bool>) -> Vec<u8> {
    let mut out = Vec::new();
    for chunk in &bits.into_iter().chunks(8) {
	let mut bin = String::new();
	for b in chunk {
	    match b {
		false => bin = format!("{}0", bin),
		true => bin = format!("{}1", bin),
	    }
	}
	out.push(u8::from_str_radix(&bin, 2).unwrap());
    }

    return out
}
