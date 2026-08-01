// THIS FILE IS FOR DOWNLOAD CONTROLLER FUNCTIONS

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use sha1::{Sha1, Digest};
use crate::torrent::{TorrentMetainfo, FileInfo, compute_dem_hash};
use crate::peer::{connect_and_handshaker, download_elpiece};

// he he macro for debug shi
macro_rules! dprintln {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!($($arg)*);
        }
    }
}

// funny thing for user and stuff hehe so funny sudo pacman im so arch linux i need me a fedora execute me
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


pub struct FileManager {
    pub base_dir: PathBuf,
    pub files: Vec<FileInfo>,
    pub piece_length: u64,
    pub total_length: u64,
}

impl FileManager {
    pub fn new(base_dir: PathBuf, meta: &TorrentMetainfo) -> Self {
        Self {
            base_dir,
            files: meta.files.clone(),
            piece_length: meta.piece_length as u64,
            total_length: meta.total_length as u64,
        }
    }

    // creating all necessary directories for the path
    pub fn prepare_dose_directories(&self) -> Result<(), String> {
        for file_infoe in &self.files {
            let mut full_path = self.base_dir.clone();
            for segmentation in &file_infoe.path {
                full_path.push(segmentation);
            }
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("we failed to create the directory: {:?} cuz {}", parent, e))?;
            }
        }
        Ok(())
    }

    // funny resume check made for the fact that its multiple files yummers
    pub fn verify_dat_onepiece(&self, piece_index: usize, expected_hash: &[u8]) -> Result<bool, String> {
        let start_elglobal = (piece_index as u64) * self.piece_length;
        let remaining_total = self.total_length - start_elglobal;
        let piece_data_lenciaga = remaining_total.min(self.piece_length) as usize;

        let mut buffer_piercer = vec![0u8; piece_data_lenciaga];
        let mut elglobal_offset_asofrightnow = 0;

        for file_infoe in &self.files {
            let file_end_offsetti = elglobal_offset_asofrightnow + file_infoe.length as u64;
            let piece_end = start_elglobal + piece_data_lenciaga as u64;

            if start_elglobal < file_end_offsetti && piece_end > elglobal_offset_asofrightnow {
                let local_file_offset = if start_elglobal > elglobal_offset_asofrightnow {
                    start_elglobal - elglobal_offset_asofrightnow
                } else {
                    0
                };

                let data_start = if elglobal_offset_asofrightnow > start_elglobal {
                    (elglobal_offset_asofrightnow - start_elglobal) as usize
                } else {
                    0
                };

                let data_end = if piece_end > file_end_offsetti {
                    (file_end_offsetti - start_elglobal) as usize
                } else {
                    piece_data_lenciaga
                };

                let mut full_path = self.base_dir.clone();
                for segment in &file_infoe.path {
                    full_path.push(segment);
                }

                let mut file = File::open(&full_path)
                    .map_err(|e| format!("i cant open {:?} for verify: {}", full_path, e))?;
                
                file.seek(SeekFrom::Start(local_file_offset))
                    .map_err(|e| format!("so the seek failed in {:?}: {}", full_path, e))?;

                file.read_exact(&mut buffer_piercer[data_start..data_end])
                    .map_err(|e| format!("so the read failed in {:?}: {}", full_path, e))?;
            }
            elglobal_offset_asofrightnow = file_end_offsetti;
        }
        
        let mut hasher = Sha1::new();
        hasher.update(&buffer_piercer);
        let actual_hash = hasher.finalize();

        Ok(actual_hash.as_slice() == expected_hash)
    }

    pub fn write_piece(&self, piece_index: usize, piece_data: &[u8]) -> Result<(), String> {
        let start_elglobal = (piece_index as u64) * self.piece_length;
        let piece_data_lenciaga = piece_data.len() as u64;
        let mut elglobal_offset_asofrightnow = 0;

        for file_infoe in &self.files {
            let file_end_offsetti = elglobal_offset_asofrightnow + file_infoe.length as u64;
            let piece_end = start_elglobal + piece_data_lenciaga;

            if start_elglobal < file_end_offsetti && piece_end > elglobal_offset_asofrightnow {
                let ellocal_offset_asofrightnow = if start_elglobal > elglobal_offset_asofrightnow {
                    start_elglobal - elglobal_offset_asofrightnow
                } else {
                    0
                };
                
                let data_start = if elglobal_offset_asofrightnow > start_elglobal {
                    (elglobal_offset_asofrightnow - start_elglobal) as usize
                } else {
                    0
                };

                let data_end = if piece_end > file_end_offsetti {
                    (file_end_offsetti - start_elglobal) as usize
                } else {
                    piece_data.len()
                };

                let slice_to_write = &piece_data[data_start..data_end];

                let mut full_path = self.base_dir.clone();
                for segment in &file_infoe.path {
                    full_path.push(segment);
                }

                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(&full_path)
                    .map_err(|e| format!("Failed to open {:?} for write: {}", full_path, e))?;

                file.seek(SeekFrom::Start(ellocal_offset_asofrightnow))
                    .map_err(|e| format!("Seek failed in {:?}: {}", full_path, e))?;

                file.write_all(slice_to_write)
                    .map_err(|e| format!("Write failed in {:?}: {}", full_path, e))?;
            }
            elglobal_offset_asofrightnow = file_end_offsetti;
        }
        Ok(())
    }
}


