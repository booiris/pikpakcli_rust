use crate::pikpak::RetrySend;

use super::{Client, Resp, CLIENT_ID, CLIENT_SECRET};
use anyhow::{Context, Result};
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct LoginResp {
    pub token_type: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub sub: String,
}

impl Client {
    pub async fn login(&mut self) -> Result<()> {
        let mut req = self
            .client
            .post("https://user.mypikpak.com/v1/auth/signin?client_id=".to_string() + CLIENT_ID)
            .json(&json!({
                "captcha_token": self.captcha_token,
                "client_id": CLIENT_ID,
                "client_secret": CLIENT_SECRET,
                "username": self.account,
                "password": self.password,
            }));

        if !self.jwt_token.is_empty() {
            req = req.bearer_auth(&self.jwt_token);
        }

        debug!("req: {:?}", req);

        match req
            .retry_send(self.retry_times)
            .await
            .context("[login]")?
            .json::<Resp<LoginResp>>()
            .await
            .context("[login]")?
        {
            Resp::Success(resp) => {
                debug!("resp: {:?}", resp);
                self.jwt_token = resp.access_token;
                self.refresh_token = resp.refresh_token;
                self.sub = resp.sub;
                self.refresh_second = resp.expires_in;
                Ok(())
            }
            Resp::Err(err) => {
                error!("login failed, err: {:#?}", err);
                Err(anyhow::anyhow!("login failed"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::load_config, logger::setup_test_logger};

    use super::*;

    #[tokio::test]
    async fn test_login() -> Result<()> {
        setup_test_logger().ok();
        if load_config("config.yml").is_err() {
            return Ok(());
        }

        if let Ok(mut client) = Client::new(0) {
            client.login().await.ok();
        }

        Ok(())
    }
}
