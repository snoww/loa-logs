use std::{os::windows::process::CommandExt, process::Command};
use anyhow::Result;
use log::*;

use crate::constants::TASK_NAME;

const CREATE_NO_WINDOW: u32 = 0x08000000;

const IS_ENABLED_SCRIPT: &'static str = r#"Get-ScheduledTask -TaskName "{name}""#;

const UNREGISTER_SCRIPT: &'static str = r#"Unregister-ScheduledTask -TaskName "{name}" -Confirm:$false"#;

const REGISTER_SCRIPT: &'static str = r#"
$action = New-ScheduledTaskAction `
    -Execute "{exe}" `
    -WorkingDirectory (Split-Path "{exe}")

$trigger = New-ScheduledTaskTrigger -AtLogOn

Register-ScheduledTask `
    -TaskName "{name}" `
    -Action $action `
    -Trigger $trigger `
    -RunLevel Highest `
    -Force
"#;

pub struct AutoLaunchManager {
    task_name: String,
    app_path: String,
}

impl AutoLaunchManager {
    pub fn new(app_path: &str) -> Self {
        Self {
            task_name: TASK_NAME.to_string(),
            app_path: app_path.to_string(),
        }
    }

    fn run_powershell(script: &str) -> Result<bool> {
        let output = Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW) 
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .output()?;

        Ok(output.status.success())
    }

    fn format_script(template: &str, name: &str, exe: Option<&str>) -> String {
        let str = template.replace("{name}", name);
        if let Some(exe) = exe {
            str.replace("{exe}", exe)
        } else {
            str
        }
    }

    fn unregister_task(&self) -> Result<()> {
        let script = Self::format_script(UNREGISTER_SCRIPT, &self.task_name, None);
        Self::run_powershell(&script)?;
        Ok(())
    }
}

impl super::AutoLaunch for AutoLaunchManager {
    fn is_enabled(&self) -> Result<bool> {
        let script = Self::format_script(IS_ENABLED_SCRIPT, &self.task_name, None);
        Self::run_powershell(&script)
    }

    fn enable(&self) -> Result<()> {
        self.unregister_task()?;

        let script = Self::format_script(REGISTER_SCRIPT, &self.task_name, Some(&self.app_path));

        match Self::run_powershell(&script) {
            Ok(true) => info!("Enabled start on boot"),
            Ok(false) => warn!("Failed to enable start on boot"),
            Err(e) => warn!("Error enabling start on boot: {}", e),
        }

        Ok(())
    }

    fn disable(&self) -> Result<()> {
        match self.unregister_task() {
            Ok(_) => info!("Disabled start on boot"),
            Err(e) => warn!("Error disabling start on boot: {}", e),
        }
        Ok(())
    }
}
