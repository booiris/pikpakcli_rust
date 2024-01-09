use std::collections::HashMap;

use anyhow::{Context, Result};
use log::*;
use serde::{Deserialize, Serialize};

use super::{Client, Resp};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct File {
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
struct StatsResp {
    pub next_page_token: String,
    pub files: Vec<File>,
}

impl Client {
    pub async fn get_folder_file_stat_list(&mut self, parent_id: String) -> Result<Vec<File>> {
        let mut query = HashMap::from([
            ("thumbnail_size", "SIZE_MEDIUM".into()),
            ("limit", "100".into()),
            ("parent_id", parent_id.clone()),
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
                .send()
                .await
                .context("[get_folder_file_stat_list]")?
                .json::<Resp<StatsResp>>()
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
                        error!("[get_folder_file_stat_list] get folder file stat list failed, err: {:#?}", err);
                        return Err(anyhow::anyhow!(
                            "[get_folder_file_stat_list] get folder file stat list failed"
                        ));
                    }
                }
            }
        }

        Ok(file_list)
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::load_config, logger::setup_test_logger};

    use super::*;

    #[tokio::test]
    async fn test_get_folder_file_stat_list() -> Result<()> {
        setup_test_logger().ok();
        if load_config("config.yml").is_err() {
            return Ok(());
        }

        if let Ok(mut client) = Client::new() {
            client.login().await.ok();
            let res = client.get_folder_file_stat_list("".to_string()).await;
            info!("{:#?}", res);
        }

        Ok(())
    }
}
