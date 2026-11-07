mod huffman;

use std::error::Error;
use std::fs;
use std::string::FromUtf8Error;
use crate::huffman::BitReader;

#[derive(Debug)]
struct Extra {
    len: usize,
    data: Vec<u8>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let data: Vec<u8> = fs::read("compressed_file.txt.gz")?;

    let mut bit_reader = BitReader::new(&data);

    // let test_bytes = bit_reader.read_bits(6);
    // let big_read = bit_reader.read_bits(13);

    let id1 = data[0];
    let id2 = data[1];
    let cm = data[2];
    let flags = data[3];
    let mtime = u32::from_le_bytes(data[4..8].try_into()?);
    let extra_flags = data[8];
    let os = data[9];

    println!(
        "
id1: {}
id2: {}
cm: {}
flags: {:08b}
mtime: {:?}
extra_flags: {}
os: {}
    ",
        id1, id2, cm, flags, mtime, extra_flags, os
    );

    let mut byte_cursor = 10usize;

    let extra = parse_extra(flags, &data, &mut byte_cursor);

    let fname_mask = 1 << 3;
    let filename = parse_string_field(fname_mask, flags, &data, &mut byte_cursor)?;

    let fcomment_mask = 1 << 4;
    let comment = parse_string_field(fcomment_mask, flags, &data, &mut byte_cursor);

    let crc16 = parse_crc16(flags, &data, &mut byte_cursor)?;

    // let mut compressed_blocks: Vec<u8> = vec![];
    // while (byte_cursor < data.len() - 9) {
    //     compressed_blocks.push(data[byte_cursor]);
    //     byte_cursor += 1;
    // }

    let inflated = huffman::inflate(&data[byte_cursor..data.len()].to_vec());

    let crc32 = u32::from_le_bytes(data[byte_cursor..byte_cursor + 4].try_into()?);
    byte_cursor += 4;
    let isize = u32::from_le_bytes(data[byte_cursor..byte_cursor + 4].try_into()?);

    println!("extra data: {:?}", extra);
    println!("filename: {:?}", filename);
    println!("comment: {:?}", comment);
    println!("crc16: {:?}", crc16);
    println!("crc32: {}", crc32);
    println!("isize: {}", isize);
    Ok(())
}

fn parse_extra(flags: u8, data: &Vec<u8>, cursor: &mut usize) -> Option<Extra> {
    let xlen_mask = 1 << 2;

    if xlen_mask & flags != 0 {
        let xlen = data[*cursor] as usize;
        *cursor += 1;
        let xdata = &data[*cursor..*cursor + xlen];
        *cursor += xlen;
        Some(Extra {
            len: xlen,
            data: Vec::from(xdata),
        })
    } else {
        None
    }
}

fn parse_string_field(
    mask: u8,
    flags: u8,
    data: &Vec<u8>,
    cursor: &mut usize,
) -> Result<Option<String>, FromUtf8Error> {
    if mask & flags != 0 {
        let mut chars: Vec<u8> = vec![];
        let mut char = data[*cursor];
        while char != 0 {
            chars.push(char);
            *cursor += 1;
            char = data[*cursor];
        }

        *cursor += 1;
        let string = String::from_utf8(chars)?;
        Ok(Some(string))
    } else {
        Ok(None)
    }
}

fn parse_crc16(
    flags: u8,
    data: &Vec<u8>,
    cursor: &mut usize,
) -> Result<Option<u16>, Box<dyn Error>> {
    let crc16_mask = 1 << 1;
    if crc16_mask & flags != 0 {
        let crc16 = u16::from_le_bytes(data[*cursor..*cursor + 2].try_into()?);
        *cursor += 2;
        Ok(Some(crc16))
    } else {
        Ok(None)
    }
}
