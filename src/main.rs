use crc32fast::Hasher;
use humanize_rs::bytes::Bytes;
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
    min_size: String,

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
        help = "Reads only N bytes to calculate checksum. Set 0 to read full file."
    )]
    head: String,

    #[structopt(long, required = true, index = 1, help = "Directories to search")]
    directories: Vec<String>,
}

fn get_hash(filename: String, filesize: usize, read_first_bytes: usize) -> io::Result<u64> {
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
        if read_first_bytes >= 1000 && bytes_readed >= read_first_bytes {
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

fn get_file_size(path: &String) -> io::Result<usize> {
    Ok(File::open(path)?.metadata()?.len() as usize)
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

fn load_files_info(
    files: &Vec<String>,
    min_size: usize,
    head: usize,
    hash_dirs: &mut HashMap<u64, HashSet<String>>,
    dir_hashes: &mut HashMap<String, HashSet<u64>>,
) {
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
        if filesize < min_size {
            continue;
        }

        let dir: String = match Path::new(file).parent() {
            Some(path) => String::from(path.to_string_lossy()),
            None => String::from(""),
        };
        let hash = match get_hash(file.clone(), filesize, head) {
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
}

fn print_duplicates(
    min_intersection: usize,
    hash_dirs: &HashMap<u64, HashSet<String>>,
    dir_hashes: &HashMap<String, HashSet<u64>>,
) {
    let mut printed = HashSet::new();

    for (_, dirs) in hash_dirs.iter() {
        let mut dirs_iter = dirs.iter();
        let mut prev_dir: &String = match dirs_iter.next() {
            Some(v) => v,
            None => break,
        };

        for dir in dirs_iter {
            let t = if *dir < *prev_dir {
                (dir, prev_dir)
            } else {
                (prev_dir, dir)
            };
            if printed.contains(&t) {
                continue;
            }
            let files = dir_hashes.get(dir).unwrap();
            let prev_files = dir_hashes.get(prev_dir).unwrap();
            let intersection: HashSet<_> = files.intersection(&prev_files).collect();
            if intersection.len() >= min_intersection {
                println!(
                    "{}: {} - {}: {} | {}",
                    dir,
                    files.len(),
                    prev_dir,
                    prev_files.len(),
                    intersection.len()
                )
            }
            printed.insert(t);
            prev_dir = dir;
        }
    }
}

fn main() {
    let args = Cli::from_args();

    let min_size = match args.min_size.parse::<Bytes>() {
        Ok(some) => some.size(),
        Err(_) => {
            eprintln!("Invalid value for '--min-size': {}.", args.min_size);
            return;
        }
    };

    let head = match args.head.parse::<Bytes>() {
        Ok(some) => some.size(),
        Err(_) => {
            eprintln!("Invalid value for '--head': {}.", args.min_size);
            return;
        }
    };
    if head > 0 && head < 1000 {
        eprintln!(
            "Warning!: --min-size values >0 and <1000 are ignored. Default value of 1024 is used."
        );
    }

    let mut hash_dirs: HashMap<u64, HashSet<String>> = HashMap::new();
    let mut dir_hashes: HashMap<String, HashSet<u64>> = HashMap::new();

    let files = get_files(args.directories);
    load_files_info(&files, min_size, head, &mut hash_dirs, &mut dir_hashes);
    print_duplicates(args.min_intersection, &hash_dirs, &dir_hashes);
}
