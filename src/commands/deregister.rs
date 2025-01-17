use rusqlite::OptionalExtension;
use serenity::{
    builder::{
        CreateCommand,
        CreateEmbed, CreateEmbedFooter,
        CreateInteractionResponse,
        CreateInteractionResponseMessage,
        EditInteractionResponse
    },
    client::Context, model::application::CommandInteraction
};
use crate::utils::{
    create_log_message, LogLevel,
    DatabaseConnectionContainer,
    EmbedColours
};
use crate::sql_scripts::discord_users;

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


pub async fn run( interaction_data: &CommandInteraction, ctx: &Context ) -> Option<CreateInteractionResponse> {

    // We'll be using the user's ID quite often, so lets just save it here for future use
    let invoking_user_id = interaction_data.user.id.get();

    // Because we'll be doing plenty of SQLite queries, even if theoretically and practically those
    // won't take long, I personally think it's a good idea to first aknowlage the user's command 
    let _ = interaction_data
        .create_response(&ctx.http, CreateInteractionResponse::Defer( CreateInteractionResponseMessage::new() ))
        .await;


    // Because we are taking the database connection from inside a mutex, if we don't drop it
    // before we attempt to return from the function the compiler will be displeased. Not really an
    // issue for us. We'll just use a code block
    let embed_for_message = {

        // --== LOCK THE DATABASE CONNECTION FOR FURTHER USE ==-- //

            // Get the `Arc<Mutex<...>>` we passed into `client.data`. And because we passed it we
            // know it WILL be there, as a result we can just `.expect()` the `Option<...>` that
            // `.get_mut()` gives us.
            let mut client_data = ctx.data.write().await;
            let connection_guard = client_data
                .get_mut::<DatabaseConnectionContainer>()
                .expect("Database connection should be passed in during creation of client");

            // Rust's Mutex<...> have a feature called poisoning, where if a thread panics while
            // holding a mutex guard, it will become "poisoned". This is a flag that signals to us
            // that the data inside could be invalid as a result.
            //
            // However, even if that were to occur in this program, our database connection will
            // still always remain valid. As a result if we *do* recieve a poisoned mutex, we will
            // send a warning to the terminal, then clear the poison
            if connection_guard.is_poisoned() {
                println!( "{}", create_log_message(
                    "DatabaseConnectionContainer Mutex was poisoned, proceeding to clear poison flag",
                    LogLevel::Warning
                ));
                connection_guard.clear_poison();
            }

            // Irrelevant to whether the mutex was or wasn't poisoned, it at this point will not be
            // considered as such. As a result after `.lock()`ing we will always obtain an `Ok(...)`
            // variant. Posioning is the only reason why `.lock()` can return an `Err(...)` variant.
            // At least as far as Rust 1.83.0 Documentation is concerned.
            let database_connection = connection_guard
                .lock()
                .expect("Poisoned flag should already have been cleared");
        // ==--
        
        // --== TESTS WHETHER PROFILE CAN BE DELETED ==-- //
        
            // Before we can safetly remove the user's profile, we need to check a few things. To
            // do so we'll have a code block that returns a result containing a `CreateEmbed` in
            // both variants, but the result's type will tell us whether we can proceed to remove
            // the user's profile
            let return_embed_result: Result<CreateEmbed, CreateEmbed> = 'test_block: {

                // If a first time user runs this command without fuffiling any of it's
                // requirements, they could be met with error after error frustrating them.
                // As a result, to help deal with any potential frustrations, each error embed
                // will contain a footer saying which test out of how many has failed. To make
                // sure it is kept in sync, this varable makes sure we only need to change the
                // count in one spot
                let total_test_count = 1;

                // --== PROFILE TEST ==-- //
                    
                    // One of the first things we need to check is whether the user exists in the
                    // database already or not. We'll use a database query here, however preparing
                    // and executing it could potentially error. This would only really happen in
                    // rare circumstances which we neither plan nor expect to happen. In this case
                    // we can just safely use a expect(). Granted a panic could result if something
                    // happens, but given we are operating in Tokio threads, the only thing the end
                    // user will see is a "command interaction failed", which is acceptable in this
                    // case.
                    let mut database_query = database_connection
                        .prepare( discord_users::SELECT_BY_ID )
                        .expect("This should not fail. Only doing so unexpectedly, for no fault of our own");

                    let database_query_result = database_query.query_row( [&invoking_user_id], |row| row.get::<usize, u64>(0) )
                        .optional()
                        .expect("This should not fail. Only doing so unexpectedly, for no fault of our own");

                    if database_query_result.is_none() {
                        let error_embed = CreateEmbed::new()
                            .title("You're not in the database")
                            .description("That's alright! With nothing to remove, we'll just do nothing more")
                            .footer( footer_test_index(1, total_test_count) )
                            .colour(EmbedColours::ERROR);
                        
                        break 'test_block Err( error_embed );
                    }
                // ==--
                
                // --== RETURN SUCCESS EMBED ==-- //
                
                    // If we got to this point of the code, that means all tests have passed.
                    // Otherwise we would have broken out of this code block earlier. And so
                    // we can return a general 'success embed'
                    let success_embed = CreateEmbed::new()
                        .title( "Your profile has been successfully removed from the database" )
                        .description( "Aaaaand cut!" )
                        .colour( EmbedColours::GOOD );

                    Ok( success_embed )
                // ==-- 
            };
        // ==--
        
        // --== REMOVE PROFILE OR EXIT WITH MESSAGE ==-- //
        
            // Both variants of our result contain an embed and regardless of which variant we
            // obtain, we will want to display that embed. However in the case of a Ok variant
            // it means all of our tests have passed and we can preform a query to remove our
            // user's profile from the database
            match return_embed_result {
                Ok( embed ) => {
                    
                    // Remove the user's profile
                    let _ = database_connection.execute( discord_users::REMOVE_ENTRY, [&invoking_user_id] );

                    // Get the user's unique tag, this is for our log message to be more readable
                    let invoking_user_tag = interaction_data.user.tag();

                    // Finally, We'll just notify that a profile was removed and who's
                    println!( "{}", create_log_message(
                            format!( "Removed {invoking_user_tag}'s profile from the database" ).as_str(),
                            LogLevel::Info
                        )
                    );

                    embed
                },
                Err( embed ) => embed
            }
        // ==--
    };
    
    // We change the earlier aknowlagement to the message we want to send
    if let Err( why ) = interaction_data.edit_response( &ctx.http, EditInteractionResponse::new().embed(embed_for_message) ).await {
        println!("{}", why );
    }

    // We manage sending the resulting message here in this function, no need to send a message
    // back up the pipe. So we'll just return a None
    None
}

