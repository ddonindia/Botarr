use super::super::search::{build_http_client, parse_size, XdccSearchProvider};
use crate::xdcc::{XdccError, XdccSearchResult, XdccUrl};
use async_trait::async_trait;
use serde::Deserialize;

/// XDCC.rocks search provider
pub struct XdccRocksProvider {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct XdccRocksResponse {
    #[serde(default)]
    results: Vec<XdccRocksServer>,
    #[serde(default)]
    page: u32,
    #[serde(default)]
    maxpages: u32,
}

#[derive(Debug, Deserialize)]
struct XdccRocksServer {
    #[serde(default)]
    servername: String,
    #[serde(default)]
    serverhost: String,
    #[serde(default)]
    channels: Vec<XdccRocksChannel>,
}

#[derive(Debug, Deserialize)]
struct XdccRocksChannel {
    #[serde(default)]
    channelname: String,
    #[serde(default)]
    bots: Vec<XdccRocksBot>,
}

#[derive(Debug, Deserialize)]
struct XdccRocksBot {
    #[serde(default)]
    botname: String,
    #[serde(default)]
    files: Vec<XdccRocksFile>,
}

#[derive(Debug, Deserialize)]
struct XdccRocksFile {
    #[serde(default)]
    packnumber: i32,
    #[serde(default)]
    numdownloads: u32,
    #[serde(default)]
    file: XdccRocksFileInfo,
}

#[derive(Debug, Deserialize, Default)]
struct XdccRocksFileInfo {
    #[serde(default)]
    filename: String,
    #[serde(default)]
    filesize: String,
}

impl XdccRocksProvider {
    pub fn new(proxy_url: Option<&str>) -> Self {
        Self {
            client: build_http_client(proxy_url),
        }
    }

    async fn fetch_page(&self, query: &str, page: u32) -> Result<XdccRocksResponse, XdccError> {
        let url = format!(
            "https://xdcc.rocks/search/?searchword={}&getpages=true&page={}",
            urlencoding::encode(query),
            page
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| XdccError::SearchFailed(format!("HTTP error: {}", e)))?;

        response
            .json()
            .await
            .map_err(|e| XdccError::SearchFailed(format!("JSON parse error: {}", e)))
    }
}

#[async_trait]
impl XdccSearchProvider for XdccRocksProvider {
    fn name(&self) -> &str {
        "XDCC.rocks"
    }

    async fn search(&self, query: &str) -> Result<Vec<XdccSearchResult>, XdccError> {
        let first = self.fetch_page(query, 1).await?;

        tracing::info!(
            "XDCC.rocks: page {}/{} with {} servers",
            first.page,
            first.maxpages,
            first.results.len()
        );

        let mut all_data = first.results;
        let max_pages = first.maxpages.min(5);
        for page in 2..=max_pages {
            if let Ok(resp) = self.fetch_page(query, page).await {
                all_data.extend(resp.results);
            }
        }

        let mut results = Vec::new();
        for server in all_data {
            let network = if server.serverhost.is_empty() {
                server.servername.clone()
            } else {
                server.serverhost.clone()
            };

            for channel in server.channels {
                let channel_name = if channel.channelname.starts_with('#') {
                    channel.channelname
                } else {
                    format!("#{}", channel.channelname)
                };

                for bot in channel.bots {
                    for file in bot.files {
                        if file.file.filename.is_empty() {
                            continue;
                        }
                        results.push(XdccSearchResult {
                            url: XdccUrl {
                                network: network.clone(),
                                channel: channel_name.clone(),
                                bot: bot.botname.clone(),
                                slot: file.packnumber,
                            },
                            filename: file.file.filename,
                            size: parse_size(&file.file.filesize),
                            size_str: file.file.filesize,
                            bot: bot.botname.clone(),
                            network: network.clone(),
                            channel: channel_name.clone(),
                            slot: file.packnumber,
                            gets: Some(file.numdownloads),
                        });
                    }
                }
            }
        }

        Ok(results)
    }
}
