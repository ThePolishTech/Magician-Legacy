use serenity::{
    builder::{
        CreateCommand, CreateEmbed,
        CreateInteractionResponse,
        CreateInteractionResponseMessage
    },
    client::Context,
    model::application::CommandInteraction
};
use crate::{
    event_handler,
    sql_scripts,
    utils::{
        create_log_message, EmbedColours, LogLevel
    }
};




pub fn build() -> CreateCommand {
    CreateCommand::new("register")
        .description("Add your profile to the database")
}

/// Register the invoking user's discord profile to the database
pub async fn run( interaction_data: &CommandInteraction, ctx: &Context, discord_bot: &event_handler::DiscordBot ) -> Option<CreateInteractionResponse> {


    // Our command can return more than one embed, so for simplicity's sake, we'll put all the code
    // into a code block, and return embeds from within it to a variable. That variable will be the
    // embed we'll later send to the user
    let embed_for_message = 'return_embed: {

        // The command inoker's user ID and tag will be needed later
        let invoking_user_id  = interaction_data.user.id.get();
        let invoking_user_tag = interaction_data.user.tag();

        // At this point we don't know if our user is in the database already or not. One way to
        // figure that out is to attempt to INSERT. If it succeedes, nice; if it fails with error
        // code 1555, then it means the user is already in the database (1555 being a primary key
        // constraint failure ). If not, then it's some unexpected error which we can just log.
        let query_result = sqlx::query( sql_scripts::discord_users::REGISTER )
            .bind( invoking_user_id as i64 )
            .execute( &discord_bot.database_connection )
            .await;

        // Here we'll check to see if our query worked, if the user is already in the database, or
        // if some other error occured
        match query_result {
            Ok(_) => {  // We'll ignore the count of rows changed, as we don't need it
                
                // We managed to enter the user into our database! Let's log it to console and
                // notify them
                println!("{}", create_log_message(
                        format!("Added {invoking_user_tag}'s profile").as_str(),
                        LogLevel::Info
                ));

                CreateEmbed::new()
                    .title("Success! You've been added to the database!")
                    .description("If you'd like to create a character, use \n/build_character")
                    .colour( EmbedColours::GOOD )                
            },
            Err( why ) => {

                // Well, something went wrong but it could be the PRIMARY KEY CONSTRAINT failure
                // we'll looking for. Let's check that
                if let sqlx::Error::Database( sqlite_error ) = &why {
                    let error_code = match sqlite_error.code() {
                        None => 0,  // If we can't get the code we're looking for, it might as well
                                    // not be the one we're looking for. 0 will do for this purpose
                        Some( code ) => code.into_owned()
                            .parse()
                            .unwrap_or(0)    // Same as before, if we can't parse it might as well
                                             // be a different code.
                    };

                    if error_code == 1555 {
                        // If we got here it means the user is in the database we are looking for,
                        // let's give them a bespoke message
                        break 'return_embed CreateEmbed::new()
                            .title("Your already in the database")
                            .description("No need to add you :P")
                            .color(EmbedColours::ERROR)
                    }
                }

                // If We got to this point, it means we have encountered a different error than
                // 1555, we need to respond with a error message
                println!("{}", create_log_message(
                        format!("Failed to add {invoking_user_tag}'s profile to the database:\n\t{why}").as_str(),
                        LogLevel::Warning
                ));

                CreateEmbed::new()
                    .title("A unexpected error occured")
                    .description("If it persists, feel free to open an issue on the bot's github page")
                    .color(EmbedColours::ERROR)

            }
        }

    };

    // We now load our resultant embed into a payload
    let new_response_message = CreateInteractionResponseMessage::new().embed( embed_for_message );
    let new_response         = CreateInteractionResponse::Message( new_response_message );
    let send_message_payload = interaction_data.create_response( &ctx.http, new_response );

    // Send the payload, report to Stdout if an error occurs
    if let Err( why ) = send_message_payload.await {
        println!("{}", create_log_message(
                format!("Failed to send response in /register:\n\t{}", why ).as_str(),
                LogLevel::Warning
        ))
    }

    None
}

