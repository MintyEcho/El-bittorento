//  use hex_literal::hex;
use sha1::{Sha1, Digest};
#[allow(dead_code)]
#[derive(Debug)]
enum BencodeValue {
    Int(i64),
    Bytes(Vec<u8>),
    List(Vec<BencodeValue>),
    Dict(Vec<(Vec<u8>, BencodeValue)>),
}
#[derive(Debug)]
struct TorrentMetainfo {
    announce: String,
    name: String,
    piece_length: i64,
    pieces: Vec<u8>,
    length: i64,
}

fn get_eldict_value<'a>(dict: &'a [(Vec<u8>, BencodeValue)], key: &str) -> Option<&'a BencodeValue> {
    dict.iter()
        .find(|(k, _)| k == key.as_bytes())
        .map(|(_, v)| v)
}

fn parse_eltorrento(value: &BencodeValue) -> Result<TorrentMetainfo, String> {
    let BencodeValue::Dict(top) = value else {
        return Err("gimme a dictionary in the top level fucking dumdum".to_string());
    };

    let announce = match get_eldict_value(top, "announce") {
        Some(BencodeValue::Bytes(b)) => String::from_utf8_lossy(b).to_string(),
        _ => return Err("i aint see no announce. or that shi might be ass'".to_string()),
    };

    let BencodeValue::Dict(info) = get_eldict_value(top, "info")
        .ok_or("no info in this dictionary".to_string())? else {
        return Err("no info. dumbass".to_string());
    };

    let name = match get_eldict_value(info, "name") {
        Some(BencodeValue::Bytes(b)) => String::from_utf8_lossy(b).to_string(),
        _ => return Err("aint no way you didnt gimme a name, dumbass'".to_string()),
    };

    let piece_length = match get_eldict_value(info, "piece length") {
        Some(BencodeValue::Int(n)) => *n,
        _ => return Err("no piece length *boowomp*'".to_string()),
    };

    let pieces = match get_eldict_value(info, "pieces") {
        Some(BencodeValue::Bytes(b)) => b.clone(),
        _ => return Err("no 'pieces' my homie".to_string()),
    };

    let length = match get_eldict_value(info, "length") {
        Some(BencodeValue::Int(n)) => *n,
        _ => return Err("no 'length'".to_string()),
    };

    Ok(TorrentMetainfo { announce, name, piece_length, pieces, length })
}



fn infoget(input: &[u8]) -> Result<(Vec<u8>, usize), String> {
    let mut position = 1 ;// we still skipping the big d bro 
    let mut items = Vec::new();
    while input[position] != b'e' {
        let (key, consumed_key) = stringiparser(&input[position..])?;
        position += consumed_key;

        if key == b"info" {
        let info_start = position;
        let (_, consumed_val) = parsdat(&input[position..])?;
        let info_end = info_start + consumed_val;
        return Ok((input[info_start..info_end].to_vec(), consumed_key));
        } else {
        let (value, consumed_val) = parsdat(&input[position..])?;
        position += consumed_val;

        items.push((key, value));
        }
    }
    Err("couldnt find el big info".to_string())

}






fn intiparser(input: &[u8]) -> Result<(i64, usize), String> {
    //to see if that is parsable or not 
    if input.len() < 3 || input[0] != b'i' {
        return Err("That must start with an i and be bigger than three bytes".to_string());
    }
    let e_positione = input.iter().position(|&b| b == b'e')
    .ok_or("El terminator e is missing big dawg".to_string())?;
    let digit = &input[1..e_positione];
    let number_string = std::str::from_utf8(digit).map_err(|e| e.to_string())?;
    let number = number_string.parse::<i64>().map_err(|e| e.to_string())?;
    Ok((number, e_positione + 1))
}
fn stringiparser(input: &[u8]) -> Result<(Vec<u8>, usize), String> {
    if input.len() < 3 {
        return Err("yo that's too short bro".to_string());
    }

    let colon_position = input.iter().position(|&b| b == b':')
        .ok_or("El terminator ':' is missing big dawg".to_string())?;

    let length_bytes = &input[..colon_position];
    let length_str = std::str::from_utf8(length_bytes).map_err(|e| e.to_string())?;
    let length: usize = length_str.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;

    let string_start = colon_position + 1;
    let string_end = string_start + length;

    if string_end > input.len() {
        return Err("string length exceeds available bytes".to_string());
    }

    let string_bytes = input[string_start..string_end].to_vec();
    let total_consumed = string_end;

    Ok((string_bytes, total_consumed))
}

fn listiparser(input: &[u8]) -> Result<(Vec<BencodeValue>, usize), String> {
    let mut position = 1; // skip the leading 'l' at index 0
    let mut items = Vec::new();

    while input[position] != b'e' {
        let (value, consumed) = parsdat(&input[position..])?;
        items.push(value);
        position += consumed;
    }

    // including the left out e
    Ok((items, position + 1))
}

fn distiparser(input: &[u8]) -> Result<(Vec<(Vec<u8>, BencodeValue)>, usize), String> {
    let mut position = 1; // skip the leading 'd'
    let mut items = Vec::new();

    while input[position] != b'e' {
        // every value or sumn must be a string
        let (key, consumed_key) = stringiparser(&input[position..])?;
        position += consumed_key;

        // key must have value after it right? right? right? right? right? right? right? right? 
        let (value, consumed_val) = parsdat(&input[position..])?;
        position += consumed_val;

        items.push((key, value));
    }

    Ok((items, position + 1))
}



fn parsdat(input: &[u8]) -> Result<(BencodeValue, usize), String> {
    match input.first() {
        Some(b'i') => {
            let (value, consumed) = intiparser(input)?;
            Ok((BencodeValue::Int(value), consumed))
        }
        Some(b'l') => {
            let (value, consumed) = listiparser(input)?;
            Ok((BencodeValue::List(value), consumed))
        }
        Some(b'0'..=b'9') => {
            let (value, consumed) = stringiparser(input)?;
            Ok((BencodeValue::Bytes(value), consumed))
        }
        Some(b'd') =>{
            let (value, consumed) = distiparser(input)?;
            Ok((BencodeValue::Dict(value), consumed))
        }
        _ => Err("unknown or unsupported bencode type, dumbass".to_string()),
    }
}


fn compute_dem_hash(input: &[u8]) -> [u8; 20]{
    let mut hashment = Sha1::new();
    hashment.update(input);
    let hashed = hashment.finalize();
    hashed.into()
}

fn main() {
    let bytes = std::fs::read("test.torrent").expect("failed to read file");

    let info_bytes = infoget(&bytes).expect("couldn't find info dict");

    let (bitos, consumedbitos) = info_bytes;

    let hashed_info = compute_dem_hash(&bitos);

    println!("check this hash!!: {}", hashed_info.iter().map(|b| format!("{:02x}", b)).collect::<String>());
    // compute_dem_hash(&bytes);
}