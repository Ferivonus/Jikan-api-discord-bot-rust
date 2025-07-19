use dotenv::dotenv;
use serde::Deserialize;
use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::{Presence, Ready};
use serenity::prelude::*;

// Structure for an Anime item from Jikan API response
#[derive(Debug, Deserialize)]
struct Anime {
    mal_id: u32, // Jikan uses mal_id for the anime ID
    title: String,
}

// Top-level search response structure for Jikan API
#[derive(Debug, Deserialize)]
struct JikanSearchResponse {
    data: Vec<Anime>,
}

/// Searches for anime on the Jikan API.
/// Returns search results using the provided query.
/// This function is asynchronous.
async fn search_anime(query: &str) -> Option<Vec<Anime>> {
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

// Discord bot olay yöneticisi
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // This function is called whenever a new message is received.
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages from bots to prevent infinite loops
        if msg.author.bot {
            return;
        }

        // Check if the message starts with "!anime "
        if msg.content.starts_with("!anime ") {
            let query = msg.content.trim_start_matches("!anime ").trim();

            if query.is_empty() {
                if let Err(why) = msg
                    .reply(
                        &ctx.http,
                        "Please provide an anime name to search. Usage: `!anime <anime name>`",
                    )
                    .await
                {
                    println!("Error sending message: {:?}", why);
                }
                return;
            }

            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, format!("Searching for anime: '{}'...", query))
                .await
            {
                println!("Error sending message: {:?}", why);
            }

            match search_anime(query).await {
                Some(animes) => {
                    if animes.is_empty() {
                        if let Err(why) = msg
                            .channel_id
                            .say(&ctx.http, format!("No results found for '{}'.", query))
                            .await
                        {
                            println!("Error sending message: {:?}", why);
                        }
                    } else {
                        let mut response_message = format!("Search results for '{}':\n", query);
                        for anime in animes {
                            response_message.push_str(&format!(
                                "  - MAL ID: {}, Title: {}\n",
                                anime.mal_id, anime.title
                            ));
                        }
                        if let Err(why) = msg.channel_id.say(&ctx.http, response_message).await {
                            println!("Error sending message: {:?}", why);
                        }
                    }
                }
                None => {
                    if let Err(why) = msg
                        .channel_id
                        .say(
                            &ctx.http,
                            "An error occurred while searching for anime. Please try again later.",
                        )
                        .await
                    {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
        }
    }

    // Set a handler to be called on the `ready` event.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    dotenv().ok(); // Load .env file
    let token = env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN in the environment.");

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES // For messages in guilds
        | GatewayIntents::DIRECT_MESSAGES // For direct messages
        | GatewayIntents::MESSAGE_CONTENT; // Crucial for reading message content

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
