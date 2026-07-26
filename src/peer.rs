// THIS IS FOR THE PEER COMMUNICATION LOGIC


use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::time::Duration;
use tokio::time::timeout;
use std::io::{Seek, SeekFrom, Read};
use crate::torrent::compute_dem_hash;
#[derive(Debug)]
pub enum PeerMessage {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    Notinterested,
    Have(u32),
    Bitfield(Vec<u8>),
    Request(u32, u32, u32),
    Piece(u32, u32, Vec<u8>),
}


pub fn build_handshake(info_hash: &[u8; 20], peer_id: &[u8; 20]) -> Vec<u8> {
    let mut hand_vector = Vec::new();
    hand_vector.push(19u8);
    hand_vector.extend_from_slice(b"BitTorrent protocol");
    hand_vector.extend_from_slice(&[0u8; 8]);    
    hand_vector.extend_from_slice(info_hash);
    hand_vector.extend_from_slice(peer_id);
    hand_vector
}

pub async fn read_message(stream: &mut TcpStream) -> Result<PeerMessage, String> {
    let mut len_buf = [0u8; 4];
     stream.read_exact(&mut len_buf).await.map_err(|e| e.to_string())?;
    let msg_len = u32::from_be_bytes(len_buf);

    if msg_len == 0 {
        return Ok(PeerMessage::KeepAlive);
    } else {
        let mut body = vec![0u8; msg_len as usize];
        stream.read_exact(&mut body).await.map_err(|e| e.to_string())?;
        match body[0] {
            0 => Ok(PeerMessage::Choke),
            1 => Ok(PeerMessage::Unchoke),
            2 => Ok(PeerMessage::Interested),
            3 => Ok(PeerMessage::Notinterested),
            4 => { let index = u32::from_be_bytes([body[1], body[2], body[3], body[4]]);
                    Ok(PeerMessage::Have(index))},
            5 => Ok(PeerMessage::Bitfield(body[1..].to_vec())),
           6 => { let index = u32::from_be_bytes([body[1], body[2], body[3], body[4]]);
                    let begin = u32::from_be_bytes([body[5], body[6], body[7], body[8]]);
                    let length = u32::from_be_bytes([body[9], body[10], body[11], body[12]]);
                    Ok(PeerMessage::Request(index, begin, length))},
           7 => {  let index = u32::from_be_bytes([body[1], body[2], body[3], body[4]]);
                   let begin = u32::from_be_bytes([body[5], body[6], body[7], body[8]]);
                   let data = body[9..].to_vec();
                   Ok(PeerMessage::Piece(index, begin, data))},
            _ => Err(format!("unknown message id: {}", body[0])),
        }
    }
}

pub async fn send_message(stream: &mut TcpStream, id: u8, payload: &[u8]) -> Result<(), String> {
    let mut msg = Vec::new();
    
    let len = 1 + payload.len() as u32; 
    let len_bytes = len.to_be_bytes(); 
    msg.extend_from_slice(&len_bytes);
    msg.push(id);
    msg.extend_from_slice(payload);
    stream.write_all(&msg).await.map_err(|e| e.to_string())?;
    
    Ok(())
}


pub fn build_elrequesto_payload(index_ofdatpiece: u32, begin: u32, length: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&index_ofdatpiece.to_be_bytes());
    payload.extend_from_slice(&begin.to_be_bytes());
    payload.extend_from_slice(&length.to_be_bytes());
    payload
}

pub async fn  connect_and_handshaker(
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
    
    send_message(&mut stream, 2, &[]).await?;

    loop {
        let msg = read_message(&mut stream).await?;

        if let PeerMessage::Unchoke = msg {
            break
        }
    }

    Ok(stream)
}


pub async fn download_elpiece(
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
                other => other
            };
        }
    }
    Ok(buffer_piercer)
}

pub fn are_we_there_yet(
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
