use std::{path::PathBuf, fs::{self, File}, io};
use anyhow::{Result, Context};

use log::info;

pub struct Resources {
    resource_path: PathBuf,
}

impl Resources {
    pub fn new(path: PathBuf) -> Self {
        Self {
            resource_path: path.join("assets/resources.zip"),
        }
    }

    pub fn extract(&self) -> Result<()> {
        if !self.check_if_file_exists() {
            info!("resources loaded");
            return Ok(());
        }

        info!("extracting resources");
        let zip = File::open(&self.resource_path)
            .context("failed to open the resource zip file")?;
        let mut archive = zip::ZipArchive::new(zip)
            .context("failed to create a ZipArchive from the resource zip file")?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .context("failed to read file from ZipArchive")?;
            let outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath)
                    .context("failed to create directory")?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)
                            .context("failed to create directory")?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)
                    .context("failed to create file for extraction")?;
                io::copy(&mut file, &mut outfile)
                    .context("failed to write data to file")?;
            }
        }

        self.delete_zip()?;

        info!("finished extracting resources");
        Ok(())
    }

    fn check_if_file_exists(&self) -> bool {
        self.resource_path.exists()
    }

    fn delete_zip(&self) -> Result<()> {
        fs::remove_file(&self.resource_path)
            .context("Failed to delete the resource zip file")?;
        Ok(())
    }
}