use serenity::{
    all::CreateInteractionResponse, builder::{
        CreateCommand, CreateEmbed,
        CreateInteractionResponseMessage
    }, client::Context, model::application::CommandInteraction
};
use crate::utils::{
    create_log_message, LogLevel,
    DatabaseConnectionContainer,
    EmbedColours
};
use crate::sql_scripts::discord_users;

pub fn build() -> CreateCommand {
    CreateCommand::new("register")
        .description("Add your profile to the database")
}
pub async fn run( interaction_data: &CommandInteraction, ctx: &Context ) -> Option<CreateInteractionResponse> {
    
    // We'll use this to grab the Discord ID of the user who called the `/register` command
    // This will be used as the primary key in our `DiscordUsers` database table
    let invoking_user_id = interaction_data.user.id.get();

    // Next we need to grab a lock on the database connection
    let mut client_data = ctx.data.write().await;
    let database_connection = match client_data.get_mut::<DatabaseConnectionContainer>() {
        Some( connection ) => {
            match connection.lock() {
                Ok( connection ) => {
                    connection
                },
                Err( why ) => {
                    // TODO: Add check to see if lock failed due to poisoning or if it's already
                    // locked. If poisoned, replace with new database connection
                    println!("{}", create_log_message(
                            format!("Failed to lock Database Connection:\n\t{why}").as_str(),
                            LogLevel::Warning
                        )
                    );
                    why.into_inner()
                }
            }
        },
        None => {
            println!("{}",
                create_log_message("Failed to lock Database Connection", LogLevel::Warning )
            );
            return None;
        }
    };

    // Armed with the knowlage of the Discord user ID of the user invoking this command, we can
    // attempt to add them to the database. There are 3 ways this can go:
    // 1) It works no issue    2) It fails as they're already in the db    3) Some other error
    // In the first two's case, we'll inform the user with an appropriate embed, in the last case?
    // Idk, not sure how an error like that *could* happen, and it's not critical, so we can just
    // ignore it safetly.
    let return_embed = {
        let cache = match database_connection.execute(discord_users::REGISTER, [&invoking_user_id] ) {
            Ok(_) => {
                CreateEmbed::new()
                    .title( "Successfully added you to the database" )
                    .description( "Want to create your character? Use `/build_character`!" )
                    .colour( EmbedColours::GOOD )
            },
            Err( failure_reason ) => {
                if let rusqlite::Error::SqliteFailure( error, _ ) = failure_reason {

                    // If the error code is 1555, than that means the error is related to a PRIMARY KEY
                    // CONSTRAINT failure. And the only reason that can occur is if the user is already
                    // in the database. In that case we can just inform the invoking user.
                    if error.extended_code == 1555 {
                        CreateEmbed::new()
                            .title("You're already registered")
                            .description("Want to remove yourself from the database? Use `/unregister`")
                            .colour( EmbedColours::ERROR )

                    } else {
                        return None; // If it didn't fail for that reason, it's an unexepcted failure and
                                // should be ok to just fail the interaction
                    }
                } else {
                    return None;     // I don't *see* why logging it would improve anything. This shouldn't
                                // fail under most circumstances. Worst case I'll just add it later
                                // potentially under a Result so that I dont have to nest ugly
                                // log-to-console lines
                }
            }
        };
        cache.clone()
    };


    // Because we return from the function if a unexpected error occurs, we know that anything
    // bellow this line is on the "Happy Track" of everything going smoothly. That means that
    // we can now send a response without worrying about any potential issues.
    //
    // We do so here:
    let response_message = CreateInteractionResponseMessage::new().embed(return_embed);
    let response = CreateInteractionResponse::Message(response_message);
    Some( response )
}

