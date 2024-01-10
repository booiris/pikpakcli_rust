use anyhow::Result;
use colored::Colorize;
use humansize::format_size;
use humansize::DECIMAL;
use log::debug;

use crate::pikpak::file::FileStatus;

use super::Client;

enum Mode {
    Default,
    Long,
    LongHuman,
}

impl Client {
    pub async fn list(mut self, long: bool, human: bool, path: String) -> Result<()> {
        let parent_id = self.get_path_id(&path).await?;
        debug!("get parent_id: {:?}", parent_id);
        let files = self
            .get_file_status_list_by_folder_id(parent_id.get_id())
            .await?;
        for file in files {
            if long {
                if human {
                    display(Mode::LongHuman, file);
                } else {
                    display(Mode::Long, file);
                }
            } else {
                display(Mode::Default, file);
            }
        }
        Ok(())
    }
}

fn display(mode: Mode, file: FileStatus) {
    match mode {
        Mode::Default => {
            if file.kind == "drive#folder" {
                println!("{:<20}", file.name.green());
            } else {
                println!("{:<20}", file.name);
            }
        }
        Mode::Long => {
            if file.kind == "drive#folder" {
                println!(
                    "{:<26} {:<6} {:<14} {}",
                    file.id,
                    file.size,
                    file.created_time,
                    file.name.green()
                );
            } else {
                println!(
                    "{:<26} {:<6} {:<14} {}",
                    file.id, file.size, file.created_time, file.name
                );
            }
        }
        Mode::LongHuman => {
            if file.kind == "drive#folder" {
                println!(
                    "{:<26} {:<6} {:<14} {}",
                    file.id,
                    format_size(file.size.parse::<u64>().unwrap_or_default(), DECIMAL),
                    file.created_time,
                    file.name.green()
                );
            } else {
                println!(
                    "{:<26} {:<6} {:<14} {}",
                    file.id,
                    format_size(file.size.parse::<u64>().unwrap_or_default(), DECIMAL),
                    file.created_time,
                    file.name
                );
            }
        }
    }
}
