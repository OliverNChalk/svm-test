use std::io::BufReader;
use std::path::Path;

use flate2::read::GzDecoder;
use serde::de::DeserializeOwned;

pub fn read_json<T>(path: &Path) -> T
where
    T: DeserializeOwned,
{
    let bytes = BufReader::new(open_read(path));

    serde_json::from_reader(bytes).unwrap()
}

pub fn read_json_gz<T>(path: &Path) -> T
where
    T: DeserializeOwned,
{
    let compressed = open_read(path);
    let bytes = BufReader::new(GzDecoder::new(compressed));

    serde_json::from_reader(bytes).unwrap()
}

fn open_read(path: &Path) -> std::fs::File {
    std::fs::OpenOptions::new()
        .read(true)
        .open(path)
        .unwrap_or_else(|err| panic!("Failed to open file; path={path:?}; err={err}"))
}
