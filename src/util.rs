use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

/// Read a file's contents to a string
pub fn read_file_to_string(path: &Path) -> String {
    let mut contents = String::new();
    if let Ok(mut file) = File::open(path) {
        while let Ok(read) = file.read_to_string(&mut contents)
        {
            trace!("read {} from {:?}", read, path);
            if read == 0 {
                break;
            }
        }
    }
    contents
}
