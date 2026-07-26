use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use tokio::time::{timeout, Duration};
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
    let peer_pool = Arc::new(Mutex::new(peers));
    let mut handles = vec![];

    for _ in 0..max_peers_arrasametime {
        let kyuu = Arc::clone(&kyuu);
        let peer_pool = Arc::clone(&peer_pool);
        let metainfoe = Arc::clone(&metainfoe);
        let output_file = Arc::clone(&output_file);

        let handle = tokio::spawn(async move {
            'peer_loop: loop {
                //so we make it so the shi grabs a fresh address from the gene pool
                let next_pourus =  {
                    let mut pool = peer_pool.lock().unwrap();
                    pool.pop()
                };
                let (ip, port) = match next_pourus {
                    Some(p) => p,
                    None => break 'peer_loop,
                };
                let addr_str = format!("{}:{}", ip, port);
                println!("alr so we trying {}", addr_str);

                let mut stream = match connect_and_handshaker(&addr_str, &hashed_infoe, &peer_id).await {
                    Ok(s) => s,
                    Err(e) => {
                        println!("alr we're skipping peer {addr_str} cuh of {e}");
                        continue 'peer_loop; //yk look for another address
                    }
                };

                loop {
                    let piece_indeks = {
                        let mut qyuyu = kyuu.lock().unwrap();
                        qyuyu.pop()
                    };
                    let piece_indeks = match piece_indeks {
                        Some(i) => i,
                        None => return, //all pieces done, so worker just die
                    };

                    let piece_length = metainfoe.piece_length as u32;
                    let total_length = metainfoe.length as u64;
                    let piece_start = piece_indeks as u64 * piece_length as u64;
                    let remaining_total = total_length - piece_start;
                    let dis_piece_length = remaining_total.min(piece_length as u64) as u32;
                    let hash_elexpected = &metainfoe.pieces[piece_indeks*20 .. piece_indeks*20+20];

                    let buffer = match tokio::time::timeout(
                        std::time::Duration::from_secs(15),
                        download_elpiece(&mut stream, piece_indeks as u32, dis_piece_length)
                    ).await {
                        Ok(Ok(buffa)) => buffa,
                        Ok(Err(e)) => {
                            println!("piece {piece_indeks} failed cuz {e}, dropping this connection");
                            kyuu.lock().unwrap().insert(0, piece_indeks);
                            continue 'peer_loop;
                    }
                     Err(_) => {
                            println!("piece {piece_indeks} timed out, dropping this connection");
                            kyuu.lock().unwrap().insert(0, piece_indeks);
                            continue 'peer_loop;
                        }
                };

                let computed = compute_dem_hash(&buffer);
                    if computed != hash_elexpected {
                        println!("Piece {piece_indeks} got the wrong hash bro!");
                        kyuu.lock().unwrap().insert(0, piece_indeks);
                        continue 'peer_loop;
                    }

                    {
                        let mut file = output_file.lock().unwrap();
                        file.seek(SeekFrom::Start(piece_start)).unwrap();
                        file.write_all(&buffer).unwrap();
                    }

                    println!("alr piece {piece_indeks} is done verified and written frfr (via {addr_str})");
                }
            }
        });
         handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
}