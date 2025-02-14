use std::env;
use std::io;
use std::io::{BufRead, Write};
use token_dict::TokenDictionary;
use std::path::Path;
use std::fs::{File, OpenOptions};

mod token_dict;

fn show_usage(name: &str) {
    println!("Usage : {name} <dictionary-file> <input-file> <output-file>");
}

fn encode<P, Q>(dict : TokenDictionary, input_path : P, output_path : Q) -> io::Result<()>
where P : AsRef<Path>, Q : AsRef<Path>
{
    let input_file = File::open(input_path)?;
    let reader = io::BufReader::new(input_file);
    
    let mut output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_path)?;
    
    for line in reader.lines() {
        let line = line?;
        let encoded_line;
        match dict.get_by_str(&line) {
            None => encoded_line = 126, // encoding a unkown word
            Some(x) => encoded_line = x,
        }

        let bytes;
        if encoded_line < 127 {
            bytes = (encoded_line as u8).to_le_bytes().to_vec();
        } else {
            bytes = encoded_line.to_le_bytes().to_vec();
        }
        output_file.write_all(&bytes)?;
    }

    Ok(())
}

fn main(){

    // Get arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        show_usage(&args[0]);
        return;
    }

    // Get the dictionary
    let dict;
    match TokenDictionary::new(&args[1]) {
        Err(e) => { 
            println!("Error reading dictionary {e}"); 
            return; 
        }
        Ok(x) => {
            println!("Loaded dictionary file {}", &args[1]); 
            dict = x;
        }
    }

    match encode(dict, &args[2], &args[3]) {
        Err(e) => println!("Error encoding file {e}"),
        Ok(()) => println!("Encoded File {} as {}", &args[2], &args[3]),
    }; 
}
