//! XDCC Search Providers
//!
//! Implements search functionality for various XDCC search engines.

use super::{XdccError, XdccSearchResult, XdccUrl};
use async_trait::async_trait;
use serde::Deserialize;

/// Trait for XDCC search providers
#[async_trait]
pub trait XdccSearchProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Search for files matching the query
    async fn search(&self, query: &str) -> Result<Vec<XdccSearchResult>, XdccError>;
}

/// Aggregates multiple search providers
pub struct SearchAggregator {
    providers: Vec<Box<dyn XdccSearchProvider>>,
}

impl SearchAggregator {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn with_default_providers(proxy_url: Option<&str>) -> Self {
        let mut agg = Self::new();
        // Add all providers - search runs in parallel
        agg.add_provider(Box::new(SkullXdccProvider::new(proxy_url)));
        agg.add_provider(Box::new(XdccRocksProvider::new(proxy_url)));
        agg.add_provider(Box::new(XdccEuProvider::new(proxy_url)));
        agg
    }

    pub fn add_provider(&mut self, provider: Box<dyn XdccSearchProvider>) {
        self.providers.push(provider);
    }

    /// Search providers and aggregate results
    /// If `target_providers` is specific, only those providers are queried.
    pub async fn search(
        &self,
        query: &str,
        target_providers: Option<&[String]>,
    ) -> Result<Vec<XdccSearchResult>, XdccError> {
        use futures::future::join_all;

        // Run searches in parallel (filtered)
        let futures: Vec<_> = self
            .providers
            .iter()
            .filter(|p| match target_providers {
                Some(targets) => targets.iter().any(|t| t.eq_ignore_ascii_case(p.name())),
                None => true,
            })
            .map(|p| p.search(query))
            .collect();
        let results = join_all(futures).await;

        let mut all_results = Vec::new();
        for result in results {
            match result {
                Ok(r) => {
                    tracing::info!(
                        "Provider {} returned {} results",
                        // We need to re-match the result to the provider name, but for logging we can't easily get the index after filtering
                        "XDCC", // simplified log to avoid index complexity
                        r.len()
                    );
                    all_results.extend(r);
                }
                Err(e) => {
                    tracing::warn!("Search provider failed: {}", e);
                }
            }
        }

        // Deduplicate by URL
        let mut seen = std::collections::HashSet::new();
        all_results.retain(|r| seen.insert(r.url.clone()));

        Ok(all_results)
    }
}

impl Default for SearchAggregator {
    fn default() -> Self {
        Self::new()
    }
}

// ============= Helper Functions =============

fn build_http_client(proxy_url: Option<&str>) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");

    if let Some(proxy) = proxy_url {
        if let Ok(proxy) = reqwest::Proxy::all(proxy) {
            builder = builder.proxy(proxy);
        }
    }

    builder.build().unwrap_or_default()
}

fn parse_size(size_str: &str) -> Option<u64> {
    let size_str = size_str
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_uppercase();
    if size_str.is_empty() {
        return None;
    }

    let (num_part, multiplier) = if size_str.ends_with("GB") || size_str.ends_with('G') {
        (
            size_str.trim_end_matches("GB").trim_end_matches('G'),
            1024u64 * 1024 * 1024,
        )
    } else if size_str.ends_with("MB") || size_str.ends_with('M') {
        (
            size_str.trim_end_matches("MB").trim_end_matches('M'),
            1024u64 * 1024,
        )
    } else if size_str.ends_with("KB") || size_str.ends_with('K') {
        (
            size_str.trim_end_matches("KB").trim_end_matches('K'),
            1024u64,
        )
    } else {
        (size_str.as_str(), 1u64)
    };

    num_part
        .trim()
        .parse::<f64>()
        .ok()
        .map(|n| (n * multiplier as f64) as u64)
}

// ============= SkullXDCC Provider =============

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

// ============= XDCC.rocks Provider =============

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

// ============= XDCC.eu Provider =============

/// XDCC.eu search provider (HTML scraping)
pub struct XdccEuProvider {
    client: reqwest::Client,
}

impl XdccEuProvider {
    pub fn new(proxy_url: Option<&str>) -> Self {
        Self {
            client: build_http_client(proxy_url),
        }
    }
}

#[async_trait]
impl XdccSearchProvider for XdccEuProvider {
    fn name(&self) -> &str {
        "XDCC.eu"
    }

    async fn search(&self, query: &str) -> Result<Vec<XdccSearchResult>, XdccError> {
        use scraper::{Html, Selector};

        let url = format!(
            "https://www.xdcc.eu/search.php?searchkey={}",
            urlencoding::encode(query)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| XdccError::SearchFailed(format!("HTTP error: {}", e)))?;

        let html = response
            .text()
            .await
            .map_err(|e| XdccError::SearchFailed(format!("Read error: {}", e)))?;

        let document = Html::parse_document(&html);
        let row_selector = Selector::parse("tr").unwrap();
        let cell_selector = Selector::parse("td").unwrap();
        let link_selector = Selector::parse("a[href^='irc://']").unwrap();

        let mut results = Vec::new();

        for row in document.select(&row_selector) {
            let cells: Vec<_> = row.select(&cell_selector).collect();
            if cells.len() < 7 {
                continue;
            }

            // Extract network from irc:// link
            let network = cells[0]
                .select(&link_selector)
                .next()
                .and_then(|a| a.value().attr("href"))
                .and_then(|href| {
                    // Parse irc://server/channel
                    href.strip_prefix("irc://")
                        .and_then(|s| s.split('/').next())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| cells[0].text().collect::<String>().trim().to_string());

            let channel_text: String = cells[1].text().collect::<String>().trim().to_string();
            let channel = if channel_text.starts_with('#') {
                channel_text
            } else {
                format!("#{}", channel_text)
            };

            let bot: String = cells[2].text().collect::<String>().trim().to_string();
            let slot_str: String = cells[3].text().collect::<String>().trim().to_string();
            let gets_str: String = cells[4].text().collect::<String>().trim().to_string();
            let size_str: String = cells[5].text().collect::<String>().trim().to_string();
            let filename: String = cells[6].text().collect::<String>().trim().to_string();

            let slot = slot_str.trim_start_matches('#').parse::<i32>().unwrap_or(0);
            if slot == 0 || network.is_empty() || bot.is_empty() || filename.is_empty() {
                continue;
            }

            let gets = gets_str.parse::<u32>().ok();

            results.push(XdccSearchResult {
                url: XdccUrl {
                    network: network.clone(),
                    channel: channel.clone(),
                    bot: bot.clone(),
                    slot,
                },
                filename,
                size: parse_size(&size_str),
                size_str,
                bot,
                network,
                channel,
                slot,
                gets,
            });
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1.5G"), Some(1610612736));
        assert_eq!(parse_size("[500M]"), Some(524288000));
        assert_eq!(parse_size("100KB"), Some(102400));
        assert_eq!(parse_size("1.2GB"), Some(1288490188));
        assert_eq!(parse_size(""), None);
    }
}
