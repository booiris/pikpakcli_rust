use anyhow::{Context, Result};
use log::*;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{pikpak::RetrySend, utils::path::slash};

use super::{Client, Resp};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct GetFolderResp {
    pub files: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileIDType {
    #[serde(rename = "drive#file")]
    File(String),
    #[serde(rename = "drive#folder")]
    Folder(String),
}

impl FileIDType {
    pub fn get_id(&self) -> &String {
        match self {
            FileIDType::File(id) => id,
            FileIDType::Folder(id) => id,
        }
    }
}

impl Client {
    pub async fn get_path_id(&mut self, path: &str) -> Result<FileIDType> {
        self.get_deep_folder_id(FileIDType::Folder("".into()), path)
            .await
    }

    // 获取文件夹 id
    // dir 可以包括 /.
    // 若以 / 开头，函数会去除 /， 且会从 parent 目录开始查找
    pub async fn get_deep_folder_id(
        &self,
        mut parent_id: FileIDType,
        path: &str,
    ) -> Result<FileIDType> {
        let dir_path = slash(path).context("[get_deep_folder_id]")?;
        if dir_path.is_empty() {
            return Ok(parent_id);
        }

        for dir in dir_path.split('/') {
            parent_id = self
                .get_sub_folder_id(parent_id.get_id(), dir)
                .await
                .context("[get_deep_folder_id]")?;
            debug!("get folder: {}, folder id: {:?}", dir, parent_id);
        }

        Ok(parent_id)
    }

    async fn get_sub_folder_id(&self, parent_id: &str, path: &str) -> Result<FileIDType> {
        let dir = slash(path).context("get_folder_id")?;

        let infos = self.get_info_by_id(parent_id).await?;
        for file in infos {
            let kind = file
                .get("kind")
                .and_then(|x| x.as_str())
                .unwrap_or_default();
            let name = file
                .get("name")
                .and_then(|x| x.as_str())
                .unwrap_or_default();
            let trashed = file
                .get("trashed")
                .and_then(|x| x.as_bool())
                .unwrap_or_default();
            if name == dir && !trashed {
                let id = file
                    .get("id")
                    .ok_or(anyhow::anyhow!("[get_folder_id] id not found"))?
                    .as_str()
                    .ok_or(anyhow::anyhow!(
                        "[get_folder_id] id can not parse to string"
                    ))?
                    .to_string();
                if kind == "drive#folder" {
                    return Ok(FileIDType::Folder(id));
                } else {
                    return Ok(FileIDType::File(id));
                };
            }
        }
        Err(anyhow::anyhow!("[get_folder_id] folder not found"))
    }

    async fn get_info_by_id(&self, parent_id: &str) -> Result<Vec<Value>> {
        let query = [
            ("parent_id", parent_id),
            ("page_token", ""),
            ("with_audit", "false"),
            ("thumbnail_size", "SIZE_LARGE"),
            ("limit", "200"),
        ];

        let mut headers = HeaderMap::new();
        headers.insert("Country", "CN".parse()?);
        headers.insert("X-Peer-Id", self.device_id.parse()?);
        headers.insert("X-User-Region", "1".parse()?);
        headers.insert("X-Alt-Capability", "3".parse()?);
        headers.insert("X-Client-Version-Code", "10083".parse()?);
        headers.insert("X-Captcha-Token", self.captcha_token.parse()?);

        let req = self
            .client
            .get("https://api-drive.mypikpak.com/drive/v1/files")
            .query(&query)
            .headers(headers)
            .bearer_auth(&self.jwt_token);

        debug!("req: {:?}", req);

        match req
            .retry_send(self.retry_times)
            .await
            .context("[get_folder_id]")?
            .json::<Resp<GetFolderResp>>()
            .await
            .context("[get_folder_id]")?
        {
            Resp::Success(resp) => {
                debug!("resp: {:?}", resp);
                Ok(resp.files)
            }
            Resp::Err(err) => {
                error!("[get_folder_id] get folder failed, err: {:#?}", err);
                Err(anyhow::anyhow!("[get_folder_id] get folder failed"))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{config::load_config, logger::setup_test_logger};

    #[tokio::test]
    async fn test_get_path_folder_id() -> Result<()> {
        setup_test_logger().ok();
        if load_config("config.yml").is_err() {
            return Ok(());
        }

        if let Ok(mut client) = Client::new(0) {
            client.login().await.ok();
            let res = client.get_path_id("My Pack").await.ok();
            log::info!("{:?}", res);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_folder_info_by_folder_id() -> Result<()> {
        setup_test_logger().ok();
        if load_config("config.yml").is_err() {
            return Ok(());
        }

        if let Ok(mut client) = Client::new(0) {
            client.login().await.ok();
            let res = client
                .get_info_by_id("VNnUEooZhMP43acATLjCgCLeo1")
                .await
                .ok();
            log::info!("{:?}", res);
        }

        Ok(())
    }
}
