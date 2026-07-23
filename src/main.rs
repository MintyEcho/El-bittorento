// MAIN FUNCTION N SHII

//calling all the modules
mod bencode;
mod torrent;
mod tracker;

use bencode::{BencodeValue, parsdat, get_eldict_value};
use torrent::{TorrentMetainfo, parse_eltorrento, infoget, compute_dem_hash};
use tracker::{urlencode_bytes, get_peers};
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
let peers = get_peers(&url).await?;
for (ip, port) in &peers {
    println!("{}:{}", ip, port);
}
Ok(())
}