use reqwest::Client;
use serde::Deserialize;

// Structure for an Anime item from Jikan API response
#[derive(Debug, Deserialize)]
pub struct Anime {
    pub mal_id: u32, // Jikan uses mal_id for the anime ID
    pub title: String,
}

// Top-level search response structure for Jikan API
#[derive(Debug, Deserialize)]
pub struct JikanSearchResponse {
    pub data: Vec<Anime>,
}

/// Searches for anime on the Jikan API.
/// Returns search results using the provided query.
/// This function is asynchronous.
pub async fn search_anime(query: &str) -> Option<Vec<Anime>> {
    let client = Client::new();
    let search_url = "https://api.jikan.moe/v4/anime";

    let res = match client
        .get(search_url)
        .query(&[("q", query), ("limit", "5")])
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Error sending anime search request: {}", e);
            return None;
        }
    };

    if res.status().is_success() {
        match res.json::<JikanSearchResponse>().await {
            Ok(search_results) => Some(search_results.data),
            Err(e) => {
                eprintln!("Error converting anime search response to JSON: {}", e);
                None
            }
        }
    } else {
        eprintln!("Error code when searching for anime: {}", res.status());
        eprintln!("Response body: {:?}", res.text().await);
        None
    }
}
