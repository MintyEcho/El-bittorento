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
    let addr_str = format!("{}:{}", ip, port);
    println!("Trying peer: {addr_str}");

    let mut stream = match TcpStream::connect(&addr_str).await {
        Ok(s) => s,
        Err(_) => continue,
    };

    let handshake = build_handshake(&hashed_info, peer_id);
    if stream.write_all(&handshake).await.is_err() {
        continue;
    }

    let mut response = [0u8; 68];
    if stream.read_exact(&mut response).await.is_err() {
        continue;
    }

    let peer_info_hash = &response[28..48];
    if peer_info_hash != hashed_info {
        eprintln!("Info hash mismatch! Dropping peer.");
        continue;
    }

    println!("Handshake successful with {addr_str}!");

    let msg = match read_message(&mut stream).await {
        Ok(m) => m,
        Err(_) => continue,
    };
    println!("{:?}", msg);
   send_message(&mut stream, 2, &[]).await?; 
loop {
    let msg = read_message(&mut stream).await?;
    println!("{:?}", msg);
    if let PeerMessage::Unchoke = msg {
        break;
    }
}
println!("Peer unchoked us!");

let piece_length = metainfo.piece_length as u32;
let block_size: u32 = 16384;
let numnum_blocks = (piece_length + block_size -1) / block_size;
let mut buffer_piercer = vec![0u8; piece_length as usize];

for block_index in 0..numnum_blocks {
    let begin = block_index * block_size;
    let remaining = piece_length - begin;
    let length = remaining.min(block_size);
    // going crazy i have no idea what im doing but im doing it
    // #no_ai_written_frfrrffrfrfr
    let payload = build_elrequesto_payload(0, begin, length);
    send_message(&mut stream, 6, &payload).await?;

    loop {
        let msg = read_message(&mut stream).await?;
        match msg {
            PeerMessage::Piece(_index, msg_begin, data) => {
                buffer_piercer[msg_begin as usize .. msg_begin as usize + data.len()]
                .copy_from_slice(&data);
                break;
            }
            other => println!("ignoring: {:?}", other),
        }
    }
}
let hash_elcomputed = compute_dem_hash(&buffer_piercer);
let hash_elexpected = &metainfo.pieces[0..20];
if hash_elcomputed == hash_elexpected {
    println!("ONG WE DID THAT SHI, FIRST PIECE VERIFIED BAYBEEEE")
} else {
    println!("yo the hashes aint right revise yo shi")
}
break;
}
Ok(())
}