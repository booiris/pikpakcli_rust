use std::path::Path;
use std::sync::OnceLock;

use anyhow::Context;
use anyhow::Result;
use futures::StreamExt;
use humansize::DECIMAL;
use log::debug;
use log::info;
use log::warn;
use reqwest::Client;
use tokio::fs::File;
use tokio::fs::OpenOptions;

use crate::pikpak::RetrySend;

use super::file::FileType;
use super::USER_AGENT;

fn get_download_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("[get_download_client] build client failed")
    })
}

pub async fn download_with_file(path: &Path, file: FileType, retry_times: i8) -> Result<()> {
    let mut opt = OpenOptions::new();
    opt.create(true).append(true);

    #[cfg(unix)]
    opt.mode(0o644);

    let mut out_file = opt.open(path).await?;
    let size = out_file.metadata().await?.len();
    let resume = size != 0;
    let mut req = get_download_client()
        .get(file.links.application_octet_stream.url)
        .header("User-Agent", USER_AGENT);
    if resume {
        info!(
            "resuming from {} bytes, file: {:?}",
            humansize::format_size(size, DECIMAL),
            path
        );
        req = req.header("Range", format!("bytes={}-", size));
    }

    debug!("req: {:?}", req);

    let resp = req
        .retry_send(retry_times)
        .await
        .context("[download_with_file]")?;

    if resume && resp.status() != 206 {
        warn!(
            "resume failed, status: {}, restarting from the beginning",
            resp.status()
        );
        drop(out_file);
        out_file = File::create(path).await?;
    }

    let content_len = resp
        .headers()
        .get("Content-Length")
        .ok_or(anyhow::anyhow!(
            "[download_with_file] Content-Length header not found"
        ))?
        .to_str()
        .context("[download_with_file]")?
        .parse::<u64>()
        .context("[download_with_file]")?;

    let mut stream = resp.bytes_stream();

    let mut copy_cnt = 0;
    while let Some(item) = stream.next().await {
        copy_cnt += tokio::io::copy(
            &mut item.context("[download_with_file] stream error")?.as_ref(),
            &mut out_file,
        )
        .await
        .context("[download_with_file] copy error")?;
    }

    if copy_cnt != content_len {
        return Err(anyhow::anyhow!(
                "[download_with_file] content length not equal to written, copy_cnt: {}, content_len: {}",
                copy_cnt,
                content_len
            ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::Result;
    use log::info;

    use crate::{
        config::load_config,
        logger::setup_test_logger,
        pikpak::{download::download_with_file, Client},
    };

    #[tokio::test]
    async fn test_download_with_file() -> Result<()> {
        setup_test_logger().ok();
        if load_config("config.yml").is_err() {
            return Ok(());
        }

        if let Ok(mut client) = Client::new(0) {
            client.login().await.ok();
            if let Ok(file) = client
                // cspell: disable-next-line
                .get_file_by_id("VNhqdoPCnu4swXeYNFIj6O1Po1".into())
                .await
            {
                let res = download_with_file(Path::new("test.mp4"), file, 0).await;
                info!("{:#?}", res);
            }
        }

        Ok(())
    }
}
