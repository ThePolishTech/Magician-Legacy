use serenity::{
    builder::{
        CreateCommand, CreateEmbed,
        CreateEmbedFooter,
        CreateInteractionResponse,
        CreateInteractionResponseMessage,
        EditInteractionResponse
    },
    client::Context,
    model::application::CommandInteraction,
    futures::StreamExt
};
use crate::{
    event_handler,
    sql_scripts::discord_users,
    utils::{
        create_log_message, EmbedColours, LogLevel
    }
};

// Conveniance function so that we don't have to write the format every single time
fn footer_test_index( test_index: u8, test_count: u8 ) -> CreateEmbedFooter {
    CreateEmbedFooter::new(
        format!( "Test {test_index}/{test_count}" )
    )
}




pub fn build() -> CreateCommand {
    CreateCommand::new("deregister")
        .description("Remove yourself from the database")
}




pub async fn run( interaction_data: &CommandInteraction, ctx: &Context, discord_bot: &event_handler::DiscordBot ) -> Option<CreateInteractionResponse> {

    // We'll be using the user's ID quite often, so lets just save it here for future use
    let invoking_user_id = interaction_data.user.id.get();

    // Because we'll be doing plenty of SQLite queries, even if theoretically and practically those
    // won't take long, I personally think it's a good idea to first aknowlage the user's command 
    let _ = interaction_data
        .create_response(&ctx.http, CreateInteractionResponse::Defer( CreateInteractionResponseMessage::new() ))
        .await;


    // There are many different messages that could get sent to the user, henceforth we shall use
    // a code block in order to simplify the process of sending a specific option

    // We will conduct a series of tests to see if we can safely remove the user's profile before
    // doing so    
    let embed_for_message = 'return_embed: {

        // For UX, we will add a footer whenever a test fails. It will state which test of how many
        // failed. It's so that if a user hits multiple tests, they know how many are left.
        const TOTAL_TEST_COUNT: u8 = 1;

        // --== PROFILE TEST ==-- //
            
            // Firstly, we need to check if the user even exists in the database or not, as
            // we cannot remove their profile if it doesn't even exist. To do that we will
            // preform a `SELECT` query, if it returns none, we will break early with a
            // corresponding error embed
            let mut rows = sqlx::query( discord_users::SELECT_BY_ID )
                .bind( invoking_user_id as i64 )    // The sqlx::Encode trait is not implemented
                                                    // for u64, but is for i64 hence the cast
                .fetch( &discord_bot.database_connection );
            
            if rows.next().await.is_none() {
                break 'return_embed CreateEmbed::new()
                    .title("Can't find you")
                    .description("Your profile is not in the database, and so it can't be removed")
                    .footer( footer_test_index(1, TOTAL_TEST_COUNT) )
                    .colour(EmbedColours::ERROR)
            }
        // ==--


        // If we haven't broken out of this block upto this point, it means that all tests have
        // passed. We can now move forward with removing the invoking user's database entry
        let query_result = sqlx::query( discord_users::REMOVE_ENTRY )
            .bind( invoking_user_id as i64 )
            .execute( &discord_bot.database_connection )
            .await;

        
        let invoking_user_tag = interaction_data.user.tag();  // We'll need the invoking user's tag
                                                              // for loging reasons later
        match query_result {
            Ok(_) => {  // We'll ignore the count of rows detected
                
                // Succeeding, we notify both stdout, and the user
                println!("{}", create_log_message(
                        format!("Removed {invoking_user_tag}'s profile").as_str(),
                        LogLevel::Info
                ));

                CreateEmbed::new()
                    .title( "Your profile has been successfully removed from the database" )
                    .description( "Aaaaand cut!" )
                    .colour( EmbedColours::GOOD )
            },
            Err( why ) => {
                // For some reason, our query has failed. Lets log the error and prepare an info
                // error to our user
                println!("{}", create_log_message(
                        format!("Failed to remove {invoking_user_tag}'s profile: \n\t{why}").as_str(),
                        LogLevel::Error
                ));

                CreateEmbed::new()
                    .title("A unexpected error occured")
                    .description("If it persists, feel free to open an issue on the bot's github page")
            }
        }

    };

    // We prepare a `EditInteractionResponse` with our embed to send and then prepare a payload
    // that we await in a further-down `if let` block to send our new embed to the end user
    let new_message = EditInteractionResponse::new().embed(embed_for_message);
    let edit_response_payload = interaction_data.edit_response( &ctx.http, new_message );


    // We change the earlier aknowlagement to the message we want to send
    if let Err( why ) = edit_response_payload.await {
        println!("{}", why );
    }

    // We manage sending the resulting message here in this function, no need to send a message
    // back up the pipe. So we'll just return a None
    None
}