pub async fn download_eltorrento(
    peers: Vec<(String, u16)>,
    metainfoe: Arc<TorrentMetainfo>,
    hashed_infoe: [u8; 20],
    peer_id: [u8; 20],
    file_manager: Arc<FileManager>,
    remaining_pisces: Vec<usize>,
    max_peers_arrasametime: usize,
    completed_pieces: Arc<AtomicUsize>,
    total_pieces: usize,
) {
    let kyuu = Arc::new(Mutex::new(remaining_pisces));
    let peer_pool = Arc::new(Mutex::new(peers));
    let mut handles = vec![];

    for _ in 0..max_peers_arrasametime {
        let kyuu = Arc::clone(&kyuu);
        let peer_pool = Arc::clone(&peer_pool);
        let metainfoe = Arc::clone(&metainfoe);
        let file_manager = Arc::clone(&file_manager);
        let completed_pieces = Arc::clone(&completed_pieces);

        let handle = tokio::spawn(async move {
            'peer_loop: loop {
                let work_left = {
                    let q = kyuu.lock().unwrap();
                    !q.is_empty()
                };
                
                if !work_left {
                    dprintln!("\nDEBUG: All pieces downloaded! Worker exiting happily.");
                    break 'peer_loop;
                }

                let next_pourus = {
                    let mut pool = peer_pool.lock().unwrap();
                    pool.pop()
                };

                let (ip, port) = match next_pourus {
                    Some(p) => p,
                    None => {
                        dprintln!("\nDEBUG: Peer pool is empty. Waiting for peers to free up...");
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue 'peer_loop;
                    }
                };
                
                let addr_str = format!("{}:{}", ip, port);

                let mut stream = match tokio::time::timeout(
                    std::time::Duration::from_secs(5), 
                    connect_and_handshaker(&addr_str, &hashed_infoe, &peer_id)
                ).await {
                    Ok(Ok(s)) => {
                        dprintln!("\nDEBUG: Successfully connected to {}!", addr_str);
                        s
                    }, 
                    Ok(Err(e)) => {
                        dprintln!("\nDEBUG: Rejected peer {}: {}", addr_str, e);
                        continue 'peer_loop; 
                    },
                    Err(_) => {
                        dprintln!("\nDEBUG: Timeout connecting to {}", addr_str);
                        continue 'peer_loop; 
                    }
                };

                loop {
                    let piece_indeks = {
                        let mut qyuyu = kyuu.lock().unwrap();
                        qyuyu.pop()
                    };
                    
                    let piece_indeks = match piece_indeks {
                        Some(i) => i,
                        None => return, 
                    };

                    let piece_length = metainfoe.piece_length as u32;
                    let total_length = metainfoe.total_length as u64;
                    let piece_start = piece_indeks as u64 * piece_length as u64;
                    let remaining_total = total_length - piece_start;
                    let dis_piece_length = remaining_total.min(piece_length as u64) as u32;
                    let hash_elexpected = &metainfoe.pieces[piece_indeks * 20 .. piece_indeks * 20 + 20];

                    let buffer = match tokio::time::timeout(
                        std::time::Duration::from_secs(15),
                        download_elpiece(&mut stream, piece_indeks as u32, dis_piece_length)
                    ).await {
                        Ok(Ok(buffa)) => {
                            dprintln!("\nDEBUG: Successfully downloaded piece {}!", piece_indeks);
                            buffa
                        },
                        Ok(Err(e)) => {
                            dprintln!("\nDEBUG: download_elpiece failed for piece {}: {}", piece_indeks, e);
                            kyuu.lock().unwrap().insert(0, piece_indeks);
                            continue 'peer_loop; 
                        }
                        Err(_) => {
                            dprintln!("\nDEBUG: Timeout downloading piece {}!", piece_indeks);
                            kyuu.lock().unwrap().insert(0, piece_indeks);
                            continue 'peer_loop; 
                        }
                    };

                    let computed = compute_dem_hash(&buffer);
                    if computed != *hash_elexpected {
                        dprintln!("\nDEBUG: Hash mismatch on piece {}! Bad peer.", piece_indeks);
                        kyuu.lock().unwrap().insert(0, piece_indeks);
                        continue 'peer_loop;
                    }

                    if let Err(e) = file_manager.write_piece(piece_indeks, &buffer) {
                        eprintln!("\nFailed to write piece {}: {}", piece_indeks, e);
                        kyuu.lock().unwrap().insert(0, piece_indeks);
                        continue 'peer_loop;
                    }

                    let done = completed_pieces.fetch_add(1, Ordering::SeqCst) + 1;
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