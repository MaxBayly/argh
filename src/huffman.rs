use std::error::Error;
use std::fmt::Error;

pub fn inflate(data: &Vec<u8>, cursor: &mut usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let header = data[*cursor];
    let bfinal_mask = 1;
    let btype_mask = 3 << 1;

    let bfinal = header & bfinal_mask == 1;
    let btype = (header & btype_mask) >> 1;

    println!("{}. {}", bfinal, btype);

    *cursor += 1;

    let decomp_block: Vec<u8> = match btype {
        // 0 => {}
        1 => {inflate_fixed_huffman(data)}
        // 2 => {}
        // 3 => {}
        _ => {return Err(Box::new(Error))}
    };



    // let inflated = data[*cursor..*cursor + len as usize].to_vec();

    // Ok(inflated)
    Ok(vec![])
}

struct HuffmanTable {
    counts: Vec<u8>,
    symbols: Vec<u16>
}

fn construct(lengths: &Vec<u8>, max_symbol: u16, max_bit_length: u8) -> HuffmanTable {
    /*
     initialise Huffman table with max_bit_length count slots
     most of the elements of `counts` will remain at 0
     (e.g. our canonical encoding only uses lengths 7, 8, 9, so 0-6
     will stay as 0)
    */
    let mut huffman = HuffmanTable {
        counts: vec![0; max_bit_length as usize],
        symbols: Vec::new()
    };
    /*
    iterate over every possible symbol and get their code's length from
    a precomputed table. iterate the recorded length's element in the
    counts vector
     */
    for symbol in 0..max_symbol {
        let symbol_length = lengths[symbol as usize] as usize;
        huffman.counts[symbol_length] += 1;
    }

    /*
    initialise offsets vector with max_bit_length + 1, because a code
    length of 0 makes no sense (it will go unused)
     */
    let mut offsets = vec![0; max_bit_length as usize + 1];
    offsets[1] = 0;
    for len in 1..max_bit_length as usize {
        // the next offset is equal to the current offset (which is the count of
        // preceding codes) plus the count of codes of current length
        offsets[len + 1] = offsets[len] + huffman.counts[len];
    }

    for symbol in 0..max_symbol {
        if lengths[symbol as usize] != 0 { // make sure the symbol actually has a code
            let length = lengths[symbol as usize] as usize;
            let offset = &mut offsets[length];
            huffman.symbols[*offset as usize] = symbol;
            *offset += 1; // the offset value must be incremented because we just inserted a symbol at the current index
        }
    }

    huffman

}
fn inflate_fixed_huffman(data: &Vec<u8>) -> Vec<u8> {
    // find first code
    let max_symbol: u16 = 0b111111111;
    let mut lengths: Vec<u8> = vec![0; max_symbol as usize];
    for symbol in 0..144 {
        lengths[symbol] = 8;
    }
    for symbol in 144..256 {
        lengths[symbol] = 9;
    }
    for symbol in 256..280 {
        lengths[symbol] = 7;
    }
    for symbol in 280..288 {
        lengths[symbol] = 8;
    }
    
    let max_distance: u8 = 0b11111;
    let distance_code_lengths: Vec<u8> = vec![5; max_distance as usize];
    

    let length_codes = construct(&lengths, max_symbol, 9);
    let distance_codes = construct(&distance_code_lengths, max_distance as u16, 5);
    
    
    vec![]
}

// fn inflate_uncompressed(data: &Vec<u8>, cursor: &mut usize, len: u16) -> Result<Vec<u8>, Box<dyn Error>> {
//    // let len = u16::from_le_bytes(data[*cursor..*cursor + 2].try_into()?);
//     // *cursor += 2;
//     //
//     // let nlen = u16::from_le_bytes(data[*cursor..*cursor + 2].try_into()?);
//     //
//     // *cursor += 2;
// }

enum CompressionType {
    Uncompressed,
    FixedHuffman,
    DynHuffman,
    Error
}