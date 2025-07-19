use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::{Presence, Ready};
use serenity::prelude::*;
use serenity::builder::{CreateEmbed, CreateEmbedFooter, CreateMessage};
use serenity::model::Timestamp;

use crate::jikan_api;
use std::collections::HashMap;
use tokio::fs;
use serde::{Serialize, Deserialize};

pub struct Handler;

// Changed the file path to include the "data" directory
const QUERIES_FILE: &str = "data/user_queries.json";

#[derive(Debug, Serialize, Deserialize, Default)]
struct UserQueries {
    queries: HashMap<String, Vec<String>>,
}

async fn load_queries() -> UserQueries {
    match fs::read_to_string(QUERIES_FILE).await {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("Queries file not found, creating a new one.");
            // Ensure the 'data' directory exists before attempting to create the file
            if let Err(dir_err) = fs::create_dir_all("data").await {
                eprintln!("Error creating 'data' directory: {}", dir_err);
            }
            UserQueries::default()
        },
        Err(e) => {
            eprintln!("Error loading queries file: {}", e);
            UserQueries::default()
        }
    }
}

async fn save_queries(user_queries: &UserQueries) {
    let json_data = serde_json::to_string_pretty(user_queries).expect("Failed to serialize queries");
    // Ensure the 'data' directory exists before attempting to save the file
    if let Err(dir_err) = fs::create_dir_all("data").await {
        eprintln!("Error creating 'data' directory before saving: {}", dir_err);
        return;
    }
    if let Err(e) = fs::write(QUERIES_FILE, json_data).await {
        eprintln!("Error saving queries to file: {}", e);
    }
}

async fn send_embed_message(ctx: &Context, msg: &Message, embed: CreateEmbed) {
    let builder = CreateMessage::new().embed(embed);
    if let Err(why) = msg.channel_id.send_message(&ctx.http, builder).await {
        println!("Error sending message: {:?}", why);
    }
}

