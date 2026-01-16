use std::{fs::read_to_string, path::PathBuf};


pub trait RegionAccessor {
    fn get(&self) -> Option<String>;
}

pub struct DefaultRegionAccessor(PathBuf);

impl RegionAccessor for DefaultRegionAccessor {
    fn get(&self) -> Option<String> {
        read_to_string(&self.0).ok()
    }
}

impl DefaultRegionAccessor {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}