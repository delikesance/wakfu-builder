use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, REFERER};
use crate::models::{Equipment, Spell};
use std::path::PathBuf;

pub struct Scraper {
    client: reqwest::Client,
}

pub fn ensure_data_dir() -> Result<PathBuf> {
    let dir = std::env::current_dir()?.join("data");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

impl Scraper {
    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64; rv:151.0) Gecko/20100101 Firefox/151.0"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
        headers.insert(REFERER, HeaderValue::from_static("https://www.zenithwakfu.com/"));
        
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
            
        Ok(Self { client })
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

            let mut last_err = None;
            for attempt in 1..=3 {
                match self.client.get(&url)
                    .header("X-Requested-With", "XMLHttpRequest")
                    .send()
                    .await
                {
                    Ok(resp) => {
                        let equipment: Vec<Equipment> = resp.json().await?;
                        if equipment.is_empty() {
                            return Ok(all_equipment);
                        }
                        let count = equipment.len();
                        all_equipment.extend(equipment);
                        if count < limit {
                            return Ok(all_equipment);
                        }
                        to_skip += limit;
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        break;
                    }
                    Err(e) => {
                        println!("Attempt {} failed: {}. Retrying in 2s...", attempt, e);
                        last_err = Some(e);
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            }
            if let Some(e) = last_err {
                return Err(e.into());
            }
        }
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
        let dir = ensure_data_dir()?;
        let path = dir.join(filename);
        let tmp_path = dir.join(format!(".{}.tmp", filename));
        let json = serde_json::to_string_pretty(data)?;
        std::fs::write(&tmp_path, json)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }
}
