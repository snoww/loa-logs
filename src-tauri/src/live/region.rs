use std::path::PathBuf;

pub trait RegionAcessor {
    fn get(&self) -> Option<String>;
}

pub struct DefaultRegionAccessor(PathBuf);
pub struct ProcessRegionAccessor;

impl RegionAcessor for ProcessRegionAccessor {
    fn get(&self) -> Option<String> {
        None
    }
}

impl RegionAcessor for DefaultRegionAccessor {
    fn get(&self) -> Option<String> {
        std::fs::read_to_string(&self.0).ok()
    }
}

impl DefaultRegionAccessor {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}
