use walkdir::WalkDir;
use std::{io, vec};
use xxhash_rust::xxh3::Xxh3Default;
use std::path::Path;
use std::fs::File;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use indicatif::ProgressBar;

const BUF_SIZE: usize = 1024 * 1024 * 8;

fn generate_hashes() -> (u32, u32) {
    let files = WalkDir::new(".").into_iter().filter_map(|e| e.ok());
    let mut hasher = Xxh3Default::new();
    let mut buffer = vec![0; BUF_SIZE];
    let mut count = 0;
    let mut good_files = 0;
    let mut bad_files = 0;
    let mut first = true;
    let mut result = String::new();
    let name = format!(".\\{}", self_name().unwrap());
    for f in files {
        let meta = fs::metadata(f.path()).unwrap();
        if !meta.is_file() {
            continue;
        }
        if f.path().display().to_string() == name {
            continue; // skip self
        }
        println!("hashing: {}", f.path().display());
        let file = File::open(f.path());
        if file.is_err() {
            println!("Error opening file: {}", file.err().unwrap());
            bad_files += 1;
            continue;
        } else {
            good_files += 1;
        }
        let mut reader = BufReader::new(file.unwrap());
        let pb = ProgressBar::new(meta.len());
        hasher.reset();
        loop {
            count = reader.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
            if meta.len() > BUF_SIZE as u64 {
                pb.inc(BUF_SIZE as u64);
            } else {
                pb.finish_and_clear();
            }
        }
        if first {
            result = format!("{} {:032X?}", f.path().display(), hasher.digest128());
            first = false;
        } else {
            result = format!("{}\n{} {:032X?}", result, f.path().display(), hasher.digest128());
        }
    }
    fs::write("hashes.txt", result).expect("Unable to write hashes.txt!");
    println!("hashes.txt saved.");
    (good_files, bad_files)
}

fn verify_hashes() -> (u32, u32, u32) {
    // returns verified, failed, not found
    let mut hasher = Xxh3Default::new();
    let mut buffer = vec![0; BUF_SIZE];
    let mut verified = 0;
    let mut failed = 0;
    let mut not_found = 0;
    let mut failed_paths: Vec<String> = Vec::new();
    let mut missing_paths: Vec<String> = Vec::new();
    let mut count = 0;
    let file = File::open("hashes.txt");
    if file.is_err() {
        println!("Error opening hashes.txt: {}", file.err().unwrap());
        return (0, 0, 0);
    }
    let reader = BufReader::new(file.unwrap());
    for line in reader.lines() {
        let s = line.unwrap().trim().to_string();
        let split_pos = s.char_indices().nth_back(32).unwrap().0;
        let hex = &s[1 + split_pos..];
        let path = &s[..split_pos];
        let file = File::open(path);
        if file.is_err() {
            println!("Error opening file: {}", file.err().unwrap());
            not_found += 1;
            missing_paths.push(path.to_string());
            continue;
        }
        let mut reader = BufReader::new(file.unwrap());
        let len = fs::metadata(path).unwrap().len();
        let pb = ProgressBar::new(len);
        hasher.reset();
        loop {
            count = reader.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
            if len > BUF_SIZE as u64 {
                pb.inc(BUF_SIZE as u64);
            } else {
                pb.finish_and_clear();
            }
        }
        if format!("{:032X?}", hasher.digest128()) == hex {
            verified += 1;
        } else {
            println!("Failed to verify {}: {} != {}", path, format!("{:032X?}", hasher.digest128()), hex);
            failed += 1;
            failed_paths.push(path.to_string());
        }
    }
    if missing_paths.len() > 0 {
        println!("The following files were not found:");
        for p in missing_paths {
            println!("{}", p);
        }
    }
    if failed_paths.len() > 0 {
        println!("The following files failed verification:");
        for p in failed_paths {
            println!("{}", p);
        }
    }
    (verified, failed, not_found)
}

fn self_name() -> Option<String> {
    std::env::current_exe()
        .ok()?
        .file_name()?
        .to_str()?
        .to_owned()
        .into()
}
fn main() {
    let mut error = false;
    if Path::new("hashes.txt").exists() {
        println!("Found hashes.txt, verifying hashes...");
        let result = verify_hashes();
        println!("{} files verified, {} files failed, {} files not found.", result.0, result.1, result.2);
        if result.1 != 0 || result.2 != 0 {
            error = true;
        }
    } else {
        println!("hashes.txt not found, hashing files...");
        let result = generate_hashes();
        println!("{} files hashed, unable to read {} files.", result.0, result.1);
        if result.1 != 0 {
            error = true;
        }
    }
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();
    let _ = stdin.read(&mut [0u8]).unwrap();
    if error {
        std::process::exit(1);
    } else {
        std::process::exit(0);
    }
}