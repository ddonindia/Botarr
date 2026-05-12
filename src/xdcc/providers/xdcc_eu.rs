use super::super::search::{build_http_client, parse_size, XdccSearchProvider};
use crate::xdcc::{XdccError, XdccSearchResult, XdccUrl};
use async_trait::async_trait;

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
