//THIS IS FOR THE ONLINE TRACKER FUNCTIONS AND BLEH BLEH

use crate::bencode::{BencodeValue, parsdat, get_eldict_value, stringiparser};
use crate::torrent::{TorrentMetainfo, parse_eltorrento, infoget, compute_dem_hash};

//those bytes aint gonna be good so we do them in percent encoding
pub fn urlencode_bytes(input: &[u8]) -> String {
    let mut encoded = String::new();
    for &byte in input {
        //this is a thing about safe characters, where it just passes safe characters "ASCII"
        if byte.is_ascii_alphanumeric(){
            encoded.push(byte as char);
            //those arent included in the ascii but still safe
        } else if byte == b'-' || byte == b'_' || byte == b'.' || byte == b'~' {
            encoded.push(byte as char);
        } else {
            //and formats it a way if its not of those safe charcters
            encoded.push_str(&format!("%{:02X}", byte));
        }
    }
    encoded
}

//this is the peer reach out function. we make a simple reqwest client and just send a normal
//get request to the pre-made URL "refer to main to see how we form it". 
pub async fn get_peers(url: &str) -> Result<Vec<(String, u16)>, String> {
    let client = reqwest::Client::new();

    let res = client.get(url).send().await.map_err(|e| e.to_string())?
        .bytes().await.map_err(|e| e.to_string())?;
    
    // since its a really big dictionary and we only want the first dictionary inside,
    //so we just extract it and we can assign the second value to an empty variable using "_"
    // extremely important and you will use it alot.
    let (response_dict, _) = parsdat(&res)?;

    let BencodeValue::Dict(top) = response_dict else {
        return Err("aint no way the tracker didnt give us a dict what?".to_string());
    };
    
    //extracting the peers from the get request.
    let peer_bytes = match get_eldict_value(&top, "peers") {
        Some(BencodeValue::Bytes(b)) => b.clone(),
        _ => return Err("no peers, dumbass".to_string()),
    };

    let mut peers = Vec::new();
    //this loop is to extract the IP addresses and the ports into something readable and workable
    for chonk in peer_bytes.chunks(6) {
        let ip = format!("{}.{}.{}.{}", chonk[0], chonk[1], chonk[2], chonk[3]);
        let port: u16 = (chonk[4] as u16) * 256 + (chonk[5] as u16);
        peers.push((ip, port));
    }

    Ok(peers)
}