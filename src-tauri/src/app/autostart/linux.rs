use anyhow::Result;
use auto_launch::{AutoLaunch, AutoLaunchBuilder};

pub struct AutoLaunchManager(AutoLaunch);

impl AutoLaunchManager {
    pub fn new(app_name: &str, app_path: &str) -> Self {
        let auto = AutoLaunchBuilder::new()
            .set_app_name(app_name)
            .set_app_path(app_path)
            .build()
            .unwrap();
        Self(auto)
    }
}

impl super::AutoLaunch for AutoLaunchManager {
    fn is_enabled(&self) -> Result<bool> {
        self.0.is_enabled().map_err(|e| anyhow::anyhow!(e))
    }

    fn enable(&self) -> Result<()> {
        self.0.enable().map_err(|e| anyhow::anyhow!(e))
    }

    fn disable(&self) -> Result<()> {
        self.0.disable().map_err(|e| anyhow::anyhow!(e))
    }
}
