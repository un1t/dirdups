use crc32fast::Hasher;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, prelude::*};
use std::vec::Vec;
use walkdir::WalkDir;

fn get_hash(filename: String) -> u32 {
    let mut f = File::open(filename).unwrap();
    let mut hasher = Hasher::new();
    const BUF_SIZE: usize = 1024 * 1024;
    let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];

    loop {
        let n = f.read(&mut buffer[..]).unwrap();
        if n == 0 {
            break;
        }
        hasher.update(&buffer[0..n]);
    }
    hasher.finalize()
}

fn main() -> io::Result<()> {
    let mut hash_dirs: HashMap<u32, Vec<String>> = HashMap::new();
    let mut dir_hashes: HashMap<String, HashSet<u32>> = HashMap::new();

    for entry in WalkDir::new("/home/ilya/Pictures")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = String::from(entry.path().to_string_lossy());
        let dir = String::from(entry.path().parent().unwrap().to_string_lossy());
        let hash = get_hash(path.clone());

        let mut dirs = Vec::new();
        if let Some(val) = hash_dirs.get(&hash) {
            dirs = val.clone();
        }
        dirs.push(dir.clone());
        hash_dirs.insert(hash.clone(), dirs);

        let mut hashes = HashSet::new();
        if let Some(val) = dir_hashes.get(&dir) {
            hashes = val.clone();
        }
        hashes.insert(hash.clone());
        dir_hashes.insert(dir, hashes);
    }

    let mut printed = HashSet::new();

    for (_, dirs) in hash_dirs.iter() {
        if dirs.len() < 2 {
            continue;
        }
        for i in 1..dirs.len() {
            let dir1 = dirs.get(i - 1).unwrap();
            let dir2 = dirs.get(i).unwrap();

            if printed.contains(dir1) && printed.contains(dir2) {
                continue;
            }

            let files1 = dir_hashes.get(dir1).unwrap();
            let files2 = dir_hashes.get(dir2).unwrap();

            let intersection: HashSet<_> = files1.intersection(&files2).collect();
            if intersection.len() > 1 {
                println!("{} - {} | {}", dir1, dir2, intersection.len())
            }
            printed.insert(dir1);
            printed.insert(dir2);
        }
    }

    Ok(())
}
