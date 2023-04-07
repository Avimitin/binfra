use anyhow::Context;
use std::{fmt::Display, ops::Deref};

use reqwest::{IntoUrl, StatusCode};
use serde::de::DeserializeOwned;

#[derive(Clone, Default)]
pub struct HttpClient(reqwest::Client);

impl Deref for HttpClient {
    type Target = reqwest::Client;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl HttpClient {
    pub fn new() -> Self {
        Self(reqwest::Client::new())
    }

    pub async fn gh_get(&self, url: impl IntoUrl) -> anyhow::Result<reqwest::Response> {
        let response = self
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header(
                "User-Agent",
                "Mozilla/5.0 (platform; rv:geckoversion) Gecko/geckotrail Firefox/firefoxversion",
            )
            .send()
            .await?;
        if response.status() != StatusCode::OK {
            let status = response.status();
            let body = response
                .text()
                .await
                .with_context(|| "Response is not a valid UTF-8 text")?;
            anyhow::bail!("[{status}] {body}");
        }

        Ok(response)
    }

    pub async fn gh_json<T: DeserializeOwned>(&self, url: impl IntoUrl) -> anyhow::Result<T> {
        let response = self.gh_get(url).await?;

        response.json::<T>().await.with_context(|| {
            format!(
                "fail to deserialize response into type {}",
                std::any::type_name::<T>()
            )
        })
    }

    pub async fn download_github_file(
        &self,
        url: impl IntoUrl,
        filename: impl Display + AsRef<std::path::Path>,
    ) -> anyhow::Result<()> {
        let content = self.gh_get(url).await?.text().await?;
        tokio::fs::write(&filename, content)
            .await
            .with_context(|| format!("fail to save github content into file {filename}"))?;

        Ok(())
    }
}
