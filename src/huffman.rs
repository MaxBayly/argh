
use std::ops::RangeBounds;

#[derive(Debug, Clone)]
pub struct BtypeError;

pub fn inflate(data: &Vec<u8>, cursor: &mut usize) -> Result<Vec<u8>, BtypeError> {
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
        _ => {return Err(BtypeError)}
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
        counts: vec![0; (max_bit_length + 1) as usize],
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

    decode(data, &length_codes, &distance_codes);
    
    vec![]
}

struct BitReader<'a> {
    data: &'a[u8],
    bit_pos: usize,
}

impl <'a> BitReader<'a> {
    fn new(data: &'a[u8]) -> Self {
        Self { data, bit_pos: 0 }
    }

    fn read_bit(&mut self) -> Option<u8> {
        if self.bit_pos >= self.data.len() * 8 {
            return None
        }

        let byte = self.data[self.bit_pos / 8];
        let bit = 7 - (self.bit_pos % 8);

        self.bit_pos += 1;

        Some((byte >> bit) & 1)
    }

    fn read_bits(&mut self, n: u8 ) -> Option<u8> {
        if self.bit_pos + n as usize >= self.data.len() * 8 {
            return None
        }

        let byte = self.data[self.bit_pos / 8];
        let lower_bit = 7 - (self.bit_pos % 8) as u8;

        self.bit_pos += n as usize;
        // we shift till the upper bit (so lower bit + n) is in LSB
        // then bitwise AND with just the ones in the lower n positions
        // found by subtracting 1 from the next power of 2 after n
        Some((byte >> (lower_bit - (n - 1))) & (2^(n+1)) - 1)
    }
}

fn decode(data: &Vec<u8>, length_codes: &HuffmanTable, distance_codes: &HuffmanTable) -> Vec<u16> {
    let mut decoded: u16 = 0;
    let mut bit_reader = BitReader::new(data);
    let mut output: Vec<u16> = vec![];
    while decoded != 256 {
        decoded = get_symbol(&mut bit_reader, length_codes).unwrap_or(0);
        if decoded < 256 {
            output.push(decoded);
        } else if 256 < decoded && decoded <= 258 {
            let length = decode_length(decoded, &mut bit_reader) as usize;
            let distance_code = bit_reader.read_bits(5).unwrap_or(0);

            let distance = decode_distance(distance_code, &mut bit_reader) as usize;

            // go back distance bytes and copy length bytes from there
            let start = output.len() - distance as usize;
            let mut copied: Vec<u16> = output[start..start+length].to_vec();
            output.append(&mut copied);
        }
    }
    
    output
}

fn decode_length(length_code: u16, bit_reader: &mut BitReader) -> u16 {
    if length_code < 265 {
        length_code - 254
    } else if length_code < 285  {
        let extra_bits = ((length_code - 265) / 4) + 1;
        (length_code - 254) + bit_reader.read_bits(extra_bits as u8).unwrap_or(0) as u16
    } else { // length code is 285
        258
    }

}

fn decode_distance(distance_code: u8, bit_reader: &mut BitReader) -> u16 {
    if distance_code < 4 {
        (distance_code + 1) as u16
    } else {
        let extra_bits = ((distance_code - 4) / 2) + 1;
        ((distance_code + 1) + bit_reader.read_bits(extra_bits).unwrap_or(0)) as u16
    }
}

fn get_symbol(bit_reader: &mut BitReader, huffman: &HuffmanTable) -> Option<u16> {
    let mut symbol = b'0';
    let mut first = 0; // first tracks the value of the smallest code of a given length
    let mut index = 0; // index tracks the offset in the symbol vector of the first code of a given length
    for bit_len in 1..16 {
        let next_bit = bit_reader.read_bit();
        match next_bit {
            None => { return None }
            Some(bit) => {
                symbol << 1;
                symbol | bit;
            }
        }
        // see how many codes we have at this length
        let count = huffman.counts[bit_len];
        if symbol < first + count { // if the code is between the first code and the maximum code of this length, we have a match
            return Some(huffman.symbols[(index + (symbol - first)) as usize]);
        } else {
            // otherwise, we set index to the start of the next length, first to the largest
            // symbol value of this length, shift first left 1 so it is at the smallest value
            // of the next length, and shift the code left 1 so it is ready to accept the
            // next bit off the data stream
            index += count;
            first += count;
            first <<= 1;
            symbol <<= 1;
        }
    }

    None
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