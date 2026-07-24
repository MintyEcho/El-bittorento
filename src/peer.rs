// THIS IS FOR THE PEER COMMUNICATION LOGIC


use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
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