//   THIS IS FOR THE PARSER FUNCITONS

#[derive(Debug)]
// making dem enums and structs yehehehhehehe
pub enum BencodeValue {
    Int(i64),
    Bytes(Vec<u8>),
    List(Vec<BencodeValue>),
    Dict(Vec<(Vec<u8>, BencodeValue)>),
}



//this is for the other function in torrent.rs. please go and read the goddamn code
pub fn get_eldict_value<'a>(dict: &'a [(Vec<u8>, BencodeValue)], key: &str) -> Option<&'a BencodeValue> {
    dict.iter()
        .find(|(k, _)| k == key.as_bytes())
        .map(|(_, v)| v)
}


pub fn intiparser(input: &[u8]) -> Result<(i64, usize), String> {
    //to see if that is parsable or not 
    if input.len() < 3 || input[0] != b'i' {
        return Err("That must start with an i and be bigger than three bytes".to_string());
    }
    let e_positione = input.iter().position(|&b| b == b'e')
    .ok_or("El terminator e is missing big dawg".to_string())?;
    //bunch of fuck arounds to find out the integer
    let digit = &input[1..e_positione];
    let number_string = std::str::from_utf8(digit).map_err(|e| e.to_string())?;
    let number = number_string.parse::<i64>().map_err(|e| e.to_string())?;
    Ok((number, e_positione + 1))
}
pub fn stringiparser(input: &[u8]) -> Result<(Vec<u8>, usize), String> {
    //idk what kinda dumbass will send me a less than 3 bytes length thing but we gotta do
    // what we gotta do yk?
    if input.len() < 3 {
        return Err("yo that's too short bro".to_string());
    }

    let colon_position = input.iter().position(|&b| b == b':')
        .ok_or("El terminator ':' is missing big dawg".to_string())?;
    // another bunch of fuck arounds to find out but this time the string
    let length_bytes = &input[..colon_position];
    let length_str = std::str::from_utf8(length_bytes).map_err(|e| e.to_string())?;
    let length: usize = length_str.parse().map_err(|e: std::num::ParseIntError| e.to_string())?;

    let string_start = colon_position + 1;
    let string_end = string_start + length;
    // this is if we ever get a bigahh string somehow
    if string_end > input.len() {
        return Err("string length exceeds available bytes".to_string());
    }

    let string_bytes = input[string_start..string_end].to_vec();
    let total_consumed = string_end;

    Ok((string_bytes, total_consumed))
}

pub fn listiparser(input: &[u8]) -> Result<(Vec<BencodeValue>, usize), String> {
    let mut position = 1; // skip that l cuz we never take those
    let mut items = Vec::new();

    while input[position] != b'e' {
        let (value, consumed) = parsdat(&input[position..])?;
        items.push(value);
        position += consumed;
    }

    // including the left out e
    Ok((items, position + 1))
}

pub fn distiparser(input: &[u8]) -> Result<(Vec<(Vec<u8>, BencodeValue)>, usize), String> {
    let mut position = 1; // skip the leading d we like women more
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



pub fn parsdat(input: &[u8]) -> Result<(BencodeValue, usize), String> {
    //match just for the fact this can nest in itself so bad so just 
    //call itself a hundred times especially if its distiparser that shi
    //gets called alot
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
