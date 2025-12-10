use dotenv::dotenv;
use twilight_util::builder::embed::{
    EmbedAuthorBuilder, EmbedBuilder, EmbedFieldBuilder, ImageSource,
};
use xitca_web::{App, codegen::route, error::Error, handler::json::LazyJson, middleware::Logger};

mod custom_error;
mod discord;
mod middleware;
mod watch_tower;

use custom_error::BadRequest;
use watch_tower::request_body::UpdateRequestBody;

#[route("/update",method = post)]
async fn watchtower_notification(
    body: Option<LazyJson<UpdateRequestBody<'_>>>,
) -> Result<&'static str, Error> {
    let valid_body = match body {
        Some(b) => b,
        None => return Err(BadRequest::new("Request body is missing").into()),
    };

    let body = valid_body.deserialize()?;
    if body.updated_containers.len() == 0 {
        return Err(BadRequest::new("No updated containers found").into());
    }

    if body.updated_containers.iter().all(|c| {
        let container_name = c.name.trim().trim_start_matches('/');
        !container_name.starts_with("SE3")
    }) {
        return Ok("No SE3 containers updated, skipping Discord notification");
    }

    let embed_author = EmbedAuthorBuilder::new("WatchTower")
        .icon_url(
            ImageSource::url("https://containrrr.dev/watchtower/images/logo-450px.png").unwrap(),
        )
        .build();

    // Build embed with release information
    let mut embed = EmbedBuilder::new()
        .title("Containers below have been updated")
        .author(embed_author)
        .color(0x406170);

    for container in body.updated_containers.iter() {
        let container_name = container.name.trim().trim_start_matches('/');
        if !container_name.starts_with("SE3") {
            continue;
        }
        let field = EmbedFieldBuilder::new(
            container_name,
            format!(
                "**Image:** {}\n**Old ID:** `{}`\n**New ID:** `{}`",
                container.image, container.old_id, container.new_id
            ),
        )
        .build();
        embed = embed.field(field);
    }

    let embed = embed.build();

    if let Err(e) = discord::send_embed(embed).await {
        eprintln!("Failed to send Discord embed: {}", e);
        // Don't fail the webhook even if Discord message fails
    }

    Ok("Finished")
}

#[route("/",method = get)]
async fn root() -> &'static str {
    "Hello, World"
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Initialize Discord bot
    let discord_token =
        std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN environment variable must be set");

    // Initialize Discord HTTP client and channel ID
    discord::init_http_client(discord_token.clone())
        .expect("Failed to initialize Discord HTTP client");

    discord::init_channel_id().expect("Failed to initialize Discord channel ID");

    // Start Discord gateway to make bot online
    discord::start_gateway(discord_token)
        .await
        .expect("Failed to start Discord gateway");

    println!("Discord bot is now online!");

    App::new()
        .at_typed(root)
        .at_typed(watchtower_notification)
        .enclosed_fn(middleware::error_handler)
        .enclosed(Logger::new())
        .serve()
        .bind(("0.0.0.0", 3000))?
        .run()
        .await
}
