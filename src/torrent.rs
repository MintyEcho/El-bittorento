// THIS IS FOR THE TORRENT READING FUNCTIONS AND WHATNOT
use crate::bencode::{BencodeValue, parsdat, get_eldict_value, stringiparser};
use sha1::{Sha1, Digest};
#[derive(Debug)]
//meta info n shiii
pub struct TorrentMetainfo {
    pub announce: String,
    pub name: String,
    pub piece_length: i64,
    pub pieces: Vec<u8>,
    pub length: i64,
}


pub fn parse_eltorrento(value: &BencodeValue) -> Result<TorrentMetainfo, String> {
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
pub fn infoget(input: &[u8]) -> Result<(Vec<u8>, usize), String> {
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


// this is for internety reasons or whatever. we just cant give the guy a list of numbers
pub fn compute_dem_hash(input: &[u8]) -> [u8; 20]{
    let mut hashment = Sha1::new();
    hashment.update(input);
    let hashed = hashment.finalize();
    hashed.into()
}