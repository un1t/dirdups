use crc32fast::Hasher;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::vec::Vec;
use walkdir::WalkDir;

fn get_hash(filename: String) -> u32 {
    let mut f = File::open(filename).unwrap();
    let mut hasher = Hasher::new();
    let mut buffer: [u8; 1024] = [0; 1024];

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
    let file_hash: HashMap<String, u32> = HashMap::new();
    let hash_files: HashMap<u32, Vec<String>> = HashMap::new();

    for entry in WalkDir::new("..")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let fullname = String::from(entry.path().to_string_lossy());
        let filename = String::from(entry.file_name().to_string_lossy());
        let dirname = String::from(entry.path().parent().unwrap().to_string_lossy());

        println!("{} {}", dirname, filename);
        let checksum = get_hash(fullname.clone());
        println!("{} {}", fullname, checksum);

        // break;
    }

    // return Ok(());

    let mut file_dirs: HashMap<String, Vec<String>> = HashMap::new();
    let mut dir_files: HashMap<String, HashSet<String>> = HashMap::new();

    let file = File::open("files.txt")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        if let Ok(path) = line {
            if let Some(parts) = path.rsplit_once('/') {
                let dir = String::from(parts.0);
                let filename = String::from(parts.1);

                let mut dirs = Vec::new();
                if let Some(val) = file_dirs.get(&filename) {
                    dirs = val.clone();
                }
                dirs.push(dir.clone());
                file_dirs.insert(filename.clone(), dirs);

                let mut files = HashSet::new();
                if let Some(val) = dir_files.get(&dir) {
                    files = val.clone();
                }
                files.insert(filename);
                dir_files.insert(dir, files);
            }
        }
    }

    let mut printed = HashSet::new();

    for (_, dirs) in file_dirs.iter() {
        if dirs.len() < 2 {
            continue;
        }
        for i in 1..dirs.len() {
            let dir1 = dirs.get(i - 1).unwrap();
            let dir2 = dirs.get(i).unwrap();

            if printed.contains(dir1) && printed.contains(dir2) {
                continue;
            }

            let files1 = dir_files.get(dir1).unwrap();
            let files2 = dir_files.get(dir2).unwrap();

            let intersection: HashSet<_> = files1.intersection(&files2).collect();
            if intersection.len() > 50 {
                println!("{} - {} | {}", dir1, dir2, intersection.len())
            }
            printed.insert(dir1);
            printed.insert(dir2);
        }
    }

    Ok(())
}
