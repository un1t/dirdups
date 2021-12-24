use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::Path;
use std::vec::Vec;

fn main() -> io::Result<()> {
    let mut file_dirs: HashMap<String, Vec<String>> = HashMap::new();
    let mut dir_files: HashMap<String, HashSet<String>> = HashMap::new();

    let file = File::open("files.txt")?;
    let reader = BufReader::new(file);

    let mut i = 0;
    for line in reader.lines() {
        i += 1;
        if i > 10000 {
            // break;
        }
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

        // dbg!(&file_dirs);
        // dbg!(&dir_files);
    }

    let mut printed = HashSet::new();

    for (filename, dirs) in file_dirs.iter() {
        if dirs.len() >  1 {

            for i in 1..dirs.len() {
                let dir1 = dirs.get(i-1).unwrap();
                let dir2 = dirs.get(i).unwrap();

                if printed.contains(dir1) && printed.contains(dir2) {
                    continue
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
    }

    // dbg!(filename);
    // dbg!(dirs);

    Ok(())
}
