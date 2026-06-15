use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, REFERER};
use crate::models::{Equipment, Spell};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct Scraper {
    client: reqwest::Client,
}

impl Scraper {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64; rv:151.0) Gecko/20100101 Firefox/151.0"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
        headers.insert(REFERER, HeaderValue::from_static("https://www.zenithwakfu.com/"));
        
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
            
        Self { client }
    }

    pub async fn fetch_all_equipment(&self) -> Result<Vec<Equipment>> {
        let mut all_equipment = Vec::new();
        let mut to_skip = 0;
        let limit = 75;

        loop {
            println!("Fetching equipment: toSkip={}...", to_skip);
            let url = format!(
                "https://api.zenithwakfu.com/builder/api/equipment?minLvl=1&maxLvl=245&rarity[]=7&rarity[]=6&rarity[]=5&rarity[]=4&rarity[]=3&rarity[]=2&toSkip={}",
                to_skip
            );

            let resp = self.client.get(&url)
                .header("X-Requested-With", "XMLHttpRequest")
                .send()
                .await?
                .json::<Vec<Equipment>>()
                .await?;

            if resp.is_empty() {
                break;
            }

            let count = resp.len();
            all_equipment.extend(resp);
            
            if count < limit {
                break;
            }

            to_skip += limit;
            // Respectful delay
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        Ok(all_equipment)
    }

    pub async fn fetch_huppermage_spells(&self) -> Result<serde_json::Value> {
        let url = "https://api.zenithwakfu.com/builder/api/spells/19";
        let resp = self.client.get(url)
            .header("X-Requested-With", "XMLHttpRequest")
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        Ok(resp)
    }

    pub fn save_cache<T: serde::Serialize>(&self, data: &T, filename: &str) -> Result<()> {
        let path = Path::new("data").join(filename);
        let mut file = File::create(path)?;
        let json = serde_json::to_string_pretty(data)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}
