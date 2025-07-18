use anyhow::Error;
use poise::CreateReply;
use serenity::all::CreateEmbed;
use spoticord_session::manager::SessionQuery;
use spoticord_utils::discord::Colors;
use crate::bot::Context;



#[poise::command(slash_command, guild_only)]
pub async fn disconnect(ctx: Context<'_>) -> Result<(), Error> {
    let manager = ctx.data();
    let guild = ctx.guild_id().expect("poise lied to me");


    let Some(session) = manager.get_session(SessionQuery::Guild(guild)) else {
        ctx.send(
            CreateReply::default()
               .embed(
                    CreateEmbed::new()
                       .title("Negaliu atsijungti")
                        .description("Aš nesu jokiame kanale.")
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
                       .title("Negaliu atsijungti")
                        .description("Tik muzikos leidėjas gali atlikti šį veiksmą.")
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
                .title("Atsijungimas!")
                .description("Atsijungta, naudokite /join komandą kad prisijungčiau.")
                .color(Colors::Info),

        ),
    )
 .await?;


    Ok(())
}
