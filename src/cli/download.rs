use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crate::pikpak::download::download_with_file;
use crate::pikpak::file::FileType;
use crate::pikpak::Client;
use crate::utils::file::create_dir_if_not_exists;
use crate::utils::path::slash;
use anyhow::Context;
use anyhow::Ok;
use anyhow::Result;
use async_recursion::async_recursion;
use log::*;
use tokio::fs;
use tokio::fs::File;
use tokio::sync::Semaphore;

impl Client {
    pub async fn download(
        mut self,
        mut paths: Vec<String>,
        output: String,
        parallel: usize,
    ) -> Result<()> {
        let output_dir = Path::new(&output);
        create_dir_if_not_exists(output_dir)?;
        let mut tasks: Vec<(String, PathBuf)> = Vec::new();
        if paths.is_empty() {
            paths = vec!["/".into()];
        }
        for path in paths {
            debug!("finding path: {}", path);
            let id = self.get_path_id(&path).await?;
            match id {
                crate::pikpak::folder::FileIDType::File(id) => {
                    let path = slash(&path)?;
                    tasks.push((id, output_dir.join(path)))
                }
                crate::pikpak::folder::FileIDType::Folder(id) => {
                    self.recursive_get_file(&mut tasks, id, PathBuf::from(path), output_dir)
                        .await?
                }
            }
        }

        debug!("tasks: {:#?}", tasks);

        let semaphore = Arc::new(Semaphore::new(parallel));
        let mut threads = vec![];
        for task in tasks {
            let permit = semaphore.clone().acquire_owned().await?;

            let file_info = self.get_file_by_id(task.0).await?;
            let retry_times = self.retry_times;

            threads.push(tokio::spawn(async move {
                let _permit = permit;
                if let Err(err) = download_file(file_info, task.1, retry_times).await {
                    error!("download file failed, err: {:#?}", err);
                }
            }));
        }

        futures::future::join_all(threads).await;

        Ok(())
    }

    #[async_recursion(?Send)]
    async fn recursive_get_file(
        &mut self,
        tasks: &mut Vec<(String, PathBuf)>,
        parent_id: String,
        parent_path: PathBuf,
        output_dir: &Path,
    ) -> Result<()> {
        debug!(
            "recursive get file, parent_id: {}, parent_path: {}",
            parent_id,
            parent_path.display()
        );
        let status_list = self.get_file_status_list_by_folder_id(&parent_id).await?;

        for status in status_list {
            if status.kind == "drive#folder" {
                if let Err(err) = self
                    .recursive_get_file(tasks, status.id, parent_path.join(status.name), output_dir)
                    .await
                {
                    error!("recursive get file failed, err: {:#?}", err);
                }
            } else {
                let parent_path = slash(
                    parent_path
                        .to_str()
                        .ok_or(anyhow::anyhow!("slash parent path error"))?,
                )?;
                if parent_path.starts_with('/') {
                    return Err(anyhow::anyhow!(
                        "parent path starts with /, parent_path: {}",
                        parent_path
                    ));
                }
                tasks.push((status.id, output_dir.join(&parent_path).join(status.name)));
            }
        }

        Ok(())
    }
}

async fn download_file(file: FileType, output_path: PathBuf, retry_times: i8) -> Result<()> {
    info!("downloading file: {:#?}", output_path);

    let file_dir = output_path
        .parent()
        .ok_or(anyhow::anyhow!("[download_file] failed to get parent dir"))?;
    create_dir_if_not_exists(file_dir)?;

    let exist = output_path
        .try_exists()
        .context("[download_file] get file exists err")?;

    let mut flag_path = output_path.clone();
    let ex = flag_path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or_default()
        .to_string();
    if !flag_path.set_extension(ex + ".pikpakclidownload") {
        return Err(anyhow::anyhow!("[download_file] failed to set extension"));
    }
    let flag = flag_path
        .try_exists()
        .context("[download_file] get flag err")?;

    if exist && !flag {
        info!(
            "[download_file] file exists and not download flag, skip: {}",
            output_path.display()
        );
        return Ok(());
    }
    if !flag {
        File::create(&flag_path)
            .await
            .context("[download_file] failed to create flag file")?;
    }

    download_with_file(output_path.as_path(), file, retry_times).await?;

    fs::remove_file(&flag_path)
        .await
        .context("[download_file] failed to remove flag")?;
    info!("file downloaded: {:#?}", output_path);
    Ok(())
}
