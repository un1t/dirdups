use crc32fast::Hasher;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, prelude::*};
use std::path::Path;
use structopt::StructOpt;
use walkdir::WalkDir;

// TODO: handle errors

#[derive(StructOpt)]
struct Cli {
    starting_point: String,
    number: usize,
    min_size: u64,
}

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

fn get_file_size(path: &String) -> u64 {
    let f = File::open(path).unwrap();
    let metadata = f.metadata().unwrap();
    metadata.len()
}

fn main() -> io::Result<()> {
    let args = Cli::from_args();

    let mut hash_dirs: HashMap<u32, HashSet<String>> = HashMap::new();
    let mut dir_hashes: HashMap<String, HashSet<u32>> = HashMap::new();

    let mut paths: Vec<String> = Vec::new();

    let starting_points: Vec<&str> = args.starting_point.split(",").collect();
    for starting_point in starting_points {
        for entry in WalkDir::new(starting_point)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = String::from(entry.path().to_string_lossy());
            paths.push(path);
        }
    }

    let files_cnt = paths.len();
    println!("Found: {} files", files_cnt);

    let mut i = 0;
    for path in paths.iter() {
        if get_file_size(path) < args.min_size {
            continue;
        }

        let dir = String::from(Path::new(path).parent().unwrap().to_string_lossy());
        let hash = get_hash(path.clone());

        let mut dirs = HashSet::new();
        if let Some(val) = hash_dirs.get(&hash) {
            dirs = val.clone();
        }
        dirs.insert(dir.clone());
        hash_dirs.insert(hash.clone(), dirs);

        let mut hashes = HashSet::new();
        if let Some(val) = dir_hashes.get(&dir) {
            hashes = val.clone();
        }
        hashes.insert(hash.clone());
        dir_hashes.insert(dir, hashes);

        if i % (files_cnt / 100) == 0 {
            print!(".");
            io::stdout().flush().unwrap();
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
                if printed.contains(dir) && printed.contains(prev_dir) {
                    continue;
                }
                let files1 = dir_hashes.get(dir).unwrap();
                let files2 = dir_hashes.get(prev_dir).unwrap();
                let intersection: HashSet<_> = files1.intersection(&files2).collect();
                if intersection.len() > args.number {
                    println!("{} - {} | {}", dir, prev_dir, intersection.len())
                }
                printed.insert(dir);
                printed.insert(prev_dir);
            }
            prev_dir = dir;
            i += 1;
        }
    }

    Ok(())
}
