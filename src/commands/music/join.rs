use std::time::Duration;

use anyhow::Result;
use log::error;
use poise::CreateReply;
use serenity::all::{
    Channel, ChannelId, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, UserId,
};
use spoticord_database::error::DatabaseError;
use spoticord_session::manager::SessionQuery;
use spoticord_utils::discord::Colors;

use crate::bot::Context;

/// Join the current voice channel
#[poise::command(slash_command, guild_only)]
pub async fn join(ctx: Context<'_>) -> Result<()> {
    let guild = ctx.guild_id().expect("poise lied to me");
    let manager = ctx.data();

    let Some(guild) = guild
        .to_guild_cached(ctx.serenity_context())
        .map(|guild| guild.clone())
    else {
        error!("Unable to fetch guild from cache, how did we get here?");

        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("An error occured")
                        .description("This server hasn't been cached yet?")
                        .color(Colors::Error),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    };

    let Some(channel) = guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|state| state.channel_id)
    else {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Negaliu prisijungti")
                        .description("Turite būti kanale kad galėtumėte naudoti /join")
                        .color(Colors::Error),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    };

    if !has_voice_permissions(ctx, channel).await? {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Cannot join voice channel")
                        .description(
                            "The voice channel you are in is not available.
                                    I might not have the permissions to see this channel.",
                        )
                        .color(Colors::Error),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    if !has_text_permissions(ctx, ctx.channel_id()).await? {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Negalima prisijungti")
                        .description(
                            "Neturiu leidimo pasiekti tavo kanalą.",
                        )
                        .color(Colors::Error),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    // Check whether the user has linked their Spotify account
    if let Err(DatabaseError::NotFound) = manager
        .database()
        .get_account(ctx.author().id.to_string())
        .await
    {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Nėra Spotify paskyros")
                        .description(
                            "Tau reikia prisijungti savo spotify paskyra kad galėtum naudotis botu.\nNaudok /link komandą.",
                        )
                        .color(Colors::Error),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    let mut session_opt = manager.get_session(SessionQuery::Guild(guild.id));

    // Check if this server already has a session active
    if let Some(session) = &session_opt {
        if session.active().await? {
            ctx.send(
                CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .title("Spotify Užsiėmes")
                            .description("Spotify jau yra naudojamas šiame serveryje.")
                            .color(Colors::Error),
                    )
                    .ephemeral(true),
            )
            .await?;

            return Ok(());
        }
    }

    // Prevent the user from using Spoticord simultaneously in multiple servers
    if let Some(session) = manager.get_session(SessionQuery::Owner(ctx.author().id)) {
        let server_name = session.guild().to_partial_guild(&ctx).await?.name;

        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Tu jau naudoti Spotify")
                        .description(format!(
                            "Jau naudoji Spotify `{}`\n\n\
                            Sustabdyk savo sesija prieš pradedant naują.",
                            spoticord_utils::discord::escape(server_name)
                        ))
                        .color(Colors::Error),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    ctx.defer().await?;

    if let Some(session) = &session_opt {
        if session.voice_channel() != channel {
            session.disconnect().await;
            session_opt = None;

            // Give serenity/songbird some time to register the disconnect
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    if let Some(session) = session_opt {
        if let Err(why) = session.reactivate(ctx.author().id).await {
            error!("Failed to reactivate session: {why}");

            ctx.send(
                CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .title("Failed to reactivate session")
                            .description(
                                "An error occured whilst trying to reactivate the session. Please try again.",
                            )
                            .color(Colors::Error),
                    )
                    .ephemeral(true),
            )
            .await?;

            return Ok(());
        }
    } else if let Err(why) = manager
        .create_session(
            ctx.serenity_context(),
            guild.id,
            channel,
            ctx.channel_id(),
            ctx.author().id,
        )
        .await
    {
        error!("Failed to create session: {why}");

        let description = if matches!(why, spoticord_session::error::Error::AuthenticationFailed) {
            "Unable to authenticate with Spotify. Did you change your password?\n\nThe broken credentials used have been deleted.\n\nYou might need to relink your account using `/link`."
        } else {
            "An error occured whilst trying to create a session. Please try again."
        };

        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Failed to create session")
                        .description(description)
                        .color(Colors::Error),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .author(
                    CreateEmbedAuthor::new("Prisijungta prie kanalo")
                        .icon_url("https://spoticord.com/speaker.png"),
                )
                .description(format!("Ateikite paklausyti muzikos į <#{}>", channel))
                .footer(CreateEmbedFooter::new(
                    "Turite pasirinkti bota per spotify",
                ))
                .color(Colors::Info),
        ),
    )
    .await?;

    Ok(())
}

async fn has_voice_permissions(ctx: Context<'_>, channel: ChannelId) -> Result<bool> {
    let me: UserId = ctx.cache().current_user().id;

    let Ok(Channel::Guild(channel)) = channel.to_channel(ctx).await else {
        return Ok(false);
    };

    let Ok(permissions) = channel.permissions_for_user(ctx, me) else {
        return Ok(false);
    };

    Ok(permissions.view_channel() && permissions.connect() && permissions.speak())
}

async fn has_text_permissions(ctx: Context<'_>, channel: ChannelId) -> Result<bool> {
    let me: UserId = ctx.cache().current_user().id;

    let Ok(Channel::Guild(channel)) = channel.to_channel(ctx).await else {
        return Ok(false);
    };

    let Ok(permissions) = channel.permissions_for_user(ctx, me) else {
        return Ok(false);
    };

    Ok(permissions.view_channel() && permissions.send_messages() && permissions.embed_links())
}
