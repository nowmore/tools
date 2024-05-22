#![allow(non_snake_case)]
const LATEST_URL: &'static str = "https://bangumi.moe/api/torrent/latest";
const TORRENT_URL: &'static str = "https://bangumi.moe/api/torrent/page";
const HTTP_PROXY: &'static str = "http://127.0.0.1:7890";
const UA: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36";

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use reqwest::{header::*, Client};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Torrent {
    _id: String,
    category_tag_id: String,
    pub title: String,
    introduction: String,
    tag_ids: Vec<String>,
    comments: u16,
    downloads: u16,
    finished: u16,
    leechers: u16,
    seeders: u16,
    uploader_id: String,
    #[serde(default)]
    team_id: Option<String>,
    pub publish_time: String,
    pub magnet: String,
    infoHash: String,
    file_id: String,
    #[serde(default)]
    teamsync: Option<bool>,
    content: Vec<Vec<String>>,
    size: String,
    btskey: String,
    #[serde(default)]
    sync: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
struct LatestRsp {
    page_count: u32,
    torrents: Vec<Torrent>,
}

pub async fn get_torrents(
    earliest: &NaiveDateTime,
) -> Result<Vec<Torrent>, Box<dyn std::error::Error>> {
    let c = Client::builder()
        .proxy(reqwest::Proxy::all(HTTP_PROXY)?)
        .build()?;
    let mut torrents: Vec<Torrent> = Vec::new();
    let mut n = 1;
    while n < 100 {
        let url = if n == 1 {
            LATEST_URL.to_string()
        } else {
            format!("{}/{}", TORRENT_URL, n)
        };
        let mut rsp: LatestRsp = c
            .get(url.clone())
            .header(USER_AGENT, UA)
            .send()
            .await?
            .json()
            .await?;

        let earliest_released = rsp
            .torrents
            .iter_mut()
            .min_by(|t1, t2| t1.publish_time.cmp(&t2.publish_time));

        let earliest_publish = Local::from_utc_datetime(
            &Local,
            &DateTime::parse_from_rfc3339(&earliest_released.unwrap().publish_time)
                .unwrap()
                .naive_utc(),
        );

        torrents.extend(rsp.torrents);
        if earliest_publish.naive_local() <= *earliest {
            break;
        }
        n += 1;
        tokio::time::sleep(std::time::Duration::from_micros(500)).await;
    }
    Ok(torrents)
}
