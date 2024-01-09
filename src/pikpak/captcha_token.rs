use std::sync::OnceLock;

use anyhow::{Context, Result};
use chrono::Utc;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::pikpak::Resp;

use super::{Client, CLIENT_ID};

const PACKAGE_NAME: &str = "com.pikcloud.pikpak";
const CLIENT_VERSION: &str = "1.21.0";

#[derive(Default, Debug, Serialize, Deserialize)]
struct Md5Obj {
    alg: String,
    salt: String,
}

fn get_md5_object() -> &'static [Md5Obj] {
    static DATA: OnceLock<Vec<Md5Obj>> = OnceLock::new();
    DATA.get_or_init(||{
        // cspell:disable-next-line
        let md5_obj = r#"[{"alg":"md5","salt":""},{"alg":"md5","salt":"E32cSkYXC2bciKJGxRsE8ZgwmH\/YwkvpD6\/O9guSOa2irCwciH4xPHaH"},{"alg":"md5","salt":"QtqgfMgHP2TFl"},{"alg":"md5","salt":"zOKgHT56L7nIzFzDpUGhpWFrgP53m3G6ML"},{"alg":"md5","salt":"S"},{"alg":"md5","salt":"THxpsktzfFXizUv7DK1y\/N7NZ1WhayViluBEvAJJ8bA1Wr6"},{"alg":"md5","salt":"y9PXH3xGUhG\/zQI8CaapRw2LhldCaFM9CRlKpZXJvj+pifu"},{"alg":"md5","salt":"+RaaG7T8FRTI4cP019N5y9ofLyHE9ySFUr"},{"alg":"md5","salt":"6Pf1l8UTeuzYldGtb\/d"}]"#;
        serde_json::from_str(md5_obj).expect("parse md5 object failed")
    })
}

impl Client {
    pub async fn auth_captcha_token(&mut self, action: String) -> Result<()> {
        let ts = Utc::now().timestamp_millis().to_string();
        let mut key = CLIENT_ID.to_string() + CLIENT_VERSION + PACKAGE_NAME + &self.device_id + &ts;
        for obj in get_md5_object() {
            if obj.alg == "md5" {
                key = format!("{:x}", md5::compute(key + &obj.salt));
            }
        }
        let body = json!({
            "action": action,
            "captcha_token": self.captcha_token,
            "client_id": CLIENT_ID,
            "device_id": self.device_id,
            "redirect_uri": "ttps://api.mypikpak.com/v1/auth/callback",
            "meta": {
                "captcha_sign": "1.".to_string() + &key,
                "user_id": self.sub,
                "package_name": PACKAGE_NAME,
                "client_version": CLIENT_VERSION,
                "timestamp": ts,
            }
        });
        let req = self
            .client
            .post("https://user.mypikpak.com/v1/shield/captcha/init")
            .query(&[("client_id", CLIENT_ID)])
            .bearer_auth(&self.jwt_token)
            .json(&body);

        debug!("req: {:?}", req);

        match req
            .send()
            .await
            .context("[auth_captcha_token]")?
            .json::<Resp<Value>>()
            .await?
        {
            Resp::Success(resp) => {
                debug!("resp: {:?}", resp);
                self.captcha_token = resp
                    .get("captcha_token")
                    .and_then(|x| x.as_str())
                    .map(|x| x.to_string())
                    .unwrap_or_default();
                Ok(())
            }
            Resp::Err(err) => {
                error!("[auth_captcha_token] failed, err: {:#?}", err);
                Err(anyhow::anyhow!("[auth_captcha_token] failed"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::load_config;
    use crate::logger::setup_test_logger;

    use super::*;

    #[tokio::test]
    async fn test_auth_captcha_token() -> Result<()> {
        setup_test_logger().ok();
        if load_config("config.yml").is_err() {
            return Ok(());
        }

        if let Ok(mut client) = Client::new() {
            client.login().await.ok();
            let res = client
                .auth_captcha_token("GET:/drive/v1/files".into())
                .await;
            info!("{:#?}", res);
        }
        Ok(())
    }
}
