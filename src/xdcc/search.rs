//! XDCC Search Providers
//!
//! Implements search functionality for various XDCC search engines.
use super::providers::*;

use super::{XdccError, XdccSearchResult};
use async_trait::async_trait;

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
        agg.add_provider(Box::new(NiblProvider::new(proxy_url)));
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

        // Filter scenep2p bots with |P|
        all_results.retain(|r| {
            !(r.network.to_lowercase().contains("scenep2p") && r.bot.to_lowercase().contains("|p|"))
        });

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

pub fn build_http_client(proxy_url: Option<&str>) -> reqwest::Client {
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

pub fn parse_size(size_str: &str) -> Option<u64> {
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
