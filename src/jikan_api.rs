use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Anime {
    pub mal_id: u32,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct JikanSearchResponse {
    pub data: Vec<Anime>,
}

#[derive(Debug, Deserialize)]
pub struct Aired {
    pub from: Option<String>,
    pub to: Option<String>,
    pub string: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImageUrls {
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CommonImageResource {
    pub jpg: Option<ImageUrls>,
}

#[derive(Debug, Deserialize)]
pub struct YoutubeMeta {
    pub youtube_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Title {
    #[serde(rename = "type")]
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct AnimeDetails {
    pub mal_id: u32,
    pub url: String,
    pub images: Option<CommonImageResource>,
    pub trailer: Option<YoutubeMeta>,
    pub approved: Option<bool>,
    pub titles: Vec<Title>,
    pub title: String,
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub title_synonyms: Vec<String>,
    #[serde(rename = "type")]
    pub anime_type: Option<String>,
    pub source: Option<String>,
    pub episodes: Option<u32>,
    pub status: String,
    pub airing: Option<bool>,
    pub aired: Aired,
    pub duration: Option<String>,
    pub rating: Option<String>,
    pub score: Option<f32>,
    pub scored_by: Option<u32>,
    pub rank: Option<u32>,
    pub popularity: Option<u32>,
    pub members: Option<u32>,
    pub favorites: Option<u32>,
    pub synopsis: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JikanDetailsResponse {
    pub data: AnimeDetails,
}

// New structs for recommendations API
#[derive(Debug, Deserialize)]
pub struct RecommendationEntryDetails {
    pub mal_id: u32,
    pub url: String,
    pub images: CommonImageResource,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct RecommendationItem {
    pub entry: RecommendationEntryDetails,
}

#[derive(Debug, Deserialize)]
pub struct JikanRecommendationsResponse {
    pub data: Vec<RecommendationItem>,
}

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

pub async fn get_anime_details_by_id(mal_id: u32) -> Option<AnimeDetails> {
    let client = Client::new();
    let details_url = format!("https://api.jikan.moe/v4/anime/{}", mal_id);

    let res = match client.get(&details_url).send().await {
        Ok(response) => response,
        Err(e) => {
            eprintln!(
                "Error sending anime details request for ID {}: {}",
                mal_id, e
            );
            return None;
        }
    };

    if res.status().is_success() {
        let body = match res.text().await {
            Ok(text) => text,
            Err(e) => {
                eprintln!("Error reading response body for ID {}: {}", mal_id, e);
                return None;
            }
        };

        match serde_json::from_str::<JikanDetailsResponse>(&body) {
            Ok(details_response) => Some(details_response.data),
            Err(e) => {
                eprintln!(
                    "Error converting anime details response to JSON for ID {}: {}",
                    mal_id, e
                );
                eprintln!("Problematic response body: {}", body);
                None
            }
        }
    } else {
        eprintln!(
            "Error code when getting anime details for ID {}: {}",
            mal_id,
            res.status()
        );
        eprintln!("Response body: {:?}", res.text().await);
        None
    }
}

// New function to get anime recommendations
pub async fn get_anime_recommendations(mal_id: u32) -> Option<Vec<RecommendationItem>> {
    let client = Client::new();
    let recommendations_url = format!("https://api.jikan.moe/v4/anime/{}/recommendations", mal_id);

    let res = match client.get(&recommendations_url).send().await {
        Ok(response) => response,
        Err(e) => {
            eprintln!(
                "Error sending recommendations request for ID {}: {}",
                mal_id, e
            );
            return None;
        }
    };

    if res.status().is_success() {
        let body = match res.text().await {
            Ok(text) => text,
            Err(e) => {
                eprintln!(
                    "Error reading recommendations response body for ID {}: {}",
                    mal_id, e
                );
                return None;
            }
        };

        match serde_json::from_str::<JikanRecommendationsResponse>(&body) {
            Ok(recommendations_response) => Some(recommendations_response.data),
            Err(e) => {
                eprintln!(
                    "Error converting recommendations response to JSON for ID {}: {}",
                    mal_id, e
                );
                eprintln!("Problematic recommendations response body: {}", body);
                None
            }
        }
    } else {
        eprintln!(
            "Error code when getting recommendations for ID {}: {}",
            mal_id,
            res.status()
        );
        eprintln!("Response body: {:?}", res.text().await);
        None
    }
}
