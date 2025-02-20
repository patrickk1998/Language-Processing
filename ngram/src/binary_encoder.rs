use std::env;
use std::io;
use std::io::BufRead;
use ngram_shared::token_dict::TokenDictionary;
use ngram_shared::token_dict;
use std::path::Path;
use std::fs::{File, OpenOptions};

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
        let line_token = dict.get_by_str(&line); 
        match token_dict::encoded_stream(&mut output_file, line_token) {
            Err(x) => {panic!("{x}");} // Panic for now!
            Ok(_) => {}
        } 
    }

    Ok(())
}

fn main() -> Result<(), String>{

    // Get arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        show_usage(&args[0]);
        return Err("Wrong Number of Arguments".into());
    }

    // Get the dictionary
    let dict;
    match TokenDictionary::new(&args[1]) {
        Err(e) => { 
            return Err(format!("Can not read dictionary {e}").into());
        }
        Ok(x) => {
            println!("Loaded dictionary file {}", &args[1]); 
            dict = x;
        }
    }

    match encode(dict, &args[2], &args[3]) {
        Err(e) => return Err(format!("Error encoding file {e}").into()),
        Ok(()) => println!("Encoded File {} as {}", &args[2], &args[3]),
    }; 

    Ok(())
}
