use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

mod mode;
use mode::Mode;

mod huffman;

// TODO: use string builders instead of strings

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mode: Mode;
    let in_file_name: &str;
    let out_file_name: &str;

    match args.len() {
        4 => {
            mode = args[1].parse()?;
            in_file_name = &args[2];
            out_file_name = &args[3];
        }
        _ => {
            return Err("invalid arguments".into());
        }
    }

    let mut file_in = File::open(in_file_name).expect("unable to open file");
    let mut file_out = File::create(out_file_name)?;
    let data_out: Vec<u8>;

    println!("{} -> {}", in_file_name, out_file_name);
    match mode {
        Mode::Compress => {
            let mut data_in = String::new();
            file_in
                .read_to_string(&mut data_in)
                .expect("unable to read file");

            data_out = compress(&data_in);
        }
        Mode::Decompress => {
            let mut data_in = Vec::<u8>::new();
            file_in
                .read_to_end(&mut data_in)
                .expect("unable to read file");

            data_out = decompress(&data_in);
        }
    }

    file_out.write_all(&data_out)?;

    Ok(())
}

fn compress(data: &str) -> Vec<u8> {
    let freq_map = create_freq_map(data);
    let code_lookup = huffman::build_code_lookup(&freq_map);

    let mut compressed = Vec::<u8>::new();
    compressed.extend_from_slice(&serialize_code_lookup(&code_lookup));
    compressed.extend_from_slice(&encode_data(data, &code_lookup));

    compressed
}

fn create_freq_map(data: &str) -> HashMap<char, u32> {
    let mut freq_map = HashMap::new();

    for c in data.chars() {
        *freq_map.entry(c).or_insert(0) += 1;
    }

    freq_map
}

fn serialize_code_lookup(code_lookup: &HashMap<char, String>) -> Vec<u8> {
    let mut header = Vec::<u8>::new();
    let code_lookup_len = code_lookup.len() as u32;
    header.extend_from_slice(&code_lookup_len.to_le_bytes());

    for (c, code) in code_lookup {
        let c_bytes_len = c.len_utf8() as u32;
        header.extend_from_slice(&c_bytes_len.to_le_bytes());
        header.extend_from_slice(c.to_string().as_bytes());

        let mut code_bits = string_to_bits(code);
        let code_bits_len = code.len() as u32;
        header.extend_from_slice(&code_bits_len.to_le_bytes());
        header.append(&mut code_bits);
    }

    header
}

fn string_to_bits(s: &str) -> Vec<u8> {
    let mut bits = Vec::new();
    let mut padded_s = s.to_string();

    let remainder = padded_s.len() % 8;
    if remainder != 0 {
        padded_s.push_str(&"0".repeat(8 - remainder));
    }

    for chunk in padded_s.as_bytes().chunks(8) {
        let bit_string = std::str::from_utf8(chunk).unwrap();
        let byte = u8::from_str_radix(bit_string, 2).unwrap();
        bits.push(byte);
    }

    bits
}

fn encode_data(data: &str, code_lookup: &HashMap<char, String>) -> Vec<u8> {
    let mut coded_data = Vec::<u8>::new();
    let mut code = String::new();

    for c in data.chars() {
        code.push_str(code_lookup.get(&c).unwrap());
        while code.len() >= 8 {
            let byte = u8::from_str_radix(&code[..8], 2).unwrap();
            coded_data.push(byte);
            code = code[8..].to_string();
        }
    }

    if code.len() > 0 {
        let padding = 8 - code.len();
        code.push_str(&"0".repeat(padding));
        let byte = u8::from_str_radix(&code[..8], 2).unwrap();
        coded_data.push(byte);
    }

    coded_data
}

fn decompress(data: &Vec<u8>) -> Vec<u8> {
    let mut data = data.clone();

    let code_lookup = parse_code_lookup(&mut data);
    let code = bits_to_string(&data, data.len() * 8);
    let decoded_data = decode_data(code, code_lookup);

    decoded_data.as_bytes().to_vec()
}

fn parse_code_lookup(data: &mut Vec<u8>) -> HashMap<char, String> {
    let code_lookup_len = u32::from_le_bytes(data[..4].to_vec().try_into().unwrap());
    data.drain(0..4);

    let mut code_lookup = HashMap::new();
    for _ in 0..code_lookup_len {
        let c_bytes_len = u32::from_le_bytes(data[..4].to_vec().try_into().unwrap()) as usize;
        data.drain(0..4);
        let c_bytes = data[..c_bytes_len].to_vec();
        let c = std::str::from_utf8(&c_bytes).unwrap().chars().next().unwrap();
        data.drain(0..c_bytes_len);

        let code_bits_len = u32::from_le_bytes(data[..4].try_into().unwrap()) as usize;
        data.drain(0..4);
        let code_bytes_len = (code_bits_len + 7) / 8;
        let code_bits = data[..code_bytes_len].to_vec();
        let code = bits_to_string(&code_bits, code_bits_len);
        data.drain(0..code_bytes_len);

        code_lookup.insert(c, code);
    }

    code_lookup
}

fn bits_to_string(bytes: &[u8], len: usize) -> String {
    let mut code = String::new();
    for byte in bytes {
        code.push_str(&format!("{:08b}", byte));
    }

    code[..len].to_string()
}

fn decode_data(code: String, code_lookup: HashMap<char, String>) -> String {
    let mut decoded_data = String::new();
    let mut i = 0;
    while i < code.len() {
        let mut j = i + 1;
        while !code_lookup.values().any(|v| **v == code[i..j]) {
            j += 1;
        }

        let c = code_lookup.iter().find(|(_, v)| ***v == code[i..j]).unwrap().0;
        decoded_data.push(*c);
        i = j;
    }
    decoded_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_freq_map() {
        let data = "hello";
        let mut expected = HashMap::new();
        expected.insert('h', 1);
        expected.insert('e', 1);
        expected.insert('l', 2);
        expected.insert('o', 1);

        assert_eq!(create_freq_map(&data), expected);
    }

    #[test]
    fn test_string_to_bits() {
        let s = "01100100101010010101000001";
        let expected = vec![0b01100100, 0b10101001, 0b01010000, 0b01000000];
        assert_eq!(string_to_bits(&s), expected);
    }

    #[test]
    fn test_bits_to_string() {
        let bytes = vec![0b01100100, 0b10101001, 0b01010000, 0b01000000];
        let len = 25;
        let expected = "0110010010101001010100000";
        assert_eq!(bits_to_string(&bytes, len), expected);
    }

    #[test]
    fn test_encode_data() {
        let data = "hello";
        let code_lookup = {
            let mut code_lookup = HashMap::new();
            code_lookup.insert('h', "00".to_string());
            code_lookup.insert('e', "01".to_string());
            code_lookup.insert('l', "10".to_string());
            code_lookup.insert('o', "11".to_string());
            code_lookup
        };
        let expected = vec![0b00011010, 0b11000000];
        assert_eq!(encode_data(&data, &code_lookup), expected);
    }

    #[test]
    fn test_decode_data() {
        let code = "0001101011".to_string();
        let code_lookup = {
            let mut code_lookup = HashMap::new();
            code_lookup.insert('h', "00".to_string());
            code_lookup.insert('e', "01".to_string());
            code_lookup.insert('l', "10".to_string());
            code_lookup.insert('o', "11".to_string());
            code_lookup
        };
        let expected = "hello";
        assert_eq!(decode_data(code, code_lookup), expected);
    }
}
