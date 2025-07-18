use anyhow::Error;
use poise::CreateReply;
use serenity::all::CreateEmbed;
use spoticord_session::manager::SessionQuery;
use spoticord_utils::discord::Colors;

use crate::bot::Context;

// highlight-start
#[poise::command(
    slash_command,
    guild_only,
    name = "atsijungti",
    description = "Atjungia botą iš balso kanalo."
)]
pub async fn disconnect(ctx: Context<'_>) -> Result<(), Error> {
// highlight-end
    let manager = ctx.data();
    let guild = ctx.guild_id().expect("poise lied to me");

    let Some(session) = manager.get_session(SessionQuery::Guild(guild)) else {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Negaliu atsijungti")
                        // highlight-next-line
                        .description("Aš neprisijungęs prie jokio balso kanalo.")
                        .color(Colors::Error),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    };

    if session.active().await? && session.owner().await? != ctx.author().id {
        ctx.send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        // highlight-start
                        .title("Veiksmas negalimas")
                        .description("Atjungti botą gali tik seansą pradėjęs narys.")
                        // highlight-end
                        .color(Colors::Error),
                )
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    session.disconnect().await;

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                // highlight-start
                .title("Atsijungiau!")
                .description("Sėkmingai atsijungiau. Naudokite `/prisijungti`, kad vėl mane pakviestumėte.")
                // highlight-end
                .color(Colors::Info),
        ),
    )
    .await?;

    Ok(())
}
