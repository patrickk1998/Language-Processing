use std::collections::HashMap;
use std::fs::File;
use std::io::{self};
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::borrow::Borrow;
use std::hash::Hash;
use std::io::Read;

pub struct TokenDictionary {
    str_to_index: HashMap<String, usize>,
    token_to_index: HashMap<u16, usize>,
    data: Vec<(String, u16)>,
}

impl TokenDictionary {

    pub fn new<P>(filename : P) -> io::Result<Self> 
    where
        P : AsRef<Path>
    {
        let mut td = Self {
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
