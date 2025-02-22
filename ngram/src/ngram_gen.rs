use std::io::Read;
use std::io::Seek;
use std::io;
use std::fs;
use std::cmp;

struct Split{
    file_name : String,
    start: i64,
    cursor: i64,
    end: i64,
    file : Option<fs::File>
}

impl Split{
    fn open(&mut self) -> io::Result<()>
    {
        let mut temp =fs::File::open(&self.file_name)?;
        temp.seek(io::SeekFrom::Start(self.start as u64))?;
        self.file = Some(temp);
        self.cursor = self.start;
        Ok(())
    }

    fn close(&mut self){
        self.file = None; 
    }

    fn split_file(file_name : String , block : i64, edge : fn(&[u8]) -> bool) -> io::Result<Vec<Split>>
    {
        let file = fs::File::open(&file_name)?;
        let file_size = file.metadata()?.len() as i64;
        Split::split_source(file, file_size, file_name, block, edge)
    }

    fn split_source<S>(mut source : S, size : i64, file_name : String, block : i64, edge : fn(&[u8]) -> bool) -> io::Result<Vec<Split>>
    where S : Read + Seek
    {
        let mut buf = [0 as u8]; 
        let mut splits : Vec<Split> = Vec::new();

        let mut start = 0; 

/*
        while cursor < file_size{
            file.seek(io::SeekFrom::Start(cursor as u64))?;
            loop {
                file.read(&mut buf[..1])?;  // read at current cursor position.
                if edge(&buf) || cursor >= file_size {
                    break;
                }
                cursor += 1;
            } 

            if cursor < file_size {
                splits.push(Split { file_name : file_name.clone(), start: start, cursor : 0, end : cursor + 1, file : None});
                start = cursor + 1; // should be the same as end.
                cursor += block; // should be block - 1 offset from start.
            }
        }
*/ 

        let mut cursor = block - 1; //cursor will always be index of unread byte
        if cursor >= size {
            splits.push(Split { file_name : file_name.clone(), start: start, cursor : 0, end : size, file : None});
            return Ok(splits);
        } 

        source.seek(io::SeekFrom::Start(cursor as u64))?;
        loop {
            // read at current cursor position.
            match source.read(&mut buf[..1]) {
                Err(e) => {return Err(e);}
                Ok(x) => { if x == 0 {break;} else {cursor += 1;} } // check if the end of the file is reached.
            }

            // check if edge condition for split is triggered; if true, push current split and start
            // a new one.
            if edge(&buf) {
                //start of next split should be end of previous one.
                println!("Pushing new split {start}, {cursor}");
                splits.push(Split { file_name : file_name.clone(), start: start, cursor : 0, end : cursor, file : None});
                start = cursor; // should be the same as end.
                cursor += block - 1; // should be block - 1 offset from start.
                if cursor < size {
                    source.seek(io::SeekFrom::Start(cursor as u64))?;
                } else {
                    break;
                }
            }
        }
        splits.push(Split { file_name : file_name.clone(), start: start, cursor : 0, end : size, file : None});

        Ok(splits)

    }
}

impl io::Read for Split{
    fn read(&mut self, buf : &mut [u8]) -> io::Result<usize>
    {
        let l = cmp::min(buf.len() as i64, self.end - self.cursor) as usize; 
        match self.file.as_ref().unwrap().read(&mut buf[..l]) {
            Ok(x) => {self.cursor += x as i64; Ok(x)}
            Err(e) => { Err(e) }
        } 
    }
} 


fn main() {

}


#[cfg(test)]
mod tests{
use std::io;
use crate::Split;

    #[test]
    fn test_split(){
        let data : Vec<u8> = vec![1,1,1,2,1,1,1,2,1,1,1,1,2,1,1];
        let starts_ends = vec![(0,4),(4,8),(8,13),(13,15)];
        let len = data.len();
        let source = io::Cursor::new(data);
        let splits = Split::split_source(source,  len as i64, "null".to_string(), 4, |x| { x[0] == 2}).unwrap();

        assert!(splits.len() == starts_ends.len());
        for i in  0..4{
            assert!(splits[i].start == starts_ends[i].0 );
            assert!(splits[i].end == starts_ends[i].1 );
        }
    }
}

