//  use hex_literal::hex;
// idk what to do with the above comment. it was there in the documents
// but like it was dead code. so i left it here just in case ill have to use it somehow
use sha1::{Sha1, Digest};
#[allow(dead_code)]
#[derive(Debug)]
// making dem enums and structs yehehehhehehe
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
//this is for the other function. please refer to line 29 and read the goddamn code
fn get_eldict_value<'a>(dict: &'a [(Vec<u8>, BencodeValue)], key: &str) -> Option<&'a BencodeValue> {
    dict.iter()
        .find(|(k, _)| k == key.as_bytes())
        .map(|(_, v)| v)
}
//this is for future use
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


//we gotta get that info hex no?
fn infoget(input: &[u8]) -> Result<(Vec<u8>, usize), String> {
    let mut position = 1 ;// we still skipping the big d bro 
    let mut items = Vec::new();
    while input[position] != b'e' {
        let (key, consumed_key) = stringiparser(&input[position..])?;
        position += consumed_key;
        //if else fuck arounds to find out the info. if its not there execute the user publicly
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


//those bytes aint gonna be good so we do them in percent
fn urlencode_bytes(input: &[u8]) -> String {
    let mut encoded = String::new();
    for &byte in input {
        if byte.is_ascii_alphanumeric(){
            encoded.push(byte as char);
        } else if byte == b'-' || byte == b'_' || byte == b'.' || byte == b'~' {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{:02X}", byte));
        }
    }
    encoded
}


fn intiparser(input: &[u8]) -> Result<(i64, usize), String> {
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
fn stringiparser(input: &[u8]) -> Result<(Vec<u8>, usize), String> {
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

fn listiparser(input: &[u8]) -> Result<(Vec<BencodeValue>, usize), String> {
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

fn distiparser(input: &[u8]) -> Result<(Vec<(Vec<u8>, BencodeValue)>, usize), String> {
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



fn parsdat(input: &[u8]) -> Result<(BencodeValue, usize), String> {
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

// this is for internety reasons or whatever. we just cant give the guy a list of numbers
fn compute_dem_hash(input: &[u8]) -> [u8; 20]{
    let mut hashment = Sha1::new();
    hashment.update(input);
    let hashed = hashment.finalize();
    hashed.into()
}


#[tokio::main]
async fn main() ->  Result<(), Box<dyn std::error::Error>> {
let bytes = std::fs::read("test.torrent").expect("failed to read file");

// first of all. i am vegeta.
let (info_bytes, _) = infoget(&bytes)?; 
let hashed_info = compute_dem_hash(&info_bytes);
let encoded_hash = urlencode_bytes(&hashed_info);

// second of all. you're not vegeta
let (funny_ben, _) = parsdat(&bytes)?;
let metainfo = parse_eltorrento(&funny_ben)?;  

// third of all. you wanna be vegeta
let peer_id = b"mintos69helloworldya";
let encoded_peer = urlencode_bytes(peer_id);

// but you cant be vegeta
let client = reqwest::Client::new();
let url = format!(
    "{}?info_hash={}&peer_id={}&port=6881&uploaded=0&downloaded=0&left={}&compact=1",
    metainfo.announce,
    encoded_hash,
    encoded_peer,
    metainfo.length
);

//because im vegeta
let res = client.get(url).send().await?.bytes().await?;
let parsed_res = parsdat(&res);
println!("{:?}", parsed_res);
Ok(())
}