async fn handle_anime_details_command(ctx: &Context, msg: &Message, mal_id: u32) {
    if let Err(why) = msg.channel_id.say(&ctx.http, format!("Fetching details for MAL ID: {}...", mal_id)).await {
        println!("Error sending message: {:?}", why);
    }

    match jikan_api::get_anime_details_by_id(mal_id).await {
        Some(details) => {
            let mut embed = CreateEmbed::new()
                .title(format!("{} (MAL ID: {})", details.title, details.mal_id))
                .url(&details.url)
                .timestamp(Timestamp::now());

            if let Some(synopsis) = &details.synopsis {
                let short_synopsis = if synopsis.len() > 1000 {
                    format!("{}...", &synopsis[..1000])
                } else {
                    synopsis.clone()
                };
                embed = embed.description(short_synopsis);
            }

            if let Some(images_res) = &details.images {
                if let Some(jpg_images) = &images_res.jpg {
                    if let Some(image_url) = &jpg_images.image_url {
                        embed = embed.image(image_url);
                    }
                }
            }

            if let Some(title_english) = &details.title_english {
                embed = embed.field("English Title", title_english, true);
            }
            if let Some(title_japanese) = &details.title_japanese {
                embed = embed.field("Japanese Title", title_japanese, true);
            }
            if !details.title_synonyms.is_empty() {
                embed = embed.field("Synonyms", details.title_synonyms.join(", "), false);
            }

            embed = embed.field("Approved", details.approved.map_or("N/A".to_string(), |b| b.to_string()), true);
            embed = embed.field("Type", details.anime_type.as_deref().unwrap_or("N/A"), true);
            embed = embed.field("Source", details.source.as_deref().unwrap_or("N/A"), true);
            embed = embed.field("Episodes", details.episodes.map_or("N/A".to_string(), |e| e.to_string()), true);
            embed = embed.field("Status", &details.status, true);
            embed = embed.field("Airing", details.airing.map_or("N/A".to_string(), |b| b.to_string()), true);
            
            if let Some(aired_string) = &details.aired.string {
                embed = embed.field("Aired", aired_string, true);
            } else {
                let mut aired_dates = String::new();
                if let Some(from) = &details.aired.from {
                    aired_dates.push_str(&format!("From: {}", from));
                }
                if let Some(to) = &details.aired.to {
                    if !aired_dates.is_empty() { aired_dates.push_str(", "); }
                    aired_dates.push_str(&format!("To: {}", to));
                }
                if !aired_dates.is_empty() {
                    embed = embed.field("Aired", aired_dates, true);
                }
            }

            if let Some(duration) = &details.duration {
                embed = embed.field("Duration", duration, true);
            }
            if let Some(rating) = &details.rating {
                embed = embed.field("Rating", rating, true);
            }
            if let Some(score) = details.score {
                embed = embed.field("Score", format!("{}/10", score), true);
            }
            if let Some(scored_by) = details.scored_by {
                embed = embed.field("Scored by", format!("{} users", scored_by), true);
            }
            if let Some(rank) = details.rank {
                embed = embed.field("Rank", format!("#{}", rank), true);
            }
            if let Some(popularity) = details.popularity {
                embed = embed.field("Popularity", format!("#{}", popularity), true);
            }
            if let Some(members) = details.members {
                embed = embed.field("Members", members.to_string(), true);
            }
            if let Some(favorites) = details.favorites {
                embed = embed.field("Favorites", favorites.to_string(), true);
            }

            if let Some(trailer_meta) = &details.trailer {
                if let Some(youtube_id) = &trailer_meta.youtube_id {
                    embed = embed.field("Trailer", format!("<https://www.youtube.com/watch?v={}>", youtube_id), false);
                }
            }

            if !details.titles.is_empty() {
                let alternative_titles: Vec<String> = details.titles.iter()
                    .filter_map(|t| Some(t.title.clone()))
                    .take(3)
                    .collect();
                if !alternative_titles.is_empty() {
                    embed = embed.field("Alternative Titles", alternative_titles.join(", "), false);
                }
            }

            send_embed_message(ctx, msg, embed).await;
        },
        None => {
            if let Err(why) = msg.channel_id.say(&ctx.http, format!("Could not find details for MAL ID: {}. It might not exist or an error occurred.", mal_id)).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }
}

async fn handle_anime_recommendations_command(ctx: &Context, msg: &Message, mal_id: u32) {
    if let Err(why) = msg.channel_id.say(&ctx.http, format!("Fetching recommendations for MAL ID: {}...", mal_id)).await {
        println!("Error sending message: {:?}", why);
    }

    match jikan_api::get_anime_recommendations(mal_id).await {
        Some(recommendations) => {
            if recommendations.is_empty() {
                if let Err(why) = msg.channel_id.say(&ctx.http, format!("No recommendations found for MAL ID: {}.", mal_id)).await {
                    println!("Error sending message: {:?}", why);
                }
            } else {
                let mut embed = CreateEmbed::new()
                    .title(format!("Recommendations for MAL ID: {}", mal_id))
                    .timestamp(Timestamp::now());

                for (i, rec_item) in recommendations.iter().take(5).enumerate() {
                    let mut field_value = String::new();
                    if let Some(jpg_images) = &rec_item.entry.images.jpg {
                        if let Some(image_url) = &jpg_images.image_url {
                            field_value.push_str(&format!("{}\n", image_url));
                        }
                    }
                    field_value.push_str(&format!("{}", rec_item.entry.url));

                    embed = embed.field(
                        format!("{}. {} (MAL ID: {})", i + 1, rec_item.entry.title, rec_item.entry.mal_id),
                        field_value,
                        false,
                    );
                }

                send_embed_message(ctx, msg, embed).await;
            }
        },
        None => {
            if let Err(why) = msg.channel_id.say(&ctx.http, format!("An error occurred while fetching recommendations for MAL ID: {}. Please try again later.", mal_id)).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }
}

async fn handle_anime_search_command(ctx: &Context, msg: &Message, query: &str, user_id: String) {
    if let Err(why) = msg.channel_id.say(&ctx.http, format!("Searching for anime: '{}'...", query)).await {
        println!("Error sending message: {:?}", why);
    }

    let mut user_queries = load_queries().await;
    user_queries.queries.entry(user_id.clone()).or_insert_with(Vec::new).push(query.to_string());
    save_queries(&user_queries).await;

    match jikan_api::search_anime(query).await {
        Some(animes) => {
            if animes.is_empty() {
                if let Err(why) = msg.channel_id.say(&ctx.http, format!("No results found for '{}'.", query)).await {
                    println!("Error sending message: {:?}", why);
                }
            } else {
                let mut embed = CreateEmbed::new()
                    .title(format!("Search results for '{}'", query))
                    .timestamp(Timestamp::now());
                
                let mut description = String::new();
                for anime in animes {
                    description.push_str(&format!("- MAL ID: {}, Title: {}\n", anime.mal_id, anime.title));
                }
                embed = embed.description(description);
                embed = embed.footer(CreateEmbedFooter::new("To get more details, type `!anime details <MAL_ID>`"));

                send_embed_message(ctx, msg, embed).await;
            }
        },
        None => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "An error occurred while searching for anime. Please try again later.").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let content = msg.content.trim();
        let user_id = msg.author.id.to_string();

        if content == "!help" {
            let help_message = "Hello! I'm an anime bot. Here are the commands you can use:\n\n\
                                • `!anime <anime name>`: Searches for anime and lists basic information.\n\n\
                                • `!anime details <MAL_ID>`: Fetches detailed information for a specific anime by its MyAnimeList ID.\n\n\
                                • `!anime recommendations <MAL_ID>`: Fetches anime recommendations based on a given MyAnimeList ID.\n\n\
                                • `!lain`: Get details about Serial Experiments Lain.\n\n\
                                • `!lain recommendations`: Get recommendations based on Serial Experiments Lain.\n\n\
                                ----------------------------------------\n\n\
                                **Example Usage:**\n\
                                `!anime Attack on Titan`\n\
                                `!anime details 16498` (for Attack on Titan)\n\
                                `!anime recommendations 16498` (for Attack on Titan recommendations)\n\
                                `!lain`\n\
                                `!lain recommendations`";
            
            let embed = CreateEmbed::new()
                .title("Ferivonus Anime Bot Yardım Menüsü")
                .description(help_message)
                .timestamp(Timestamp::now());

            let builder = CreateMessage::new().embed(embed);
            if let Err(why) = msg.channel_id.send_message(&ctx.http, builder).await {
                println!("Error sending help message: {:?}", why);
            }
            return;
        }

        if content.starts_with("!anime details ") {
            let parts: Vec<&str> = content.splitn(3, ' ').collect();
            if parts.len() < 3 {
                if let Err(why) = msg.reply(&ctx.http, "Please provide an anime ID for details. Usage: `!anime details <MAL_ID>`").await {
                    println!("Error sending message: {:?}", why);
                }
                return;
            }
            let mal_id_str = parts[2].trim();
            let mal_id = match mal_id_str.parse::<u32>() {
                Ok(id) => id,
                Err(_) => {
                    if let Err(why) = msg.reply(&ctx.http, "Invalid MAL ID. Please provide a numeric ID.").await {
                        println!("Error sending message: {:?}", why);
                    }
                    return;
                }
            };
            handle_anime_details_command(&ctx, &msg, mal_id).await;
        }
        else if content.starts_with("!anime recommendations ") {
            let parts: Vec<&str> = content.splitn(3, ' ').collect();
            if parts.len() < 3 {
                if let Err(why) = msg.reply(&ctx.http, "Please provide an anime ID for recommendations. Usage: `!anime recommendations <MAL_ID>`").await {
                    println!("Error sending message: {:?}", why);
                }
                return;
            }
            let mal_id_str = parts[2].trim();
            let mal_id = match mal_id_str.parse::<u32>() {
                Ok(id) => id,
                Err(_) => {
                    if let Err(why) = msg.reply(&ctx.http, "Invalid MAL ID. Please provide a numeric ID.").await {
                        println!("Error sending message: {:?}", why);
                    }
                    return;
                }
            };
            handle_anime_recommendations_command(&ctx, &msg, mal_id).await;
        }
        else if content == "!lain" {
            let mal_id = 339; // MAL ID for Serial Experiments Lain
            handle_anime_details_command(&ctx, &msg, mal_id).await;
        }
        else if content == "!lain recommendations" {
            let mal_id = 339; // MAL ID for Serial Experiments Lain
            handle_anime_recommendations_command(&ctx, &msg, mal_id).await;
        }
        else if content.starts_with("!anime ") {
            let query = content.trim_start_matches("!anime ").trim();
            if query.is_empty() {
                if let Err(why) = msg.reply(&ctx.http, "Please provide an anime name to search. Usage: `!anime <anime name>` or `!anime details <MAL_ID>` or `!anime recommendations <MAL_ID>`").await {
                    println!("Error sending message: {:?}", why);
                }
                return;
            }
            handle_anime_search_command(&ctx, &msg, query, user_id).await;
        }
    }

    async fn presence_update(&self, _ctx: Context, _new_data: Presence) {
        println!("Presence Update");
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}