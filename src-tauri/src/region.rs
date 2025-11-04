use std::{fmt::Debug, path::PathBuf};

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait RegionAccessor : Debug {
    fn get(&self) -> Option<String>;
}

#[derive(Debug)]
pub struct SavedToFileRegionAccessor(PathBuf);

impl RegionAccessor for SavedToFileRegionAccessor {
    fn get(&self) -> Option<String> {
        std::fs::read_to_string(&self.0).ok()
    }
}

impl SavedToFileRegionAccessor {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}