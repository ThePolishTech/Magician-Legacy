#[allow(unused_imports)]
use serenity::{
    all::{
        ActionRowComponent, CreateActionRow, CreateInputText,
        CreateModal, CreateSelectMenu, CreateSelectMenuOption, InputText, InputTextStyle, ModalInteraction}, builder::{
        CreateCommand, CreateEmbed,
        CreateInteractionResponse,
        CreateInteractionResponseMessage,
    }, client::Context, model::application::CommandInteraction,
    futures::StreamExt
};

use crate::{event_handler::DiscordBot, sql_scripts::characters, utils::{EmbedColours, create_log_message, LogLevel}};



pub fn build() -> CreateCommand {
    CreateCommand::new("build_character")
        .description("Build your character")
}

pub async fn run( interaction_data: &CommandInteraction, ctx: &Context ) -> Option<CreateInteractionResponse> {



    let modal_components = vec![
        CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Short, "Character Name", "name")),
        CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Short, "Character Species", "species")),
        CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Paragraph, "Character Backstory", "Backstory"))
    ];

    let new_modal = CreateModal::new("build_character", "Build a character")
        .components( modal_components );

    let a = CreateInteractionResponse::Modal(new_modal);
    let b = interaction_data.create_response(&ctx.http, a);

    if let Err(why) = b.await {
        println!("{}", why)
    }

    None
}

// After a user submits the modal, we need to parse the incoming data
pub async fn handle_modal( interaction_data: &ModalInteraction, ctx: &Context, discord_bot: &DiscordBot ) {

    // Later on we will need to use the invokering user's id for database queries. And tag for
    // logging purposes. We'll assign them here
    let invoking_user_id  = interaction_data.user.id.get();
    let invoking_user_tag = interaction_data.user.tag();
    
    // The modal that we recieve has 3 components, each containing a InputText action row
    // component. This is certain as we created the modal in the above function. That is
    // why it is alright if we just return early from the function as this should not occur
    // in production
    let character_data = {
        let mut data = vec![];

        for item in interaction_data.data.components.iter() {
            let result = match item.components[0].clone() {
                ActionRowComponent::InputText(data) => data.value.clone(),
                _ => return  // Shouldn't occur. See previous comment
            };

            if result.is_none() {
                return  // Shouldn't occur. See previous comment
            }

            data.push( result.expect("None case already handled") )
        }
    //    Name             Species          Backstory
        ( data[0].clone(), data[1].clone(), data[2].clone() )
    };

    let embed_for_message = 'return_embed: {

        // --== CHARACTER NAME UNIQUENESS TEST ==-- //

            // Check to see the user already has a character of the given name.
            // I would use a composite key in the SQL table, but we've got a foreign key in DiscordUsers
            // and those can't reference to a part of a composite key.
            let mut query_result = sqlx::query( characters::SELECT_BY_NAME_AND_OWNER_ID )
                .bind(invoking_user_id as i64)    // fk_discordId
                .bind(&character_data.0)    // Character Name
                .fetch( &discord_bot.database_connection );
            
            if query_result.next().await.is_some() {
                break 'return_embed CreateEmbed::new()
                    .title(format!("{} Is already in the database", character_data.0))
                    .description("If you want to remove them, use /delete_character")
                    .colour(EmbedColours::ERROR)
            }
        // ==--


        let query_result = sqlx::query( characters::ADD_CHARACTER )
        // -= Bind Values =- //
            .bind(invoking_user_id as i64)  // fk_discordId
            .bind(&character_data.0)        // Chracater Name
            .bind(&character_data.1)        // Chracater Species
            .bind(&character_data.2)        // Chracater Backstory
        // =-
            .execute( &discord_bot.database_connection )
            .await;

        match query_result {
            Ok(_) => {  // We don't need to worry about how many lines got modified
                CreateEmbed::new()
                    .title(format!("{} successfully added!", character_data.0))
                    .colour(EmbedColours::GOOD)
            },
            Err(why) => {
                // One of the errors that could occur is of code 787 which designates a FOREIGN KEY
                // constraint failure. This occurs when the user doesn't have a profile. So we need
                // to check for that and explain it to the user. Or if it's some other error, say
                // to try again
                if let sqlx::Error::Database( sqlite_error ) = &why {
                    let error_code = match sqlite_error.code() {
                        None => 0,  // If we can't get the code we're looking for, it might as well
                                    // not be the one we're looking for. 0 will do for this purpose
                        Some( code ) => code.into_owned()
                            .parse()
                            .unwrap_or(0)    // Same as before, if we can't parse it might as well
                                             // be a different code.
                    };

                    if error_code == 787 {
                        break 'return_embed CreateEmbed::new()
                            .title("You haven't been added to the database")
                            .description("You can add yourself by using /register. After that you can build your character!")
                            .colour(EmbedColours::ERROR);
                    }
                };

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
                format!("Failed to send response in /register:\n\t{}", why ),
                LogLevel::Warning
        ))
    }
}

