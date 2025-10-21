use futures::StreamExt;
use std::sync::Arc;
use std::sync::OnceLock;
use twilight_gateway::{Config, Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Embed;
use twilight_model::id::{Id, marker::ChannelMarker};

// Static storage for Discord channel ID
static CHANNEL_ID: OnceLock<Id<ChannelMarker>> = OnceLock::new();

// Static storage for Discord HTTP client
static HTTP_CLIENT: OnceLock<Arc<HttpClient>> = OnceLock::new();

/// Initialize the Discord channel ID from environment variable
pub fn init_channel_id() -> Result<(), String> {
    let channel_id_str = std::env::var("DISCORD_CHANNEL_ID")
        .map_err(|_| "DISCORD_CHANNEL_ID environment variable not set")?;

    let channel_id: u64 = channel_id_str
        .parse()
        .map_err(|_| "DISCORD_CHANNEL_ID must be a valid number")?;

    CHANNEL_ID
        .set(Id::new(channel_id))
        .map_err(|_| "Channel ID already initialized")?;

    Ok(())
}

/// Initialize the Discord HTTP client
pub fn init_http_client(token: String) -> Result<(), String> {
    let client = Arc::new(HttpClient::new(token));
    HTTP_CLIENT
        .set(client)
        .map_err(|_| "HTTP client already initialized")?;

    Ok(())
}

/// Get the Discord channel ID
pub fn get_channel_id() -> Result<Id<ChannelMarker>, String> {
    CHANNEL_ID
        .get()
        .copied()
        .ok_or_else(|| "Channel ID not initialized".to_string())
}

/// Get the Discord HTTP client
pub fn get_http_client() -> Result<Arc<HttpClient>, String> {
    HTTP_CLIENT
        .get()
        .cloned()
        .ok_or_else(|| "HTTP client not initialized".to_string())
}

/// Send a message to the Discord channel
pub async fn send_message(content: String) -> Result<(), String> {
    let channel_id = get_channel_id()?;
    let client = get_http_client()?;

    client
        .create_message(channel_id)
        .content(&content)
        .await
        .map_err(|e| format!("Failed to send message: {}", e))?;

    Ok(())
}

/// Send an embed to the Discord channel
pub async fn send_embed(embed: Embed) -> Result<(), String> {
    let channel_id = get_channel_id()?;
    let client = get_http_client()?;

    client
        .create_message(channel_id)
        .embeds(&[embed])
        .await
        .map_err(|e| format!("Failed to send embed: {}", e))?;

    Ok(())
}

/// Start the Discord gateway shard (makes bot online)
pub async fn start_gateway(token: String) -> Result<(), String> {
    let config = Config::new(token, Intents::empty());

    let mut shard = Shard::with_config(ShardId::ONE, config);

    // Spawn the gateway processing in a background task
    tokio::spawn(async move {
        loop {
            let event = match shard.next().await {
                Some(Ok(event)) => event,
                Some(Err(source)) => {
                    eprintln!("Error receiving event: {:?}", source);
                    continue;
                }
                None => break,
            };

            // Process events (we don't need to handle any specific events for this bot)
            // The gateway connection itself keeps the bot online
            tokio::spawn(async move {
                // Event processing can be added here if needed
                drop(event);
            });
        }
    });

    Ok(())
}
