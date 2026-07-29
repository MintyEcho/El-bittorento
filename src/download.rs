use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use tokio::time::{timeout, Duration};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::torrent::{TorrentMetainfo, compute_dem_hash};
use crate::peer::{connect_and_handshaker, download_elpiece};

//funny thing for user and stuff hehe so funny sudo pacman im so arch linux i need me a fedora execute me
pub fn print_progress_bar(done: usize, total: usize) {
    let bar_width = 30;
    let filled = (done * bar_width) / total.max(1);
    let empty = bar_width - filled;

    let percent = (done * 100) / total.max(1);

    print!(
        "\r[{}{}] {}% ({}/{})",
        "#".repeat(filled),
        "-".repeat(empty),
        percent,
        done,
        total
    );
    std::io::stdout().flush().unwrap();
}

//genuine BEHEMOTH OF A FUNCTION. but we gon explain it because its genuinley important...
//even though im gonna rewrite a big part of it for the multi file downloads sake. but we ballin


//  IMPORTANT TO UNDERSTAND SINCE IT ISNT COMMON KNOWLEDGE:
// this function relies heavily on Arc and Mutex.
// Arc is a smart pointer system which allows sharing of data between multiple threads
// so basically all the workers can have access to the vector at the same time no problem.
//and mutex allow exclusive ownership of values inside. so we can lock a value from there so -
// only one worker has access to it. which basically takes out the ability of 2 workers reaching out
// to the same peer at the same time. hope you understand
pub async fn download_eltorrento(
    peers: Vec<(String, u16)>,
    metainfoe: Arc<TorrentMetainfo>,
    hashed_infoe: [u8; 20],
    peer_id: [u8; 20],
    output_file: Arc<Mutex<File>>,
    remaining_pisces: Vec<usize>,
    max_peers_arrasametime: usize,
    completed_pieces: Arc<AtomicUsize>,
    total_pieces: usize,
) {
    // kyuu here is basically a queue arch-mutex vector.
    let kyuu = Arc::new(Mutex::new(remaining_pisces));
    let peer_pool = Arc::new(Mutex::new(peers));
    let mut handles = vec![];
    //arrasametime frfr
    for _ in 0..max_peers_arrasametime {
        let kyuu = Arc::clone(&kyuu);
        let peer_pool = Arc::clone(&peer_pool);
        let metainfoe = Arc::clone(&metainfoe);
        let output_file = Arc::clone(&output_file);
        let completed_pieces = Arc::clone(&completed_pieces);

        let handle = tokio::spawn(async move {
            'peer_loop: loop {
                //so we make it so the shi grabs a fresh address from the gene pool
                let next_pourus =  {
                    let mut pool = peer_pool.lock().unwrap();
                    pool.pop()
                };
                let (ip, port) = match next_pourus {
                    Some(p) => p,
                    None => break 'peer_loop, //yeah we done
                };
                let addr_str = format!("{}:{}", ip, port);

                let mut stream = match connect_and_handshaker(&addr_str, &hashed_infoe, &peer_id).await {
                    Ok(s) => s,
                    Err(e) => {
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
                    //so...from previous commits...this...existed for alot...
                    // im too lazy to explain what it is though
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
                            kyuu.lock().unwrap().insert(0, piece_indeks);
                            continue 'peer_loop;
                    }
                     Err(_) => {
                            kyuu.lock().unwrap().insert(0, piece_indeks);
                            continue 'peer_loop;
                        }
                };

                let computed = compute_dem_hash(&buffer);
                    if computed != hash_elexpected {
                        kyuu.lock().unwrap().insert(0, piece_indeks);
                        continue 'peer_loop;
                    }

                    {
                        let mut file = output_file.lock().unwrap();
                        file.seek(SeekFrom::Start(piece_start)).unwrap();
                        file.write_all(&buffer).unwrap();
                    }

                   let done = completed_pieces.fetch_add(1, Ordering::SeqCst) + 1; // +1 since fetch_add returns the OLD value
                    print_progress_bar(done, total_pieces);
                }
            }
        });
         handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
}

//i have a love hate relationship with this file.