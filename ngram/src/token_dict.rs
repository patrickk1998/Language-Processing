use std::collections::HashMap;
use std::vec::Vec;
use std::fs::File;
use std::io::{self, Read, Write};
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::borrow::Borrow;
use std::hash::Hash;

/*  
 * TokenDictionary is created from a frequency count of words, represented as  
 * a sorted Vec<(&String, &i32)>, where words are sorted in descending order  
 * of frequency. Tokens are integer values ranging from 0 to TokenDictionary::MAX.  
 *  
 * The 0 token represents an unknown word (a word that does not have a unique token).  
 * Tokens starting from 1 are assigned in descending order of word frequency.  
 *  
 * Tokens can be encoded in two ways:  
 * 1. As a u16 integer.  
 * 2. Using a variable-width encoding scheme, where each token is represented  
 *    using either 1 or 2 bytes.  
 *  
 * Because word frequency follows a power-law distribution, most words occur infrequently.  
 * This allows us to store a collection of n tokens using approximately 1.5 * n bytes,  
 * optimizing storage efficiency.  
 *  
 * A typical use case is as follows:  
 * - A process analyzes a corpus to compute word frequencies and stores the dictionary  
 *   in a file using `store_dictionary`.  
 * - A separate process then loads the dictionary using `new` to encode the corpus into tokens.  
 * - This enables operations such as n-gram counting and language model sampling to be performed  
 *   directly on tokenized data without converting back to words, reducing storage 
 *   and memory requirements.
 */
pub struct TokenDictionary {
    pub str_to_index: HashMap<String, usize>,
    pub token_to_index: HashMap<u16, usize>,
    pub data: Vec<(String, u16)>,
}

impl TokenDictionary {

    pub const MAX : usize = 16510;

    pub fn new<P>(filename : P) -> io::Result<Self> 
    where
        P : AsRef<Path>
    {
        let file = File::open(filename)?;
        let file_size = file.metadata()?.len();
       
        TokenDictionary::read_dictionary(file, file_size)
    }

    // len is needed because read_exact does not distinguish between an EofF error that has filed
    // the buffer or not. 
    fn read_dictionary<S>(mut source: S, len : u64) -> io::Result<TokenDictionary> 
    where S : Read
    {
        let mut td = TokenDictionary {
            str_to_index: HashMap::new(),
            token_to_index: HashMap::new(),
            data: Vec::new(),
        };

        let mut read_size = 0;
        let mut token_buf = [0u8; 2];
        let mut freq_buf = [0u8; 4];
        let mut len_buf = [0u8; 1];
            
        while read_size < len {
            source.read_exact(&mut token_buf)?;
            source.read_exact(&mut freq_buf)?;
            source.read_exact(&mut len_buf)?;

            let token = u16::from_le_bytes(token_buf);
            let _frequency = i32::from_le_bytes(freq_buf);
            let string_length = u8::from_le_bytes(len_buf) as usize;

            let mut string_buf = vec![0u8; string_length];
            source.read_exact(&mut string_buf)?; 
            
            let text = String::from_utf8(string_buf).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 string")
            })?;

            match td.insert(text, token) {
                Err(x) => return Err(Error::new(ErrorKind::InvalidData, x)),
                Ok(()) => {}
            }
            read_size += 7 + string_length as u64; 
        }

        Ok(td)
    }



    pub fn store_dictionary<P>(filename: P, vocab: &Vec<(&String, &i32)>, cutoff: i32) -> io::Result<(u16, i32)>
    where
        P: AsRef<Path>,
    {
        let file = File::create(filename)?;
       
        TokenDictionary::write_dictionary(file, vocab, cutoff)
    }

  
    fn write_dictionary<D>(mut dest : D, vocab : &Vec<(&String, &i32)>, cutoff: i32) -> io::Result<(u16, i32)>
    where D : Write
    {
        // token 0 is the unknown token
        let mut token: u16 = 0;
        let mut counter: u16 = 0;
        let mut mass: i32 = 0;

        // We want to store the following
        // Token : u16
        // Frequency : i32
        // String Length : u8
        // String : variable length
        for word in vocab {
            if *word.1 < cutoff || token > TokenDictionary::MAX as u16 {
                break;
            }
            token += 1;
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
            dest.write_all(&token.to_le_bytes())?; // Write u16
            dest.write_all(&(word.1).to_le_bytes())?; // Write i32
            dest.write_all(&(string_length as u8).to_le_bytes())?; // Write u8
            dest.write_all(text.as_bytes())?; // Write string

        }

        Ok((counter, mass))
    }

    pub fn insert(&mut self, string : String, token : u16) -> Result<(), String> 
    {
        
        if token == 0 {
            return Err("0 is the <unknown> token. It does not represnt a word".to_string());
        }
        if self.str_to_index.contains_key(&string) || self.token_to_index.contains_key(&token) {
            return Err("TokenDictionary already contains Token or Word".to_string());
        }

        let index = self.data.len();
        self.data.push((string.clone(), token.clone()));
        self.str_to_index.insert(string, index);
        self.token_to_index.insert(token, index);

        Ok(())
    }

    pub fn get_by_str<S>(&self, string: &S) -> u16 
    where 
        String : Borrow<S>, S :  Hash + Eq + ?Sized,
    {
        match self.str_to_index.get(string){
            None => 0,
            Some(&index) => self.data[index].1.clone()
        }
    }

    pub fn get_by_token(&self, token: u16) -> Option<String> 
    {
        self.token_to_index.get(&token).map(|&index| self.data[index].0.clone())
    }
}

