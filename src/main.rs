use dotenv::dotenv;
use std::env;

use serenity::prelude::*; // Presence da burada kullanılmadığı için kaldırılabilir, ancak örneğinizde vardı.

// Yeni modülleri tanımlıyoruz
mod discord_handler;
mod jikan_api; // Handler struct'ı burada

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    dotenv().ok(); // Load .env file
    let token = env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN in the environment.");

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES // For messages in guilds
        | GatewayIntents::DIRECT_MESSAGES // For direct messages
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS; // GUILDS intent'i de Presence için gerekli olabilir

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(discord_handler::Handler) // Handler'ı discord_handler modülünden çağırıyoruz
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
