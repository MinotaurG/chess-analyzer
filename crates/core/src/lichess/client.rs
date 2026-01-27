//! Lichess API client for fetching games and evaluations

use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use std::time::Duration;

use super::types::*;
use crate::error::Result;

const LICHESS_API_BASE: &str = "https://lichess.org/api";

pub struct LichessClient {
    client: Client,
    token: Option<String>,
}

impl LichessClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            token: None,
        })
    }

    pub fn with_token(token: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            token: Some(token),
        })
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/x-ndjson"));
        
        if let Some(ref token) = self.token {
            if let Ok(value) = HeaderValue::from_str(&format!("Bearer {}", token)) {
                headers.insert(AUTHORIZATION, value);
            }
        }
        
        headers
    }

    /// Fetch games for a user
    pub async fn get_user_games(&self, username: &str, params: &GameExportParams) -> Result<Vec<LichessGame>> {
        let url = format!("{}/games/user/{}", LICHESS_API_BASE, username);
        
        let mut request = self.client
            .get(&url)
            .headers(self.headers())
            .query(&[
                ("pgnInJson", "true"),
                ("opening", "true"),
                ("moves", "true"),
            ]);

        if let Some(max) = params.max {
            request = request.query(&[("max", max.to_string())]);
        }
        if let Some(ref perf_type) = params.perf_type {
            request = request.query(&[("perfType", perf_type.as_str())]);
        }
        if params.rated_only {
            request = request.query(&[("rated", "true")]);
        }
        if params.with_analysis {
            request = request.query(&[("analysed", "true")]);
        }
        if let Some(since) = params.since {
            request = request.query(&[("since", since.to_string())]);
        }

        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(crate::error::Error::Lichess(format!(
                "API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let text = response.text().await?;
        let games = parse_ndjson_games(&text)?;
        
        Ok(games)
    }

    /// Get cloud evaluation for a FEN position
    pub async fn cloud_eval(&self, fen: &str, multi_pv: u8) -> Result<CloudEval> {
        let url = format!("{}/cloud-eval", LICHESS_API_BASE);
        
        let response = self.client
            .get(&url)
            .query(&[("fen", fen), ("multiPv", &multi_pv.to_string())])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(crate::error::Error::Lichess(format!(
                "Cloud eval error: {}",
                response.status()
            )));
        }

        let eval: CloudEval = response.json().await?;
        Ok(eval)
    }

    /// Get user profile
    pub async fn get_user(&self, username: &str) -> Result<LichessUser> {
        let url = format!("{}/user/{}", LICHESS_API_BASE, username);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(crate::error::Error::Lichess(format!(
                "User not found: {}",
                username
            )));
        }

        let user: LichessUser = response.json().await?;
        Ok(user)
    }
}

impl Default for LichessClient {
    fn default() -> Self {
        Self::new().expect("Failed to create HTTP client")
    }
}

fn parse_ndjson_games(text: &str) -> Result<Vec<LichessGame>> {
    let mut games = Vec::new();
    
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        match serde_json::from_str::<LichessGame>(line) {
            Ok(game) => games.push(game),
            Err(e) => {
                eprintln!("Warning: Failed to parse game: {}", e);
                continue;
            }
        }
    }
    
    Ok(games)
}
