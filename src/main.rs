// MAIN FUNCTION N SHII

//calling all the modules
mod bencode;
mod torrent;
mod tracker;
mod peer;
use bencode::{BencodeValue, parsdat, get_eldict_value};
use torrent::{TorrentMetainfo, parse_eltorrento, infoget, compute_dem_hash};
use tracker::{urlencode_bytes, get_peers};
use peer::{PeerMessage, build_handshake, read_message, send_message, build_elrequesto_payload};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write, Read};


async fn  connect_and_handshaker(
   addr_str: &str,
   hashed_info: &[u8; 20],
   peer_id: &[u8; 20],
) -> Result<TcpStream, String> {
    let mut stream = timeout(Duration::from_secs(5), TcpStream::connect(addr_str))
    .await
    .map_err(|_| "Connection timed out".to_string())?
    .map_err(|e| e.to_string())?;

    let handshake = build_handshake(hashed_info, peer_id);
    stream.write_all(&handshake).await.map_err(|e| e.to_string())?;

    let mut response = [0u8; 68];
    stream.read_exact(&mut response).await.map_err(|e| e.to_string())?;

    let peer_info_hash = &response[28..48];
    if peer_info_hash != hashed_info {
        return Err("info hash mismatch".to_string());
    }
    let msg = read_message(&mut stream).await?;
    println!("{:?}", msg);
    
    send_message(&mut stream, 2, &[]).await?;

    loop {
        let msg = read_message(&mut stream).await?;
        println!("{:?}", msg);
        if let PeerMessage::Unchoke = msg {
            break
        }
    }

    Ok(stream)
}


async fn download_elpiece(
    stream: &mut TcpStream,
    piece_index: u32,
    dis_piece_length: u32,
) -> Result<Vec<u8>, String> {
    let block_size: u32 = 16384;
    let numnum_blocks = (dis_piece_length + block_size -1) / block_size;
    let mut buffer_piercer = vec![0u8; dis_piece_length as usize];

    for block_index in 0..numnum_blocks {
        let begin = block_index * block_size;
        let remaining = dis_piece_length - begin;
        let length = remaining.min(block_size);

        let payload = build_elrequesto_payload(piece_index, begin, length);
        send_message(stream, 6, &payload).await?;

        loop {
            let msg = read_message(stream).await?;
            match msg {
                PeerMessage::Piece(_index, msg_begin, data) => {
                    buffer_piercer[msg_begin as usize..msg_begin as usize + data.len()]
                    .copy_from_slice(&data);

                    break;
                }
                other => println!("rn we just ignored: {:?}", other),
            }
        }
    }
    Ok(buffer_piercer)
}

fn are_we_there_yet(
    output_file: &mut std::fs::File,
    piece_start: u64,
    this_piece_length: u32,
    hash_elexpected: &[u8],
) -> bool {
    let mut buffer = vec![0u8; this_piece_length as usize];
    if output_file.seek(SeekFrom::Start(piece_start)).is_err() {
        return false;
    }
    match output_file.read_exact(&mut buffer) {
        Ok(_) => compute_dem_hash(&buffer) == hash_elexpected,
        Err(_) => false,
    }
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
let mut output_file = OpenOptions::new()
    .create(true)
    .read(true)
    .write(true)
    .open(&metainfo.name)?;
//because im vegeta
let peers = get_peers(&url).await?;
for (ip, port) in &peers {
    let addr_str = format!("{}:{}", ip, port);
    println!("Trying peer: {addr_str}");

    let mut stream = match connect_and_handshaker(&addr_str, &hashed_info, peer_id).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping peer: {e}");
            continue;
        }
    };

    println!("Peer unchoked us!");

let total_length = metainfo.length as u64;
let piece_length = metainfo.piece_length as u32;
let num_pieces = metainfo.pieces.len() / 20;
let block_size: u32 = 16384;

for piece_index in 0..num_pieces {
    let piece_start = piece_index as u64 * piece_length as u64;
    let remaining_total = total_length - piece_start;
    let this_piece_length = remaining_total.min(piece_length as u64) as u32;
    let hash_elexpected = &metainfo.pieces[piece_index*20 .. piece_index*20+20];

    if are_we_there_yet(&mut output_file, piece_start, this_piece_length, hash_elexpected) {
        println!("yo looks like you already have piece {}, imma skip it", piece_index);
        continue;
    }

    let buffer_piercer = match download_elpiece(&mut stream, piece_index as u32, this_piece_length).await {
        Ok(m) => m,
        Err(e) => {
            println!("sorry bro i couldnt download the piece {}:{}", piece_index, 0);
            break;
        }
    };
    let hash_elcomputed = compute_dem_hash(&buffer_piercer);
    if hash_elcomputed != hash_elexpected {
        println!("yo piece {} mismatch, initiating slur sequence.", piece_index);
        break;
    }
    output_file.seek(SeekFrom::Start(piece_start))?;
    output_file.write_all(&buffer_piercer)?;
    println!("Piece {} verified and written ({}/{})", piece_index, piece_index + 1, num_pieces);
}
break;
}
Ok(())
}