/* Encoding and Decoding Functions */

/*
*   The variable length encoding scheme is the following:
*
*   Token is > 0 and < 128 then it is encoded in one byte by 0xxxxxxx
*   If the token is  > 127 and < 16,510 then it is encoded in two bytes 
*   as 1xxxxxxx 1xxxxxxx.
*/
pub fn encode_token(token : u16) -> Result<[u8; 2], String>
{
    if token < 128 {
        return Ok([token as u8, 0]);
    } 

    if token > 127 && token < 16510 {
        let mut ls = token as u8;  
        let mut ms = (token >> 7) as u8;      
        ls = 0b10000000 | ls;
        ms = 0b10000000 | ms;
        return Ok([ls, ms]);
    }

    Err("Token out of range".into())
}

pub fn encoded_stream<D>(mut destination:  D, token: u16) -> Result<(), String>
where 
    D: Write
{
    let buf = match encode_token(token) {
        Err(e) => return Err(e),
        Ok(x) => x,
    };

    let write_result = if coded_size(buf[0]) == 1 {
        destination.write(&buf[0..1])
    } else {
        destination.write(&buf[0..2])
    };

    match write_result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Write error: {}", e)),
    }
}

pub fn coded_size(b : u8) -> u8
{
    if (b >> 7) == 1 {
        2 
    } else {
        1
    } 
}

pub fn decode_token<S>(mut source : S ) -> io::Result<u16>
where 
    S : Read
{
    let mut byte1 = [0; 1];
    let mut byte2 = [0; 1];
    match source.read(&mut byte1) {
        Err(e) => return Err(e),
        Ok(_x) => {}
    }

    if byte1[0] < 128 {
        return Ok(byte1[0] as u16);
    }

    match source.read(&mut byte2){
        Err(e) => return Err(e),
        Ok(_x) => {}
    }

    byte1[0] = (byte1[0] & 0b01111111) | (byte2[0] << 7);   
    byte2[0] = (byte2[0] & 0b01111111) >> 1;
    Ok( u16::from_le_bytes( [byte1[0], byte2[0]]) )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_encode_decode_tokens() {
        let tokens = vec![10, 127, 200, 500, 1000, 16000];
        let mut buffer = Vec::new();

        for &token in &tokens {
            encoded_stream(&mut buffer, token).expect("Encoding failed");
        }

        let mut cursor = Cursor::new(buffer);
        let mut decoded_tokens = Vec::new();

        for _ in &tokens {
            let decoded_token = decode_token(&mut cursor).expect("Decoding failed");
            decoded_tokens.push(decoded_token);
        }

        assert_eq!(tokens, decoded_tokens, "Decoded tokens do not match the original set!");
    }

    #[test]
    fn test_write_and_read_td() {
        let vocab : Vec<(String, i32)> = vec![
            ("hello".to_string(), 42),
            ("world".to_string(), 84),
            ("rust".to_string(), 128),
            ("programming".to_string(), 256),
            ("language".to_string(), 512),
            ("test".to_string(), 1024),
            ("data".to_string(), 2048),
            ("example".to_string(), 4096),
            ("buffer".to_string(), 8192),
            ("cursor".to_string(), 16384),
        ];

        
        let mut buffer = Cursor::new(Vec::new());
        let cutoff = 10;
       
        let vocab_ref = vocab.iter().map( |(s , n)| { (s , n)} ).collect();
        let (counter, mass) = TokenDictionary::write_dictionary(&mut buffer, &vocab_ref, cutoff).expect("Failed to write dictionary in test");
        assert_eq!(counter, vocab.len() as u16);
        assert_eq!(mass, vocab.iter().map(|(_, f)| *f).sum::<i32>());
        
        buffer.set_position(0);
        let len = buffer.get_ref().len();
        let td = TokenDictionary::read_dictionary(&mut buffer, len as u64).expect("Failed to read dictionary test");
       
        let mut token = 1;
        for (word, _) in vocab.iter() {
            assert_eq!(td.get_by_str(word), token);
            assert_eq!(&td.get_by_token(token).expect("Token we expect do exist does not!"), word);
            token += 1;
        }
    }
}
