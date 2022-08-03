use std::collections::HashMap;

use hyper::{body::HttpBody, client::HttpConnector, Body, Client as HyperClient, Method, Request};
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use serde_json::Value;

const BASE_URL: &str = "https://wakatime.com/api/v1";
const ALL_TIME_SINCE_TODAY: &str = "users/current/all_time_since_today";
const WEEKDAYS: &str = "/api/v1/users/current/stats/last_7_days";

type Client = HyperClient<HttpsConnector<HttpConnector>, Body>;

pub fn new_client() -> Client {
    HyperClient::builder().build(HttpsConnector::new())
}

async fn get<T>(url: &str, cli: &Client, api_key: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("{}/{}?api_key={}", BASE_URL, url, api_key))
        .body(Body::empty())
        .map_err(|e| e.to_string())?;
    let mut resp = cli.request(req).await.map_err(|e| e.to_string())?;
    let data = resp
        .data()
        .await
        .ok_or("empty body".to_string())?
        .map_err(|e| e.to_string())?;
    let s = String::from_utf8(data.to_vec()).unwrap();
    println!("{s}");
    serde_json::from_str(&s).map_err(|e| e.to_string())
}

async fn gets<T>(url: &str, users: &HashMap<String, String>) -> HashMap<String, Result<T, String>>
where
    T: for<'de> Deserialize<'de>,
{
    let cli = new_client();
    let mut map = HashMap::new();
    for (name, api_key) in users.iter() {
        map.insert(name.to_string(), get(url, &cli, api_key).await);
    }
    map
}

#[derive(Debug, Deserialize)]
pub struct SinceToday {
    pub data: SinceTodayData,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct SinceTodayData {
    pub decimal: String, // xx.xx
    pub digital: String, // xx:xx
    pub is_up_to_date: bool,
    pub percent_calculated: f32, // update_percent 100.0 for all updated
    pub range: HashMap<String, Value>, //todo
    pub timeout: i32,
    pub total_seconds: i32,
}

pub async fn get_today(
    users: &HashMap<String, String>,
) -> HashMap<String, Result<SinceToday, String>> {
    gets(ALL_TIME_SINCE_TODAY, users).await
}

#[derive(Debug, Deserialize)]
pub struct Weekdays {
    pub data: HashMap<String, Value>, //todo
    pub range: String,
    pub human_readable_range: String,
    pub status: String,
    pub is_including_today: bool,
    pub is_up_to_date: bool,
    pub percent_calculated: i32,
    pub start: String,
    pub end: String,
    pub timezone: String,
    pub timeout: i32,
    pub writes_only: bool,
    pub user_id: String,
    pub created_at: String,
    pub modified_at: String,
    // not in doc, api_key only?
    pub username: String,
    pub daily_average: f32,
    pub total_seconds: f32,
}

pub async fn get_weekdays(
    users: &HashMap<String, String>,
) -> HashMap<String, Result<Weekdays, String>> {
    gets(WEEKDAYS, users).await
}
