use serenity::{
    builder::{
        CreateCommand, CreateEmbed,
        CreateInteractionResponse,
        CreateInteractionResponseMessage
    },
    client::Context,
    model::application::CommandInteraction
};
use crate::{sql_scripts, utils::{
    create_log_message, DatabaseConnectionContainer, EmbedColours, LogLevel
}};




pub fn build() -> CreateCommand {
    CreateCommand::new("register")
        .description("Add your profile to the database")
}

/// Register the invoking user's discord profile to the database
pub async fn run( interaction_data: &CommandInteraction, ctx: &Context ) -> Option<CreateInteractionResponse> {

    // Further on, we'll have to use the invoking user's ID and tag. To simplify reading throught
    // the code, we'll put them into a variable
    let invoking_user_id  = interaction_data.user.id.get();
    let invoking_user_tag = interaction_data.user.tag();


    // Our command can return more than one embed, so for simplicity's sake, we'll put all the code
    // into a code block, and return embeds from within it to a variable. That variable will be the
    // embed we'll later send to the user
    let return_embed = 'embed_block: {

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

        // --== ATTEMPT TO INSERT INTO DATABSE ==-- //
            
            // There are three ways this code execution could go:
            // 1) User isn't registered;  2) User is registered;  3) Some other error has occured;
            //     Firstly, let's discuss the 3rd branch, an error occuring. This is an unlikely
            // event, one who's cause is no fault of our own (Unlike in the case of the user
            // already being on the database, but we'll get to that in a moment).
            //     Next case to conisder is the user already is in the database. This will cause an
            // error, but one we do forsee. Namely error code 1555 "SQLITE_CONSTRAINT_PRIMARYKEY".
            // In such a eventuallity we'll just inform the user they're already in the database
            //     Lastly, no error occurs whatsoever, in that eventuallity we'll just inform the
            // user of the success
            let query_cache = database_connection.execute( sql_scripts::discord_users::REGISTER, [&invoking_user_id] );
        // ==--

        // --== MATCH ON BRANCHES ==-- //
        
            // Now that we have sent the query, we can match on it's result to see which branch we
            // are on. Using that knowlage we shall break with the appropraite embed for our user
            if let Err( why ) = query_cache {
                let error_code = why
                    .sqlite_error()
                    .expect("database_connection.execute error should have a sqlite_error")
                    .extended_code;

                // As stated before, if we get error code 1555 then that means that the user is
                // already in the database
                if error_code == 1555 {
                    break 'embed_block CreateEmbed::new()
                        .title("You're already in the database")
                        .description("Want to remove yourself from the database? Use `/deregister`")
                        .colour(EmbedColours::ERROR)
                }

                // If it isn't 1555, then it's some other error. In this case we'll just send a
                // generic error code.
                break 'embed_block CreateEmbed::new()
                    .title("An unexpected error occured")
                    .description("This shouldn't happen, consider yourself lucky i guess. If the problem pressists please open an issue on this bot's github")
                    .colour(EmbedColours::ERROR)
            }

            // Finally, if there was no error, execution reaches here. Which means we can inform
            // the terminal and then break with a nice success embed for our dear user
            println!("{}", create_log_message(
                    format!("Added {invoking_user_tag}'s profile to the database").as_str(),
                    LogLevel::Info
            ));

            CreateEmbed::new()
                .title("Success! You've been added to the database!")
                .description("If you'd like to create a character, use \n/build_character")
                .colour(EmbedColours::GOOD)
        // ==--
    };

    // We now load our resultant embed into a payload
    let new_response_message = CreateInteractionResponseMessage::new().embed(return_embed);
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

