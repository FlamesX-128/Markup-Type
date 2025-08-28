use std::io::Read;

pub struct FileReader {
    reader: std::io::BufReader<std::fs::File>,
}

impl Iterator for FileReader {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = [0; 1];

        match self.reader.read(&mut buffer) {
            Err(_) => {
                unreachable!()
            }
            Ok(0) => None,
            Ok(_) => Some(buffer[0] as char),
        }
    }
}

impl FileReader {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);

        Ok(Self { reader })
    }
}
