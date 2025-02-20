use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use ngram_shared::token_dict::TokenDictionary;


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
    match TokenDictionary::store_dictionary(output_path, &vocab_sorted, 400) {
        Ok(x) => println!(
            "Stored {} in dictionary, covering {}%",
            x.0,
            (x.1 as f32) / (counted as f32) * 100.0
        ),
        Err(e) => return Err(format!("Error writing dicionary: {e}").into()),
    }

    Ok(())
}
