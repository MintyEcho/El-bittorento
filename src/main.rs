// MAIN FUNCTION N SHII

//calling all the modules
mod bencode;
mod torrent;
mod tracker;
mod peer;
mod download;
use bencode::{BencodeValue, parsdat, get_eldict_value};
use torrent::{TorrentMetainfo, parse_eltorrento, infoget, compute_dem_hash};
use tracker::{urlencode_bytes, get_peers};
use peer::{PeerMessage, build_handshake, read_message, send_message, build_elrequesto_payload, connect_and_handshaker,download_elpiece,are_we_there_yet};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write, Read};
use download::download_eltorrento;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};



#[tokio::main]
async fn main() ->  Result<(), Box<dyn std::error::Error>> {
let args: Vec<String> = std::env::args().collect();
if args.len() < 3  {
    eprintln!("how to use: {} <path_to_torrent> <where you want the file to be>", args[0]);
    std::process::exit(1);
}
let torrent_path = &args[1];
let output_folder = &args[2];
std::fs::create_dir_all(output_folder);


let bytes = std::fs::read(torrent_path).expect("failed to read file");

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

let output_path = std::path::Path::new(output_folder).join(&metainfo.name);
let mut output_file = OpenOptions::new()
    .create(true)
    .read(true)
    .write(true)
    .open(&output_path)?;
//because im vegeta
let peers = get_peers(&url).await?;
let piece_length = metainfo.piece_length as u32;
let total_length = metainfo.length as u64;
let num_pieces = metainfo.pieces.len() / 20;

// figure out which pieces are still needed, using your existing check
let mut remaining_pieces = Vec::new();
for piece_index in 0..num_pieces {
    let piece_start = piece_index as u64 * piece_length as u64;
    let remaining_total = total_length - piece_start;
    let this_piece_length = remaining_total.min(piece_length as u64) as u32;
    let expected_hash = &metainfo.pieces[piece_index*20 .. piece_index*20+20];

    if !are_we_there_yet(&mut output_file, piece_start, this_piece_length, expected_hash) {
        remaining_pieces.push(piece_index);
    }
}
println!("{} of {} pieces remaining", remaining_pieces.len(), num_pieces);

let metainfo_arc = Arc::new(metainfo);
let output_file_arc = Arc::new(Mutex::new(output_file));
let completed_pieces = Arc::new(AtomicUsize::new(0));

download_eltorrento(
    peers,
    metainfo_arc,
    hashed_info,
    *peer_id,
    output_file_arc,
    remaining_pieces,
    20, 
    completed_pieces,
    num_pieces,
).await;
println!("we done frfr")
Ok(())
}