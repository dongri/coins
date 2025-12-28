use anyhow::Result;
use serde::Deserialize;

const COINGECKO_API_BASE: &str = "https://api.coingecko.com/api/v3";

#[derive(Debug, Clone, Deserialize)]
pub struct CoinMarket {
    pub symbol: String,
    pub name: String,
    pub current_price: Option<f64>,
    pub market_cap: Option<f64>,
    pub market_cap_rank: Option<u32>,
    pub total_volume: Option<f64>,
    pub high_24h: Option<f64>,
    pub low_24h: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub price_change_percentage_1h_in_currency: Option<f64>,
    pub price_change_percentage_24h_in_currency: Option<f64>,
    pub price_change_percentage_7d_in_currency: Option<f64>,
    pub price_change_percentage_14d_in_currency: Option<f64>,
    pub price_change_percentage_30d_in_currency: Option<f64>,
    pub price_change_percentage_60d_in_currency: Option<f64>,
    pub price_change_percentage_200d_in_currency: Option<f64>,
    pub price_change_percentage_1y_in_currency: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
    pub ath: Option<f64>,
    pub ath_date: Option<String>,
    pub atl: Option<f64>,
    pub atl_date: Option<String>,
    pub last_updated: Option<String>,
    #[serde(default)]
    pub sparkline_in_7d: Option<SparklineData>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SparklineData {
    pub price: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct CoinGeckoClient {
    client: reqwest::Client,
}

impl CoinGeckoClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("coins-cli/0.1.0")
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }

    /// Fetch top 100 coins by market cap with price change percentages
    pub async fn get_markets(&self, vs_currency: &str) -> Result<Vec<CoinMarket>> {
        let url = format!(
            "{}/coins/markets?vs_currency={}&order=market_cap_desc&per_page=100&page=1&sparkline=true&price_change_percentage=1h,24h,7d,14d,30d,60d,200d,1y",
            COINGECKO_API_BASE, vs_currency
        );

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("User-Agent", "coins-cli/0.1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("API request failed with status: {}", response.status());
        }

        let coins: Vec<CoinMarket> = response.json().await?;
        Ok(coins)
    }
}

impl Default for CoinGeckoClient {
    fn default() -> Self {
        Self::new()
    }
}
