use anyhow::Result;
use reqwest::{header, RequestBuilder};
use serde::{Deserialize, Serialize};

use crate::config::get_config;

mod captcha_token;
pub mod download;
pub mod file;
pub mod folder;
mod login;

#[derive(Debug, Default)]
pub struct Client {
    account: String,
    password: String,
    jwt_token: String,
    refresh_token: String,
    captcha_token: String,
    sub: String,
    device_id: String,
    refresh_second: i64,
    client: reqwest::Client,
    pub retry_times: i8,
}

const USER_AGENT: &str = "ANDROID-com.pikcloud.pikpak/1.21.0";
const CLIENT_ID: &str = "YNxT9w7GMdWvEOKa";
const CLIENT_SECRET: &str = "dbw2OtmVEeuUvIptb1Coyg";

impl Client {
    pub fn new(retry_times: i8) -> Result<Self> {
        let account = get_config().username.clone();
        let password = get_config().password.clone();
        let device_id = format!("{:x}", md5::compute(&account));

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/json; charset=utf-8"),
        );
        headers.insert("User-Agent", header::HeaderValue::from_static(USER_AGENT));
        headers.insert(
            "X-Device-Id",
            device_id.parse().expect("parse device id header failed"),
        );

        let mut client_builder: reqwest::ClientBuilder =
            reqwest::Client::builder().default_headers(headers);
        if let Some(proxy) = get_config().proxy.as_ref() {
            client_builder = client_builder.proxy(reqwest::Proxy::all(proxy)?);
        }
        let client = client_builder.build()?;

        Ok(Client {
            account,
            password,
            client,
            device_id,
            retry_times,
            ..Default::default()
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrResp {
    pub error: String,
    #[serde(rename = "error_code")]
    pub error_code: i64,
    #[serde(rename = "error_url")]
    pub error_url: String,
    #[serde(rename = "error_description")]
    pub error_description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum Resp<T> {
    Success(T),
    Err(ErrResp),
}

pub trait RetrySend {
    async fn retry_send(self, retry_times: i8) -> Result<reqwest::Response>;
}

impl RetrySend for RequestBuilder {
    async fn retry_send(self, retry_times: i8) -> Result<reqwest::Response> {
        if retry_times < 0 {
            let mut cnt = 0;
            loop {
                match self
                    .try_clone()
                    .ok_or(anyhow::anyhow!("clone request failed"))?
                    .send()
                    .await
                {
                    Ok(resp) => return Ok(resp),
                    Err(err) => {
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        log::warn!("request failed, err: {}, retry times: {}", err, cnt + 1);
                        cnt += 1;
                    }
                }
            }
        } else {
            for i in 0..=retry_times {
                match self
                    .try_clone()
                    .ok_or(anyhow::anyhow!("clone request failed"))?
                    .send()
                    .await
                {
                    Ok(resp) => return Ok(resp),
                    Err(err) => {
                        if i == retry_times {
                            return Err(err.into());
                        }
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        log::warn!("request failed, err: {}, retry times: {}", err, i + 1);
                    }
                }
            }
            panic!("retry send failed")
        }
    }
}
