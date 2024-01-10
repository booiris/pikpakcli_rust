use std::collections::HashMap;

use anyhow::{Context, Result};
use log::*;
use serde::{Deserialize, Serialize};

use crate::pikpak::RetrySend;

use super::{Client, Resp};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct FileStatus {
    pub kind: String,
    pub id: String,
    pub parent_id: String,
    pub name: String,
    pub user_id: String,
    pub size: String,
    pub file_extension: String,
    pub mime_type: String,
    pub created_time: String,
    pub modified_time: String,
    pub icon_link: String,
    pub thumbnail_link: String,
    pub md5_checksum: String,
    pub hash: String,
    pub phase: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FileType {
    pub kind: String,
    pub id: String,
    pub parent_id: String,
    pub name: String,
    pub user_id: String,
    pub size: String,
    pub revision: String,
    pub file_extension: String,
    pub mime_type: String,
    pub starred: bool,
    pub web_content_link: String,
    pub created_time: String,
    pub modified_time: String,
    pub icon_link: String,
    pub thumbnail_link: String,
    pub md5_checksum: String,
    pub hash: String,
    pub links: Links,
    pub phase: String,
    pub trashed: bool,
    pub delete_time: String,
    pub original_url: String,
    pub original_file_index: i64,
    pub space: String,
    pub writable: bool,
    pub folder_type: String,
    pub sort_name: String,
    pub user_modified_time: String,
    pub file_category: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Links {
    #[serde(rename = "application/octet-stream")]
    pub application_octet_stream: ApplicationOctetStream,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ApplicationOctetStream {
    pub url: String,
    pub token: String,
    pub expire: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct StatusResp {
    pub next_page_token: String,
    pub files: Vec<FileStatus>,
}

impl Client {
    pub async fn get_file_status_list_by_folder_id(
        &mut self,
        folder_id: &str,
    ) -> Result<Vec<FileStatus>> {
        let mut query = HashMap::from([
            ("thumbnail_size", "SIZE_MEDIUM".into()),
            ("limit", "100".into()),
            ("parent_id", folder_id.to_owned()),
            ("with_audit", "false".into()),
            ("filters", r#"{"trashed":{"eq":false}}"#.into()),
        ]);
        let mut file_list = vec![];

        loop {
            let req = self
                .client
                .get("https://api-drive.mypikpak.com/drive/v1/files")
                .query(&query)
                .bearer_auth(&self.jwt_token)
                .header("X-Captcha-Token", &self.captcha_token)
                .header("Content-Type", "application/json");

            debug!("req: {:?}", req);

            match req
                .retry_send(self.retry_times)
                .await
                .context("[get_folder_file_stat_list]")?
                .json::<Resp<StatusResp>>()
                .await
                .context("[get_folder_file_stat_list]")?
            {
                Resp::Success(resp) => {
                    debug!("resp: {:?}", resp);
                    file_list.extend(resp.files);
                    if resp.next_page_token.is_empty() {
                        break;
                    }
                    query.insert("page_token", resp.next_page_token);
                }
                Resp::Err(err) => {
                    if err.error_code == 9 {
                        if let Err(err) =
                            self.auth_captcha_token("GET:/drive/v1/files".into()).await
                        {
                            error!(
                                "[get_folder_file_stat_list] auth captcha token failed, err: {:#?}",
                                err
                            );
                            return Err(anyhow::anyhow!(
                                "[get_folder_file_stat_list] auth captcha token failed"
                            ));
                        }
                    } else {
                        error!("[get_folder_file_stat_list] failed, err: {:#?}", err);
                        return Err(anyhow::anyhow!("[get_folder_file_stat_list]  failed"));
                    }
                }
            }
        }

        Ok(file_list)
    }

    pub async fn get_file_by_id(&mut self, file_id: String) -> Result<FileType> {
        let req = self
            .client
            .get(format!(
                "https://api-drive.mypikpak.com/drive/v1/files/{}",
                file_id
            ))
            .bearer_auth(&self.jwt_token)
            .header("thumbnail_size", "SIZE_MEDIUM")
            .header("X-Captcha-Token", &self.captcha_token)
            .header("X-Device-Id", &self.device_id);

        debug!("req: {:?}", req);

        for _ in 0..2 {
            match req
                .try_clone()
                .ok_or(anyhow::anyhow!("clone request failed"))?
                .retry_send(self.retry_times)
                .await
                .context("[get_file_by_id]")?
                .json::<Resp<FileType>>()
                .await
                .context("[get_file_by_id]")?
            {
                Resp::Success(resp) => {
                    debug!("resp: {:?}", resp);
                    return Ok(resp);
                }
                Resp::Err(err) => {
                    if err.error_code == 9 {
                        if let Err(err) =
                            self.auth_captcha_token("GET:/drive/v1/files".into()).await
                        {
                            error!("[get_file_by_id] failed, err: {:#?}", err);
                            return Err(anyhow::anyhow!("[get_file_by_id]  failed"));
                        }
                    } else {
                        error!("[get_file_by_id] failed, err: {:#?}", err);
                        return Err(anyhow::anyhow!("[get_file_by_id] failed"));
                    }
                }
            }
        }

        Err(anyhow::anyhow!("[get_file_by_id] failed"))
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::load_config, logger::setup_test_logger};

    use super::*;

    #[tokio::test]
    async fn test_get_file_status_list_by_folder_id() -> Result<()> {
        setup_test_logger().ok();
        if load_config("config.yml").is_err() {
            return Ok(());
        }

        if let Ok(mut client) = Client::new(0) {
            client.login().await.ok();
            let res = client.get_file_status_list_by_folder_id("").await;
            info!("{:#?}", res);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_file_by_id() -> Result<()> {
        setup_test_logger().ok();
        if load_config("config.yml").is_err() {
            return Ok(());
        }

        if let Ok(mut client) = Client::new(0) {
            client.login().await.ok();
            let res = client
                // cspell: disable-next-line
                .get_file_by_id("VNnUEooZhMP43acATLjCgCLeo1".to_string())
                .await;
            info!("{:#?}", res);
        }

        Ok(())
    }
}
