#[allow(dead_code)]
#[derive(Debug)]
enum BencodeValue {
    Int(i64),
    Bytes(Vec<u8>),
    List(Vec<BencodeValue>),
    Dict(Vec<(Vec<u8>, BencodeValue)>),
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
        _ => Err("unknown or unsupported bencode type".to_string()),
    }
}
fn main() {
    let result = intiparser(b"i42e");
    println!("{:?}", result);//i wonder

    let other_result = stringiparser(b"4:spam");
    println!("{:?}", other_result);

    let anotherotherresult = listiparser(b"l4:spami42ee");
    println!("{:?}", anotherotherresult);

    let anotherotherresulter = distiparser(b"d3:cow3:moo4:spam4:eggs6:numberi42ee");
    println!("{:?}", anotherotherresulter);

 let torrent = parsdat(br#"d8:announce41:http://bttracker.debian.org:6969/announce7:comment35:"Debian CD from cdimage.debian.org"13:creation datei1690028920e4:infod6:lengthi657457152e4:name31:debian-12.1.0-amd64-netinst.iso12:piece lengthi262144e6:pieces20:12345678901234567890ee"#);
    println!("{:?}", torrent)
}