use std::collections::HashMap;
use std::vec::Vec;
use std::fs::File;
use std::io::{self, Read, Write};
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::borrow::Borrow;
use std::hash::Hash;

pub struct TokenDictionary {
    pub str_to_index: HashMap<String, usize>,
    pub token_to_index: HashMap<u16, usize>,
    pub data: Vec<(String, u16)>,
}

impl TokenDictionary {

    pub fn new<P>(filename : P) -> io::Result<Self> 
    where
        P : AsRef<Path>
    {
        let mut td = TokenDictionary {
            str_to_index: HashMap::new(),
            token_to_index: HashMap::new(),
            data: Vec::new(),
        };
        
        let mut file = File::open(filename)?;
        let file_size = file.metadata()?.len();
        let mut read_size = 0;
        
        let mut token_buf = [0u8; 2];
        let mut freq_buf = [0u8; 4];
        let mut len_buf = [0u8; 1];
            
        while read_size < file_size {
            file.read_exact(&mut token_buf)?;
            file.read_exact(&mut freq_buf)?;
            file.read_exact(&mut len_buf)?;

            let token = u16::from_le_bytes(token_buf);
            let _frequency = i32::from_le_bytes(freq_buf);
            let string_length = u8::from_le_bytes(len_buf) as usize;

            let mut string_buf = vec![0u8; string_length];
            file.read_exact(&mut string_buf)?; 
            
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
    
    pub fn insert(&mut self, string : String, token : u16) -> Result<(), String> 
    {
     
        if self.str_to_index.contains_key(&string) || self.token_to_index.contains_key(&token) {
            return Err("TokenDictionary already contains Token or Word".to_string());
        }
        let index = self.data.len();
        self.data.push((string.clone(), token.clone()));
        self.str_to_index.insert(string, index);
        self.token_to_index.insert(token, index);

        Ok(())
    }

    pub fn get_by_str<S>(&self, string: &S) -> Option<u16> 
    where 
        String : Borrow<S>, S :  Hash + Eq + ?Sized,
    {
        self.str_to_index.get(string).map(|&index| self.data[index].1.clone())
    }

    pub fn get_by_token(&self, token: u16) -> Option<String> 
    {
        self.token_to_index.get(&token).map(|&index| self.data[index].0.clone())
    }
}

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
}
