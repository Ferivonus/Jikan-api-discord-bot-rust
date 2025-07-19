use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::{Presence, Ready};
use serenity::prelude::*;

// jikan_api modülünü içeri aktarıyoruz
use crate::jikan_api;

// Discord bot olay yöneticisi
pub struct Handler;

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

            match jikan_api::search_anime(query).await {
                // jikan_api modülünden çağırıyoruz
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

    // This event will be dispatched for guilds, but not for direct messages.
    // async fn message(&self, _ctx: Context, msg: Message) {
    //     println!("Received message: {}", msg.content);
    // }

    // As the intents set in this example, this event shall never be dispatched.
    // Try it by changing your status.
    async fn presence_update(&self, _ctx: Context, _new_data: Presence) {
        println!("Presence Update");
    }

    // Set a handler to be called on the `ready` event.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
