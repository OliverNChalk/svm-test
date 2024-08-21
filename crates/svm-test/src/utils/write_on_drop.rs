use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use flate2::write::GzEncoder;
use serde::Serialize;
use tracing::error;

#[derive(Debug)]
pub struct WriteOnDrop<T>
where
    T: Serialize,
{
    dirty: bool,
    pub data: T,
    pub path: Option<PathBuf>,
}

impl<T> WriteOnDrop<T>
where
    T: Serialize,
{
    pub fn new(data: T, path: Option<PathBuf>) -> Self {
        WriteOnDrop { dirty: false, data, path }
    }
}

impl<T> Deref for WriteOnDrop<T>
where
    T: Serialize,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for WriteOnDrop<T>
where
    T: Serialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty = true;
        &mut self.data
    }
}

impl<T> Drop for WriteOnDrop<T>
where
    T: Serialize,
{
    fn drop(&mut self) {
        if self.dirty {
            if let Some(path) = &self.path {
                try_write_json_gz(path, &self.data);
            }
        }
    }
}

pub fn try_write_json_gz<T>(path: &Path, data: &T)
where
    T: Serialize,
{
    let file = match std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
    {
        Ok(file) => file,
        Err(err) => {
            error!("Failed to write to file; path={path:?}; err={err}");
            return;
        }
    };
    let compression = GzEncoder::new(file, flate2::Compression::best());

    match serde_json::to_writer(compression, &data) {
        Ok(serialized) => serialized,
        Err(err) => {
            error!("Failed to serialize data; path={path:?}; err={err}");
        }
    }
}
