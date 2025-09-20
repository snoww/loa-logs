use anyhow::Result;
use std::path::{Path, PathBuf};
use winsafe::{self as w, co, prelude::*};

pub struct AutoLaunchManager {
    task_name: String,
    app_path: PathBuf,
}

impl AutoLaunchManager {
    pub fn new(_app_name: &str, app_path: &Path) -> Self {
        Self {
            task_name: "LOA_Logs_Auto_Start".to_string(),
            app_path: app_path.to_path_buf(),
        }
    }
}

impl super::AutoLaunch for AutoLaunchManager {
    fn is_enabled(&self) -> Result<bool> {
        let _scope = w::CoInitializeEx(co::COINIT::MULTITHREADED);
        let service = w::CoCreateInstance::<w::ITaskService>(
            &co::CLSID::TaskScheduler,
            None::<&w::IUnknown>,
            co::CLSCTX::INPROC_SERVER,
        )?;
        service.Connect(None, None, None, None)?;
        let folder = service.GetFolder(r"\")?;
        let is_enabled = match folder.GetTask(&self.task_name) {
            Ok(t) => t.get_Enabled()?,
            Err(_) => false,
        };
        Ok(is_enabled)
    }

    fn enable(&self) -> Result<()> {
        let _scope = w::CoInitializeEx(co::COINIT::MULTITHREADED);
        let service = w::CoCreateInstance::<w::ITaskService>(
            &co::CLSID::TaskScheduler,
            None::<&w::IUnknown>,
            co::CLSCTX::INPROC_SERVER,
        )?;
        service.Connect(None, None, None, None)?;
        let folder = service.GetFolder(r"\")?;
        let task = service.NewTask()?;

        task.get_Triggers()?.Create(co::TASK_TRIGGER_TYPE2::LOGON)?;
        task.get_Principal()?
            .put_RunLevel(co::TASK_RUNLEVEL_TYPE::HIGHEST)?;

        let action = task
            .get_Actions()?
            .Create(co::TASK_ACTION_TYPE::EXEC)?
            .QueryInterface::<w::IExecAction>()?;
        action.put_Path(&self.app_path.display().to_string())?;
        let working_directory = self.app_path.parent().expect("should have parent");
        action.put_WorkingDirectory(&working_directory.display().to_string())?;

        folder.RegisterTaskDefinition(
            Some(&self.task_name),
            &task,
            co::TASK_CREATION::CREATE_OR_UPDATE,
            None,
            None,
            co::TASK_LOGON::INTERACTIVE_TOKEN,
            None,
        )?;
        Ok(())
    }

    fn disable(&self) -> Result<()> {
        let _scope = w::CoInitializeEx(co::COINIT::MULTITHREADED);
        let service = w::CoCreateInstance::<w::ITaskService>(
            &co::CLSID::TaskScheduler,
            None::<&w::IUnknown>,
            co::CLSCTX::INPROC_SERVER,
        )?;
        service.Connect(None, None, None, None)?;
        service.GetFolder(r"\")?.DeleteTask(&self.task_name)?;
        Ok(())
    }
}
