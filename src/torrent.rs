// THIS IS FOR THE TORRENT READING FUNCTIONS AND WHATNOT
use crate::bencode::{BencodeValue, parsdat, get_eldict_value, stringiparser};
use sha1::{Sha1, Digest};
#[derive(Debug, Clone)]
//edited the metainfo to support multiple files
pub struct FileInfo {
    pub path: Vec<String>,
    pub length: i64,
}
pub struct TorrentMetainfo {
    pub announce: String,
    pub piece_length: i64,
    pub pieces: Vec<u8>,
    pub files: Vec<FileInfo>,
    pub total_length: i64,
}

// so i refactored the parsing to support multiple files
pub fn parse_eltorrento(value: &BencodeValue) -> Result<TorrentMetainfo, String> {
    let BencodeValue::Dict(top) = value else {
        return Err("gimme a dictionary in the top level fucking dumdum".to_string());
    };

    let announce = match get_eldict_value(top, "announce") {
        Some(BencodeValue::Bytes(b)) => String::from_utf8_lossy(b).to_string(),
        _ => return Err("i aint see no announce. or that shi might be ass'".to_string()),
    };

    let BencodeValue::Dict(info) = get_eldict_value(top, "info")
        .ok_or("no info in this dictionary".to_string())? 
    else {
        return Err("no info. dumbass".to_string());
    };

    let piece_length = match get_eldict_value(info, "piece length") {
        Some(BencodeValue::Int(n)) => *n,
        _ => return Err("no piece length *boowomp*'".to_string()),
    };

    let pieces = match get_eldict_value(info, "pieces") {
        Some(BencodeValue::Bytes(b)) => b.clone(),
        _ => return Err("no 'pieces' my homie".to_string()),
    };


    let mut file_infos: Vec<FileInfo> = Vec::new();
    
    //check to see the file list check inside it and allat
    if let Some(BencodeValue::List(file_list)) = get_eldict_value(info, "files") {

        for item_innafile in file_list {
            if let BencodeValue::Dict(file_dict) = item_innafile {
                let length = match get_eldict_value(file_dict, "length") {
                    Some(BencodeValue::Int(n)) => *n,
                    _ => return Err("the file in that list is missing a length".to_string()),
                };
                //normal path. we assign the path in bytes since its segemented we collect it
                let pathsir = match get_eldict_value(file_dict, "path") {
                    Some(BencodeValue::List(path_listito)) => {
                        let mut temp_paths = Vec::new();
                        //this is the part where we iterate the segmented file paths
                        for path_segmentation in path_listito {
                            if let BencodeValue::Bytes(bytes) = path_segmentation {
                                temp_paths.push(String::from_utf8_lossy(bytes).to_string());
                            } else {
                                return Err("the path segment isnt in bytes honey".to_string());
                            }
                        }
                        temp_paths
                    },
                    _ => return Err("file in the list is missing a path".to_string()),
                };
                // we just push the shi and its all good
                file_infos.push(FileInfo {
                    path: pathsir,
                    length,
                });
            }
        }
        //in case its just a normal one file. we do like the previous parsing.
    } else {

        let name = match get_eldict_value(info, "name") {
            Some(BencodeValue::Bytes(b)) => String::from_utf8_lossy(b).to_string(),
            _ => return Err("aint no way you didnt gimme a name, dumbass'".to_string()),
        };
        
        let length = match get_eldict_value(info, "length") {
            Some(BencodeValue::Int(n)) => *n,
            _ => return Err("no 'length' for single file".to_string()),
        };

        file_infos.push(FileInfo {
            path: vec![name], 
            length,
        });
    }


    let total_length: i64 = file_infos.iter().map(|f| f.length).sum();

    //push dat shi outside
    Ok(TorrentMetainfo { announce, piece_length, pieces, files: file_infos, total_length })
}

//we gotta get that info hex no?
pub fn infoget(input: &[u8]) -> Result<(Vec<u8>, usize), String> {
    let mut position = 1 ;// we still skipping the big d bro 
    let mut items = Vec::new();
    while input[position] != b'e' {
        let (key, consumed_key) = stringiparser(&input[position..])?;
        position += consumed_key;
        //if else fuck arounds to find out the info. if its not there execute the user publicly
        if key == b"info" {
        let info_start = position;
        let (_, consumed_val) = parsdat(&input[position..])?;
        let info_end = info_start + consumed_val;
        return Ok((input[info_start..info_end].to_vec(), consumed_key));
        } else {
        let (value, consumed_val) = parsdat(&input[position..])?;
        position += consumed_val;

        items.push((key, value));
        }
    }
    Err("couldnt find el big info".to_string())

}


// this is for internety reasons or whatever. we just cant give the guy a list of numbers
pub fn compute_dem_hash(input: &[u8]) -> [u8; 20]{
    let mut hashment = Sha1::new();
    hashment.update(input);
    let hashed = hashment.finalize();
    hashed.into()
}