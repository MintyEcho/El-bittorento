use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};

use crate::torrent::{TorrentMetainfo, compute_dem_hash};
use crate::peer::{connect_and_handshaker, download_elpiece};

pub async fn download_eltorrento(
    peers: Vec<(String, u16)>,
    metainfoe: Arc<TorrentMetainfo>,
    hashed_infoe: [u8; 20],
    peer_id: [u8; 20],
    output_file: Arc<Mutex<File>>,
    remaining_pisces: Vec<usize>,
    max_peers_arrasametime: usize,
) {
    let kyuu = Arc::new(Mutex::new(remaining_pisces));
    let mut handles = vec![];

    let peers_to_use = peers.into_iter().take(max_peers_arrasametime);

    for (ip, port) in peers_to_use {
        let kyuu = Arc::clone(&kyuu);
        let metainfoe = Arc::clone(&metainfoe);
        let output_file = Arc::clone(&output_file);

        let handle = tokio::spawn(async move {
            let addr_str = format!("{}:{}", ip, port);
            println!("alr so we tryin peer: {addr_str}");

            let mut stream = match connect_and_handshaker(&addr_str, &hashed_infoe, &peer_id).await {
                Ok(s) => s,
                Err(e) => {
                    println!("alr we're skipping peer {addr_str}, cuz of {e}");
                    return;
                }
            };

            loop {
                let piece_indeks = {
                    let mut qyuyu = kyuu.lock().unwrap();
                    qyuyu.pop()
                };

                let piece_indeks = match piece_indeks {
                    Some(i) => i,
                    None => break,
                };

                let piece_length = metainfoe.piece_length as u32;
                let total_length = metainfoe.length as u64;
                let piece_start = piece_indeks as u64 * piece_length as u64;
                let remaining_total = total_length - piece_start;
                let dis_piece_length = remaining_total.min(piece_length as u64) as u32;
                let hash_elexpected = &metainfoe.pieces[piece_indeks*20 .. piece_indeks*20+20];

                let buffer = match download_elpiece(&mut stream, piece_indeks as u32, dis_piece_length).await {
                    Ok(buffa) => buffa,
                    Err(e) => {
                        println!("alr so piece {piece_indeks} failed cuz {e}. we gonna try it again later");
                        kyuu.lock().unwrap().push(piece_indeks);
                        continue;
                    }
                };

                let computed = compute_dem_hash(&buffer);
                if computed != hash_elexpected {
                    println!("Piece {piece_indeks} got the wrong hash bro!");
                    kyuu.lock().unwrap().push(piece_indeks);
                    continue;
                }

                {
                    let mut file = output_file.lock().unwrap();
                    file.seek(SeekFrom::Start(piece_start)).unwrap();
                    file.write_all(&buffer).unwrap();
                }

                println!("alr piece {piece_indeks} is done verified and written frfr");
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
}