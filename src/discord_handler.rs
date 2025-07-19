use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::{Presence, Ready};
use serenity::prelude::*;

use crate::jikan_api;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let content = msg.content.trim();

        if content == "!help" {
            let help_message = "Hello! I'm an anime bot. Here are the commands you can use:\n\
                                `!anime <anime name>`: Searches for anime and lists basic information.\n\
                                `!anime details <MAL_ID>`: Fetches detailed information for a specific anime by its MyAnimeList ID.\n\
                                `!anime recommendations <MAL_ID>`: Fetches anime recommendations based on a given MyAnimeList ID.\n\
                                Example:\n\
                                `!anime Attack on Titan`\n\
                                `!anime details 16498` (for Attack on Titan)\n\
                                `!anime recommendations 16498` (for Attack on Titan recommendations)";
            if let Err(why) = msg.channel_id.say(&ctx.http, help_message).await {
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

            if let Err(why) = msg.channel_id.say(&ctx.http, format!("Fetching details for MAL ID: {}...", mal_id)).await {
                println!("Error sending message: {:?}", why);
            }

            match jikan_api::get_anime_details_by_id(mal_id).await {
                Some(details) => {
                    let mut response_message = format!("**{}** (MAL ID: {})\n", details.title, details.mal_id);
                    response_message.push_str(&format!("URL: <{}>\n", details.url));
                    
                    if let Some(title_english) = &details.title_english {
                        response_message.push_str(&format!("English Title: {}\n", title_english));
                    }
                    if let Some(title_japanese) = &details.title_japanese {
                        response_message.push_str(&format!("Japanese Title: {}\n", title_japanese));
                    }
                    if !details.title_synonyms.is_empty() {
                        response_message.push_str(&format!("Synonyms: {}\n", details.title_synonyms.join(", ")));
                    }

                    if let Some(approved) = details.approved {
                        response_message.push_str(&format!("Approved: {}\n", approved));
                    } else {
                        response_message.push_str("Approved: N/A\n");
                    }

                    if let Some(anime_type) = &details.anime_type {
                        response_message.push_str(&format!("Type: {}\n", anime_type));
                    }
                    if let Some(source) = &details.source {
                        response_message.push_str(&format!("Source: {}\n", source));
                    }
                    response_message.push_str(&format!("Episodes: {}\n", details.episodes.map_or("N/A".to_string(), |e| e.to_string())));
                    response_message.push_str(&format!("Status: {}\n", details.status));
                    if let Some(airing) = details.airing {
                        response_message.push_str(&format!("Airing: {}\n", airing));
                    } else {
                        response_message.push_str("Airing: N/A\n");
                    }
                    if let Some(aired_string) = &details.aired.string {
                        response_message.push_str(&format!("Aired: {}\n", aired_string));
                    } else {
                        if let Some(from) = &details.aired.from {
                            response_message.push_str(&format!("Aired From: {}\n", from));
                        }
                        if let Some(to) = &details.aired.to {
                            response_message.push_str(&format!("Aired To: {}\n", to));
                        }
                    }
                    if let Some(duration) = &details.duration {
                        response_message.push_str(&format!("Duration: {}\n", duration));
                    }
                    if let Some(rating) = &details.rating {
                        response_message.push_str(&format!("Rating: {}\n", rating));
                    }

                    if let Some(score) = details.score {
                        response_message.push_str(&format!("Score: {}/10\n", score));
                    }
                    if let Some(scored_by) = details.scored_by {
                        response_message.push_str(&format!("Scored by: {} users\n", scored_by));
                    }
                    if let Some(rank) = details.rank {
                        response_message.push_str(&format!("Rank: #{}\n", rank));
                    }
                    if let Some(popularity) = details.popularity {
                        response_message.push_str(&format!("Popularity: #{}\n", popularity));
                    }
                    if let Some(members) = details.members {
                        response_message.push_str(&format!("Members: {}\n", members));
                    }
                    if let Some(favorites) = details.favorites {
                        response_message.push_str(&format!("Favorites: {}\n", favorites));
                    }
                    
                    if let Some(synopsis) = details.synopsis {
                        let short_synopsis = if synopsis.len() > 1000 {
                            format!("{}...", &synopsis[..1000])
                        } else {
                            synopsis
                        };
                        response_message.push_str(&format!("Synopsis: {}\n", short_synopsis));
                    }

                    if let Some(images_res) = &details.images {
                        if let Some(jpg_images) = &images_res.jpg {
                            if let Some(image_url) = &jpg_images.image_url {
                                response_message.push_str(&format!("Image: <{}>\n", image_url));
                            }
                        }
                    }

                    if let Some(trailer_meta) = &details.trailer {
                        if let Some(youtube_id) = &trailer_meta.youtube_id {
                            response_message.push_str(&format!("Trailer: <https://www.youtube.com/watch?v={}>\n", youtube_id));
                        }
                    }

                    if !details.titles.is_empty() {
                        let alternative_titles: Vec<String> = details.titles.iter()
                            .filter_map(|t| Some(t.title.clone()))
                            .take(3)
                            .collect();
                        if !alternative_titles.is_empty() {
                            response_message.push_str(&format!("Alternative Titles: {}\n", alternative_titles.join(", ")));
                        }
                    }

                    if let Err(why) = msg.channel_id.say(&ctx.http, response_message).await {
                        println!("Error sending message: {:?}", why);
                    }
                },
                None => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, format!("Could not find details for MAL ID: {}. It might not exist or an error occurred.", mal_id)).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
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
                        let mut response_message = format!("Recommendations for MAL ID {}:\n", mal_id);
                        for (i, rec_item) in recommendations.iter().take(5).enumerate() {
                            response_message.push_str(&format!("{}. **{}** (MAL ID: {})\n", 
                                i + 1, 
                                rec_item.entry.title, 
                                rec_item.entry.mal_id
                            ));
                            if let Some(jpg_images) = &rec_item.entry.images.jpg {
                                if let Some(image_url) = &jpg_images.image_url {
                                    response_message.push_str(&format!("   Image: <{}>\n", image_url));
                                }
                            }
                            response_message.push_str(&format!("   URL: <{}>\n", rec_item.entry.url));
                        }
                        if let Err(why) = msg.channel_id.say(&ctx.http, response_message).await {
                            println!("Error sending message: {:?}", why);
                        }
                    }
                },
                None => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, format!("An error occurred while fetching recommendations for MAL ID: {}. Please try again later.", mal_id)).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
        }
        else if content.starts_with("!anime ") {
            let query = content.trim_start_matches("!anime ").trim();

            if query.is_empty() {
                if let Err(why) = msg.reply(&ctx.http, "Please provide an anime name to search. Usage: `!anime <anime name>` or `!anime details <MAL_ID>` or `!anime recommendations <MAL_ID>`").await {
                    println!("Error sending message: {:?}", why);
                }
                return;
            }

            if let Err(why) = msg.channel_id.say(&ctx.http, format!("Searching for anime: '{}'...", query)).await {
                println!("Error sending message: {:?}", why);
            }

            match jikan_api::search_anime(query).await {
                Some(animes) => {
                    if animes.is_empty() {
                        if let Err(why) = msg.channel_id.say(&ctx.http, format!("No results found for '{}'.", query)).await {
                            println!("Error sending message: {:?}", why);
                        }
                    } else {
                        let mut response_message = format!("Search results for '{}':\n", query);
                        for anime in animes {
                            response_message.push_str(&format!("  - MAL ID: {}, Title: {}\n", anime.mal_id, anime.title));
                        }
                        response_message.push_str("\nTo get more details, type `!anime details <MAL_ID>`");
                        if let Err(why) = msg.channel_id.say(&ctx.http, response_message).await {
                            println!("Error sending message: {:?}", why);
                        }
                    }
                },
                None => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, "An error occurred while searching for anime. Please try again later.").await {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
        }
    }

    async fn presence_update(&self, _ctx: Context, _new_data: Presence) {
        println!("Presence Update");
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
