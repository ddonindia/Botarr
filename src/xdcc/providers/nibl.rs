use super::super::search::{build_http_client, parse_size, XdccSearchProvider};
use crate::xdcc::{XdccError, XdccSearchResult, XdccUrl};
use async_trait::async_trait;
use serde::Deserialize;

/// NIBL search provider (nibl.co.uk) - Anime-focused XDCC search
/// All bots are on irc.rizon.net / #nibl
pub struct NiblProvider {
    client: reqwest::Client,
    /// Cached bot list: maps bot ID → bot name
    bot_cache: tokio::sync::RwLock<Option<NiblBotCache>>,
}

struct NiblBotCache {
    bots: std::collections::HashMap<i64, String>,
    fetched_at: std::time::Instant,
}

impl NiblBotCache {
    fn is_fresh(&self) -> bool {
        self.fetched_at.elapsed() < std::time::Duration::from_secs(3600) // 1 hour TTL
    }
}

#[derive(Debug, Deserialize)]
struct NiblApiResponse<T> {
    #[serde(default)]
    status: String,
    #[serde(default)]
    content: Vec<T>,
}

#[derive(Debug, Default, Deserialize)]
struct NiblBot {
    id: i64,
    name: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NiblPack {
    #[serde(default)]
    bot_id: i64,
    #[serde(default)]
    number: i32,
    #[serde(default)]
    name: String,
    #[serde(default)]
    size: String,
    #[serde(default)]
    sizekbits: u64,
}

const NIBL_NETWORK: &str = "irc.rizon.net";
const NIBL_CHANNEL: &str = "#nibl";

impl NiblProvider {
    pub fn new(proxy_url: Option<&str>) -> Self {
        Self {
            client: build_http_client(proxy_url),
            bot_cache: tokio::sync::RwLock::new(None),
        }
    }

    /// Fetch and cache the bot list from NIBL
    async fn ensure_bots(&self) -> Result<(), XdccError> {
        // Check if cache is still fresh
        {
            let cache = self.bot_cache.read().await;
            if let Some(ref c) = *cache {
                if c.is_fresh() {
                    return Ok(());
                }
            }
        }

        // Fetch fresh bot list
        let response = self
            .client
            .get("https://api.nibl.co.uk/nibl/bots")
            .send()
            .await
            .map_err(|e| XdccError::SearchFailed(format!("NIBL bots HTTP error: {}", e)))?;

        let api_resp: NiblApiResponse<NiblBot> = response
            .json()
            .await
            .map_err(|e| XdccError::SearchFailed(format!("NIBL bots JSON error: {}", e)))?;

        let bots: std::collections::HashMap<i64, String> = api_resp
            .content
            .into_iter()
            .map(|b| (b.id, b.name))
            .collect();

        tracing::info!("NIBL: cached {} bots", bots.len());

        let mut cache = self.bot_cache.write().await;
        *cache = Some(NiblBotCache {
            bots,
            fetched_at: std::time::Instant::now(),
        });

        Ok(())
    }

    /// Resolve a bot ID to its name
    async fn bot_name(&self, bot_id: i64) -> String {
        let cache = self.bot_cache.read().await;
        cache
            .as_ref()
            .and_then(|c| c.bots.get(&bot_id).cloned())
            .unwrap_or_else(|| format!("Bot#{}", bot_id))
    }

    async fn fetch_page(
        &self,
        query: &str,
        page: u32,
        size: u32,
    ) -> Result<NiblApiResponse<NiblPack>, XdccError> {
        let url = format!(
            "https://api.nibl.co.uk/nibl/search?query={}&botId=-1&page={}&size={}",
            urlencoding::encode(query),
            page,
            size
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| XdccError::SearchFailed(format!("NIBL search HTTP error: {}", e)))?;

        response
            .json()
            .await
            .map_err(|e| XdccError::SearchFailed(format!("NIBL search JSON error: {}", e)))
    }
}

#[async_trait]
impl XdccSearchProvider for NiblProvider {
    fn name(&self) -> &str {
        "NIBL"
    }

    async fn search(&self, query: &str) -> Result<Vec<XdccSearchResult>, XdccError> {
        // Ensure bot cache is populated
        self.ensure_bots().await?;

        // Fetch up to 250 results (max 5 pages of 50)
        let first = self.fetch_page(query, 0, 50).await?;
        if first.status != "OK" {
            return Ok(Vec::new());
        }

        tracing::info!("NIBL: first page returned {} results", first.content.len());

        let mut all_packs = first.content;

        // Fetch additional pages if first page was full
        if all_packs.len() >= 50 {
            for page in 1..5u32 {
                match self.fetch_page(query, page, 50).await {
                    Ok(resp) => {
                        if resp.content.is_empty() {
                            break;
                        }
                        let count = resp.content.len();
                        all_packs.extend(resp.content);
                        if count < 50 {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("NIBL page {} failed: {}", page, e);
                        break;
                    }
                }
            }
        }

        let mut results = Vec::with_capacity(all_packs.len());
        for pack in all_packs {
            if pack.name.is_empty() || pack.number == 0 {
                continue;
            }

            let bot_name = self.bot_name(pack.bot_id).await;

            let size_bytes = if pack.sizekbits > 0 {
                Some(pack.sizekbits)
            } else {
                parse_size(&pack.size)
            };

            results.push(XdccSearchResult {
                url: XdccUrl {
                    network: NIBL_NETWORK.to_string(),
                    channel: NIBL_CHANNEL.to_string(),
                    bot: bot_name.clone(),
                    slot: pack.number,
                },
                filename: pack.name,
                size: size_bytes,
                size_str: pack.size,
                bot: bot_name,
                network: NIBL_NETWORK.to_string(),
                channel: NIBL_CHANNEL.to_string(),
                slot: pack.number,
                gets: None,
            });
        }

        Ok(results)
    }
}
