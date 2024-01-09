use anyhow::Result;
use reqwest::header;
use serde::{Deserialize, Serialize};

use crate::config::get_config;

mod captcha_token;
pub mod file;
mod folder;
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
}

const USER_AGENT: &str = "ANDROID-com.pikcloud.pikpak/1.21.0";
const CLIENT_ID: &str = "YNxT9w7GMdWvEOKa";
const CLIENT_SECRET: &str = "dbw2OtmVEeuUvIptb1Coyg";

impl Client {
    pub fn new() -> Result<Self> {
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
