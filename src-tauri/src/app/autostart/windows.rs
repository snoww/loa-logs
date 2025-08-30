use crate::app::compat::Command;
use anyhow::Result;
use log::{info, warn};

pub struct AutoLaunchManager {
    task_name: String,
    app_path: String,
}

impl AutoLaunchManager {
    pub fn new(_app_name: &str, app_path: &str) -> Self {
        Self {
            task_name: "LOA_Logs_Auto_Start".to_string(),
            app_path: app_path.to_string(),
        }
    }
}

impl super::AutoLaunch for AutoLaunchManager {
    fn is_enabled(&self) -> Result<bool> {
        let output = Command::new("schtasks")
            .args(["/query", "/tn", &self.task_name])
            .output();

        let result = match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };
        Ok(result)
    }

    fn enable(&self) -> Result<()> {
        Command::new("schtasks")
            .args(["/delete", "/tn", &self.task_name, "/f"])
            .output()
            .ok();

        let output = Command::new("schtasks")
            .args([
                "/create",
                "/tn",
                &self.task_name,
                "/tr",
                &format!("\"{}\"", &self.app_path),
                "/sc",
                "onlogon",
                "/rl",
                "highest",
            ])
            .output();

        match output {
            Ok(o) if o.status.success() => info!("enabled start on boot"),
            Ok(e) => warn!("error enabling start on boot: {:?}", e),
            Err(e) => warn!("error enabling start on boot: {}", e),
        };
        Ok(())
    }

    fn disable(&self) -> Result<()> {
        let output = Command::new("schtasks")
            .args(["/delete", "/tn", &self.task_name, "/f"])
            .output();

        match output {
            Ok(o) if o.status.success() => info!("disabled start on boot"),
            Ok(e) => warn!("error disabling start on boot: {:?}", e),
            Err(e) => warn!("error disabling start on boot: {}", e),
        };
        Ok(())
    }
}
