use crc32fast::Hasher;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, prelude::*};
use std::path::Path;
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt)]
struct Cli {
    #[structopt(
        short = "m",
        value_name = "N",
        long,
        required = true,
        default_value = "1",
        help = "Ignore files which is smaller than this size"
    )]
    min_size: u64,

    #[structopt(
        short = "i",
        value_name = "N",
        long,
        required = true,
        default_value = "10",
        help = "How many equal files must be in 2 directories to consider those directories as duplicates"
    )]
    min_intersection: usize,

    #[structopt(
        short = "h",
        value_name = "N",
        long,
        required = true,
        default_value = "1024",
        help = "Reads only N bytes to calculate checksum"
    )]
    head: usize,

    #[structopt(long, required = true, index = 1, help = "Directories to search")]
    directories: Vec<String>,
}

fn get_hash(filename: String, filesize: u64, read_first_bytes: usize) -> io::Result<u64> {
    let crc32 = get_crc32_checksum(filename, read_first_bytes)?;
    Ok(crc32 as u64 + filesize as u64)
}

fn get_crc32_checksum(filename: String, read_first_bytes: usize) -> io::Result<u32> {
    let mut f = File::open(filename)?;
    let mut hasher = Hasher::new();
    const BUF_SIZE: usize = 1024;
    let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];

    let mut bytes_readed = 0;
    loop {
        if read_first_bytes >= 1024 && bytes_readed >= read_first_bytes {
            break;
        }
        let n = f.read(&mut buffer[..])?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[0..n]);
        bytes_readed += n;
    }
    Ok(hasher.finalize())
}

fn get_file_size(path: &String) -> io::Result<u64> {
    Ok(File::open(path)?.metadata()?.len())
}

fn get_files(directories: Vec<String>) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();

    for directory in directories.iter() {
        for entry in WalkDir::new(directory)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = String::from(entry.path().to_string_lossy());
            files.push(path);
        }
    }
    files
}

fn main() {
    let args = Cli::from_args();

    let mut hash_dirs: HashMap<u64, HashSet<String>> = HashMap::new();
    let mut dir_hashes: HashMap<String, HashSet<u64>> = HashMap::new();

    let files = get_files(args.directories);

    let files_cnt = files.len();
    println!("Found: {} files", files_cnt);

    let mut i = 0;
    for file in files.iter() {
        let filesize = match get_file_size(file) {
            Ok(filesize) => filesize,
            Err(e) => {
                eprintln!("Error: {}: {}", file, e);
                continue;
            }
        };
        if filesize < args.min_size {
            continue;
        }

        let dir: String = match Path::new(file).parent() {
            Some(path) => String::from(path.to_string_lossy()),
            None => String::from(""),
        };
        let hash = match get_hash(file.clone(), filesize, args.head) {
            Ok(hash) => hash,
            Err(e) => {
                eprintln!("Error: {}: {}", file, e);
                continue;
            }
        };

        if let Some(val) = hash_dirs.get_mut(&hash) {
            val.insert(dir.clone());
        } else {
            let mut dirs = HashSet::new();
            dirs.insert(dir.clone());
            hash_dirs.insert(hash.clone(), dirs);
        }

        if let Some(val) = dir_hashes.get_mut(&dir) {
            val.insert(hash.clone());
        } else {
            let mut hashes = HashSet::new();
            hashes.insert(hash.clone());
            dir_hashes.insert(dir, hashes);
        }

        if i % (files_cnt / 100) == 0 {
            print!(".");
            io::stdout().flush().ok();
        }
        i += 1;
    }
    println!("");

    let mut printed = HashSet::new();

    let hack = String::from(""); // Fixes "borrow of possibly-uninitialized variable"

    for (_, dirs) in hash_dirs.iter() {
        let mut prev_dir: &String = &hack;
        let mut i = 0;
        for dir in dirs.iter() {
            if i > 0 {
                let t = if *dir < *prev_dir {
                    (dir, prev_dir)
                } else {
                    (prev_dir, dir)
                };
                if printed.contains(&t) {
                    continue;
                }

                let files1 = dir_hashes.get(dir).unwrap();
                let files2 = dir_hashes.get(prev_dir).unwrap();
                let intersection: HashSet<_> = files1.intersection(&files2).collect();
                if intersection.len() > args.min_intersection {
                    println!("{} - {} | {}", dir, prev_dir, intersection.len())
                }
                printed.insert(t);
            }
            prev_dir = dir;
            i += 1;
        }
    }
}
