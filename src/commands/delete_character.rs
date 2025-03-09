// Remove a user's character
//
// - Dispacted with a slash command, which has an autocomplete field to select character by search
//     (it returns character ids)
// - If the code recives a valid character_id that belongs to the user a modal will be dispatched
//     to the user
// - The modal will contain one text field, the user will have to type in the character's name to
//     confirm (The character's name will be given in the modal)
// - Next up, a database query will remove the character, then if nothing fails the character will
//     be removed from the character cache

use std::sync::Mutex;

// Temp, to catch rust analyser auto adding stuff
#[allow(unused_imports, unused_variables)]

use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateCommandOption, CreateInteractionResponse, ModalInteraction,
    AutocompleteChoice, AutocompleteOption, CreateAutocompleteResponse,
    ResolvedValue,
    CreateModal, CreateActionRow, ActionRow, ActionRowComponent,
    CreateInputText, InputTextStyle
};

use serenity::{all::{CreateEmbed, CreateInteractionResponseMessage}, builder::{
//    CreateCommand, CreateCommandOption,
}};

use crate::{
    event_handler::DiscordBot, sql_scripts::characters, utils::{
        clone_user_characters, create_log_message, DatabaseCharactersCache, EmbedColours, LogLevel
    }
};


/// Build the delete_character's command signature to be sent to Discord's Gateway
pub fn build() -> CreateCommand {
    let options = vec![
        CreateCommandOption::new(
                serenity::all::CommandOptionType::Number,
                "character",
                "The character to be removed. Must belong to you"
            )
            .required(true)
            .set_autocomplete(true)
    ];

    CreateCommand::new("delete_character")
        .description("Remove a character")
        .set_options(options)
}


// After receiving the event, run it
pub async fn run( interaction_data: &CommandInteraction, ctx: &Context ) -> Option<CreateInteractionResponse> {

    let invoking_user_id = interaction_data.user.id.get();

    let response_payload = 'with_response: {
        
        let selected_id = match interaction_data.data.options()[0].value {
            ResolvedValue::Number(num) => num as u16,
            _ => {
                return None
            }
        };

        // --== GET CLONE OF USER CHARACTER DATA ==-- //
        
            // 
            let user_owned_characters = {
                let data_read = ctx.data.read().await;
                let character_map_mutex = match data_read.get::<DatabaseCharactersCache>(){
                    Some(mutex) => mutex,
                    None => {
                        // This branch only gets executed when our TypeMap in Context.data doesn't
                        // contain the 'DatabaseCharactersCache' key. This shouldn't ever really
                        // occur due to the fact that it gets inserted in main.rs
                        return None;
                    }
                };

                clone_user_characters(character_map_mutex.clone(), &invoking_user_id).unwrap_or(vec![])
            };
        // ==-- 
        
        // --== GET SELECTED CHARACTER ==-- //
        
            //
            let found_characters = user_owned_characters
                .iter()
                .filter( |x| x.0 == selected_id )
                .collect::<Vec<&(u16, String)>>();

            let found_character = found_characters.first();

            match found_character {
                Some(character) => {
                    let modal_components = vec![
                        CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Short, "Please confirm character name", &character.1))
                    ];
                    let modal = CreateModal::new(
                            format!("delete_character:{}", &character.0 ),
                            format!("Deleting {}", &character.1 )
                        )
                        .components(modal_components);
                    
                    CreateInteractionResponse::Modal(modal)
                },
                None => {
                    let embed = CreateEmbed::new()
                        .title("Selected character doesn't belong to you")
                        .description("We couldn't find the selected character from your owned ones")
                        .colour(EmbedColours::ERROR);
                    let payload = CreateInteractionResponseMessage::new().embed(embed);

                    CreateInteractionResponse::Message(payload)
                }
            }
        // ==--
    };

    let response = interaction_data.create_response( &ctx, response_payload );

    if let Err(why) = response.await {
        println!("Failed to send modal response in /delete_character:\n\t{why}")
    }

    None
}


