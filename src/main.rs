// MAIN FUNCTION N SHII

//calling all the modules
mod bencode;
mod torrent;
mod tracker;
mod peer;
mod download;

use bencode::{BencodeValue, parsdat, get_eldict_value};
use torrent::{TorrentMetainfo, FileInfo, parse_eltorrento, infoget, compute_dem_hash};
use tracker::{urlencode_bytes, get_peers};
use peer::{PeerMessage, build_handshake, read_message, send_message, build_elrequesto_payload, connect_and_handshaker, download_elpiece};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use download::{download_eltorrento, FileManager}; 
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering, AtomicBool};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;

pub fn start_spinner(message: String) -> (std::thread::JoinHandle<()>, Arc<AtomicBool>) {
    let keep_running = Arc::new(AtomicBool::new(true));
    let kr_clone = keep_running.clone();
    
    let handle = std::thread::spawn(move || {
        let dots = [".", "..", "..."];
        let mut i = 0;
        

        while kr_clone.load(Ordering::SeqCst) {
            print!("\r{}{}", message, dots[i % 3]);
            std::io::stdout().flush().unwrap();
            
            std::thread::sleep(std::time::Duration::from_millis(400));
            i += 1;
        }
        
        print!("\r{}... Done!   \n", message);
        std::io::stdout().flush().unwrap();
    });
    
    (handle, keep_running)
}

fn validate_torrent_path(raw_path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw_path);
    if raw_path.contains('\0') {
        return Err("input path contains invalid null bytes. you sneaky bastard".into());
    }

    let canonical = path.canonicalize()
        .map_err(|_| format!("no torrent file here to be found: '{}'", raw_path))?;

    if !canonical.is_file() {
        return Err(format!("that path aint a regular file. i dont trust you: '{}'", raw_path));
    }
    Ok(canonical)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3  {
        eprintln!("how to use: {} <path_to_torrent> <where you want the file to be>", args[0]);
        std::process::exit(1);
    }
    let torrent_path = validate_torrent_path(&args[1])
            .unwrap_or_else(|err| {
                eprintln!("Input Error: {}", err);
                std::process::exit(1);
            });

    let output_folder = PathBuf::from(&args[2]);
    fs::create_dir_all(&output_folder)?;
    let canonical_output_dir = output_folder.canonicalize()?;

    let bytes = fs::read(&torrent_path)?;

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
        metainfo.total_length 
    );


    let (spinner_handle, spinner_flag) = start_spinner("Contacting tracker and finding peers".to_string());
    let peers = get_peers(&url).await?;
    spinner_flag.store(false, Ordering::SeqCst); 
    spinner_handle.join().unwrap();             
    let file_manager = FileManager::new(canonical_output_dir.clone(), &metainfo);


    // because im vegeta
    let peers = get_peers(&url).await?;
    let piece_length = metainfo.piece_length as u32;
    let total_length = metainfo.total_length as u64; 
    let num_pieces = metainfo.pieces.len() / 20;


    

    let file_manager = FileManager::new(canonical_output_dir.clone(), &metainfo);
    

    file_manager.prepare_dose_directories().unwrap_or_else(|err| {
        eprintln!("Directory Setup Error: {}", err);
        std::process::exit(1);
    });


    let mut remaining_pieces = Vec::new();
    for piece_index in 0..num_pieces {
        let expected_hash = &metainfo.pieces[piece_index * 20 .. piece_index * 20 + 20];
        

        let is_complete = file_manager.verify_dat_onepiece(piece_index, expected_hash).unwrap_or(false);

        if !is_complete {
            remaining_pieces.push(piece_index);
        }
    }
    println!("{} of {} pieces remaining", remaining_pieces.len(), num_pieces);


    let metainfo_arc = Arc::new(metainfo);
    let file_manager_arc = Arc::new(file_manager); 
    let completed_pieces = Arc::new(AtomicUsize::new(0));

    download_eltorrento(
        peers,
        metainfo_arc,
        hashed_info,
        *peer_id,
        file_manager_arc, 
        remaining_pieces,
        20, 
        completed_pieces,
        num_pieces,
    ).await;
    
    println!("we done frfr");
    Ok(())
}