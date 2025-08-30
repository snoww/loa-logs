use std::path::PathBuf;

pub trait RegionAcessor {
    fn get(&self) -> Option<String>;
}

pub struct DefaultRegionAccessor(PathBuf);
pub struct ProcessRegionAccessor;
pub struct FakeRegionAccessor(String);

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

impl RegionAcessor for FakeRegionAccessor {
    fn get(&self) -> Option<String> {
        Some(self.0.clone())
    }
}

impl DefaultRegionAccessor {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl FakeRegionAccessor {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}
