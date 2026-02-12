use core::hash;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::{
    fs::File,
    hash::Hash,
    io::{self, BufRead, Read, Seek, SeekFrom, Write},
    vec,
};

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if the user provided at least one argument (args[0] is the program name)
    if args.len() < 2 {
        println!("Usage:");
        println!("  cargo run find <word>   - Search for a word");
        println!("  cargo run index         - Create index.txt from tokens");
        println!("  cargo run magic         - Create magic_file.txt from index");
        return;
    }

    let command = &args[1];

    if command == "find" {
        // Check if a word was actually provided after 'find'
        let target = args.get(2).map(|s| s.as_str()).unwrap_or("a");

        println!("Searching for: {}", target);
        if let Some(word_data) = find_word(target) {
            println!("Word: {} | Count: {}", word_data.word, word_data.word_count);
        } else {
            println!("Word '{}' not found.", target);
        }
    } else if command == "index" {
        println!("Generating index.txt...");
        make_index_file();
        println!("Done!");
    } else if command == "magic" {
        println!("Generating magic_file.txt...");
        make_magic_file();
        println!("Done!");
    } else {
        println!("Unknown command: {}", command);
    }
}

struct WordData {
    word: String,
    word_count: usize,
}

fn find_word(target: &str) -> Option<WordData> {
    let mut magic_file = File::open("magic_file.txt").expect("Could not open magic_file.txt");
    let mut index_file = File::open("index.txt").expect("Could not open index.txt");
    let target_hash = lazy_hash(target);
    let jump = (target_hash * 8) as u64;
    magic_file.seek(SeekFrom::Start(jump));

    let mut index_buf: [u8; 8] = [0u8; 8];
    magic_file.read_exact(&mut index_buf);

    let index: u64 = u64::from_ne_bytes(index_buf);
    index_file.seek(SeekFrom::Start(index));
    let reader = io::BufReader::new(index_file);
    for line_ in reader.lines() {
        let line = line_.expect("could not read line");
        let split_line: Vec<&str> = line.split_whitespace().collect();
        let word = split_line[0];
        if lazy_hash(word) != target_hash {
            return None;
        }
        if word == target {
            let data = WordData {
                word: split_line[0].to_string(),
                word_count: split_line.len() - 1,
            };
            return Some(data);
        }
    }
    return None;
}

fn lazy_hash(word: &str) -> usize {
    let mut hash: usize = 0;
    let mut count = 0;
    for c in word.chars() {
        let x: usize = match c {
            'a'..='z' => (c as u8 + 1 - b'a') as usize,
            'å' => ('z' as u8 + 2 - b'a') as usize,
            'ä' => ('z' as u8 + 3 - b'a') as usize,
            'ö' => ('z' as u8 + 4 - b'a') as usize,
            _ => 0,
        };
        hash = (hash * 32) + x;
        count += 1;
        if count == 3 {
            break;
        }
    }
    return hash;
}

fn make_magic_file() {
    let index_file = File::open("index.txt").expect("Could not open index.txt");
    let reader = io::BufReader::new(index_file);
    let mut magic_file = File::create("magic_file.txt").expect("could not create magic_file.txt");
    let mut hash_table: Vec<u64> = vec![u64::MAX; 35000];
    let mut current_offset: u64 = 0;

    for line_ in reader.lines() {
        let line = line_.expect("could not read line in index");
        let word = line.split_whitespace().next().unwrap();
        let line_len: u64 = (line.len() + 1) as u64;
        let hash = lazy_hash(word);

        if hash_table[hash] == u64::MAX {
            hash_table[hash] = current_offset;
        }
        current_offset += line_len;
    }
    for offset in hash_table {
        magic_file.write_all(&offset.to_ne_bytes());
    }
}

fn make_index_file() {
    let input_file = File::open("token/token.txt").expect("Could not open token.txt");
    let reader = io::BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1252))
            .build(input_file),
    );
    let mut output_file = File::create("index.txt").expect("could not create index.txt");
    let mut line = String::new();
    let mut locations: Vec<u64> = Vec::new();
    let mut pre_word: String = String::new();

    for line_ in reader.lines() {
        let line = line_.expect("could not read line");
        let word_location: Vec<&str> = line.split_whitespace().collect();
        let word = word_location[0];
        if pre_word.eq("") {
            pre_word = word.to_string();
        }
        if !word.eq(&pre_word) {
            //write to file

            write!(output_file, "{} ", pre_word);
            for location in locations {
                write!(output_file, "{} ", location);
            }
            writeln!(output_file, "");

            pre_word = word.to_string();
            locations = Vec::new();
        }

        let location: u64 = word_location[1].parse().unwrap();
        locations.push(location);
    }
    //writes index of last word
    if !pre_word.eq("") {
        write!(output_file, "{} ", pre_word);
        for location in locations {
            write!(output_file, "{} ", location);
        }
        writeln!(output_file, "");
    }
}
