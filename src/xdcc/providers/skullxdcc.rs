use super::super::search::{build_http_client, parse_size, XdccSearchProvider};
use crate::xdcc::{XdccError, XdccSearchResult, XdccUrl};
use async_trait::async_trait;
use serde::Deserialize;

/// SkullXDCC search provider (skullxdcc.com)
pub struct SkullXdccProvider {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct SkullXdccResponse {
    #[serde(default)]
    error: bool,
    #[serde(default)]
    data: Vec<SkullXdccResult>,
    #[serde(default)]
    total_results: u32,
    #[serde(default)]
    pages: u32,
}

#[derive(Debug, Deserialize)]
struct SkullXdccResult {
    #[serde(default)]
    network: String,
    #[serde(default)]
    channel: String,
    #[serde(default)]
    bot: String,
    #[serde(default)]
    packnum: i32,
    #[serde(default)]
    fname: String,
    #[serde(default)]
    fsize: String,
    #[serde(default)]
    gets: u32,
}

impl SkullXdccProvider {
    pub fn new(proxy_url: Option<&str>) -> Self {
        Self {
            client: build_http_client(proxy_url),
        }
    }

    async fn fetch_page(&self, query: &str, page: u32) -> Result<SkullXdccResponse, XdccError> {
        let url = format!(
            "https://skullxdcc.com/ws.php?sterm={}&limit_results=250&page={}",
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
impl XdccSearchProvider for SkullXdccProvider {
    fn name(&self) -> &str {
        "SkullXDCC"
    }

    async fn search(&self, query: &str) -> Result<Vec<XdccSearchResult>, XdccError> {
        let first = self.fetch_page(query, 0).await?;
        if first.error {
            return Ok(Vec::new());
        }

        tracing::info!(
            "SkullXDCC: {} total results across {} pages",
            first.total_results,
            first.pages
        );

        let mut all_data = first.data;
        let max_pages = first.pages.min(5);
        for page in 1..max_pages {
            if let Ok(resp) = self.fetch_page(query, page).await {
                all_data.extend(resp.data);
            }
        }

        Ok(all_data
            .into_iter()
            .filter(|r| !r.network.is_empty() && !r.bot.is_empty() && !r.fname.is_empty())
            .map(|r| {
                let channel = if r.channel.starts_with('#') {
                    r.channel
                } else {
                    format!("#{}", r.channel)
                };
                XdccSearchResult {
                    url: XdccUrl {
                        network: r.network.clone(),
                        channel: channel.clone(),
                        bot: r.bot.clone(),
                        slot: r.packnum,
                    },
                    filename: r.fname,
                    size: parse_size(&r.fsize),
                    size_str: r.fsize,
                    bot: r.bot,
                    network: r.network,
                    channel,
                    slot: r.packnum,
                    gets: Some(r.gets),
                }
            })
            .collect())
    }
}