//
pub async fn handle_autocomplete( interaction_data: &CommandInteraction, ctx: &Context ) {

    let invoking_user_id = interaction_data.user.id.get();

    let character_choices: Vec<(u16, String)> = 'character_data: {

        //
        let query = match interaction_data.data.options()[0].value {
            ResolvedValue::Autocomplete { value, .. } => value.to_lowercase(),
            _ => {
                println!("{:#?}", interaction_data.data.options()[0].value);
                break 'character_data vec![]
            }
        };

        //
        let users_characters = {
            
            let data_read = ctx.data.read().await;
            let users_character_map_mutex = match data_read.get::<DatabaseCharactersCache>() {
                Some(mutex) => mutex,
                None => {
                    // This shouldn't ever occur as the TypeMapKey should be inserted in main.rs
                    break 'character_data vec![];
                }
            };

            let user_character_map = match users_character_map_mutex.lock() {
                Ok(map) => map,
                Err(_) => {
                    // This can only happen when the mutex is locked, which shouldn't ever happen
                    break 'character_data vec![];
                }
            };

            let user_character_data = match user_character_map.get(&invoking_user_id) {
                Some(data) => data,
                None => {
                    println!("Userid is missing");
                    &vec![]
                }
            };

            user_character_data.clone()
        };

        let mut begins_with_choices = vec![];
        let mut contains_choices    = vec![];

        for character_data in users_characters.iter() {

            let character_name = character_data.1.to_lowercase();

            match (character_name.starts_with(&query), character_name.contains(&query)) {
                (true, _) => begins_with_choices.push(character_data.clone()),
                (false, true) => contains_choices.push(character_data.clone()),
                _ => { /* Do Nothing */ }
            }
        }
        
        [ begins_with_choices, contains_choices ].concat()
    };

    let autocomplete_choices = {
        let mut choices = vec![];
        
        for character_data in character_choices.iter() {
            let (character_id, character_name) = character_data;
            choices.push(
                AutocompleteChoice::new(character_name, *character_id)
            );
        }

        choices
    };

    let response = CreateAutocompleteResponse::new().set_choices(autocomplete_choices);
    let response_payload = CreateInteractionResponse::Autocomplete(response);
    let send_response = interaction_data.create_response( &ctx.http, response_payload );

    if let Err(why) = send_response.await {
        println!("{}", create_log_message(
                format!("Failed to send autocomple response:\n\t{why}"), 
                LogLevel::Warning
        ))
    }

} 

//
pub async fn handle_modal( interaction_data: &ModalInteraction, ctx: &Context, discord_bot: &DiscordBot ) {

    let invoking_user_id = interaction_data.user.id.get();
    let invoking_user_tag = interaction_data.user.tag();
    
    let target_character_id: u16 = interaction_data.data.custom_id
        .split(':')
        .collect::<Vec<&str>>()[1]
        .parse()
        .expect("We only recive a number from gateway, it should have no issue parsing");

    let target_character_name = {
            
            let user_owned_characters = {
                let data_read = ctx.data.read().await;
                let character_map_mutex = match data_read.get::<DatabaseCharactersCache>(){
                    Some(mutex) => mutex,
                    None => {
                        // This branch only gets executed when our TypeMap in Context.data doesn't
                        // contain the 'DatabaseCharactersCache' key. This shouldn't ever really
                        // occur due to the fact that it gets inserted in main.rs
                        return
                    }
                };

                clone_user_characters(character_map_mutex.clone(), &invoking_user_id)
                    .unwrap_or(vec![])
            };
        // ==-- 
        
        // --== GET SELECTED CHARACTER ==-- //
        
            //
            let found_characters = user_owned_characters
                .iter()
                .filter( |x| x.0 == target_character_id )
                .collect::<Vec<&(u16, String)>>();

            let character_name = match found_characters.first() {
                None => {
                    // This shouldn't happen, so in this case i'll just return from the function
                    return
                },
                Some(data) => data.1.clone()
            };
            character_name
        // ==--
    };

    let query_result = sqlx::query( characters::REMOVE_CHARACTER )
        .bind( target_character_id )
        .execute( &discord_bot.database_connection )
        .await;

    let return_response = match query_result {
        Ok(_) => {

            // --== UPDATE CACHE ==-- //
            
                //
                {
                    let data_read = ctx.data.read().await;
                    let character_map_arc = data_read
                        .get::<DatabaseCharactersCache>()
                        .expect("Key should be in map as it gets inserted in main.rs");

                    let mut character_map_mut = match character_map_arc.lock() {
                        Ok(data) => data,
                        Err(why) => return
                    };

                    let user_characters = match character_map_mut.get_mut(&invoking_user_id) {
                        Some(vec_of_chars) => vec_of_chars,
                        None => return
                    };

                    user_characters.retain( |character| character.0 != target_character_id );
                    
                }
            // ==--
            
            let embed = CreateEmbed::new()
                .title("Successfully removed {target_character_name}")
                .description("They are now gone")
                .colour(EmbedColours::GOOD);

            let response_message = CreateInteractionResponseMessage::new().embed(embed);
            CreateInteractionResponse::Message(response_message)

        },
        Err(why) => {

            println!("{}", create_log_message(
                    format!("Failed to remove {}'s character:\n\t{}", invoking_user_tag, why),
                    LogLevel::Warning
            ));

            let embed = CreateEmbed::new()
                .title("An unexpected Error occured")
                .description("If this persists, feel free to open an issue on github")
                .colour(EmbedColours::ERROR);

            let response_message = CreateInteractionResponseMessage::new().embed(embed);
            CreateInteractionResponse::Message(response_message)
        }
    };

    let send_response_payload = interaction_data.create_response(&ctx.http, return_response);
    let _ = send_response_payload.await;
}

