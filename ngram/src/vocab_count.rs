use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;


fn show_usage(name: &str) {
    println!("Usage : {name} <input-file> <dicionary-file>");
}

fn count_words<P>(filename: P, vocab: &mut HashMap<String, i32>) -> io::Result<i32>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let mut counter: i32 = 0;

    for line_result in reader.lines() {
        let line = line_result?;
        let count = vocab.entry(line).or_insert(0);
        *count += 1;
        counter += 1;
    }

    Ok(counter)
}

fn store_dictionary<P>(filename: P, vocab: &Vec<(&String, &i32)>, cutoff: i32) -> io::Result<(u16, i32)>
where
    P: AsRef<Path>,
{
    let mut file = File::create(filename)?;

    // Possible token values are 0-126 and then 256 through 65,536.
    // This allows variable length encoding of the corpus. Token 127 is
    // reserved for <unknown> for tokens below the cutoff.
    let mut token: u16 = 0;
    let mut counter: u16 = 0;
    let mut mass: i32 = 0;

    // We want to store the following
    // Token : u16
    // Frequency : i32
    // String Length : u8
    // String : variable length
    for word in vocab {
        if word.1 < &cutoff || token > u16::MAX {
            break;
        }
        counter += 1;
        mass += word.1;
        let text = word.0;
        let string_length = text.len();
        if string_length > u8::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "String too long",
            ));
        }
        file.write_all(&token.to_le_bytes())?; // Write u16
        file.write_all(&(word.1).to_le_bytes())?; // Write i32
        file.write_all(&(string_length as u8).to_le_bytes())?; // Write u8
        file.write_all(text.as_bytes())?; // Write string
        if token < 127 || token > 255 {
            token += 1;
        } else {
            token = 256;
        }
    }

    Ok((counter, mass))
}

/*
    What do we need now? We need a way to load the dicitonary into memory, and to do two way lookup.
    Dicitonary size constant, so we do not need to worry about space.
*/

fn main() -> Result<(), String>{
    // Get arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        show_usage(&args[0]);
        return Err("Wrong Number of Arguments".into());
    }

    let mut vocab: HashMap<String, i32> = HashMap::new();
    let input_path = &args[1];

    // execution time of this program will be dominated by this function call,
    // with time cost scaling proportional to the size of the corpus.
    let counted = match count_words(input_path, &mut vocab) {
        Ok(x) => {
            println!("{x} lines counted!");
            x
        }
        Err(e) => {
            return Err(format!("Error reading file: {e}").into());
        }
    };

    let mut vocab_sorted: Vec<(&String, &i32)> = vocab.iter().collect();
    vocab_sorted.sort_by(|a, b| b.1.cmp(a.1));

    let output_path = &args[2];
    match store_dictionary(output_path, &vocab_sorted, 400) {
        Ok(x) => println!(
            "Stored {} in dictionary, covering {}%",
            x.0,
            (x.1 as f32) / (counted as f32) * 100.0
        ),
        Err(e) => return Err(format!("Error writing dicionary: {e}").into()),
    }

    Ok(())
}
