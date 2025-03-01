use serenity::{
    all::{AutocompleteChoice, CommandOptionType, CreateAutocompleteResponse, CreateCommandOption, CreateInteractionResponse}, builder::{
        CreateCommand, CreateInteractionResponseMessage
    }, client::Context, model::application::{CommandInteraction, ResolvedValue}
};
use crate::utils::{
    create_log_message, DatabaseCharactersCache, LogLevel
};

pub fn build() -> CreateCommand {
    CreateCommand::new("tmp")
        .description("Testing some stuff")
        .add_option(CreateCommandOption::new(CommandOptionType::String, "test", "Autocomplete? Please?").set_autocomplete(true).required(true))
}
pub async fn run( interaction_data: &CommandInteraction, ctx: &Context ) -> Option<CreateInteractionResponse> {
    
    let option = match interaction_data.data.options()[0].value {
        ResolvedValue::String( data ) => data,
        _ => {
            // This will always be a string, therefor this branch is
            unreachable!()
        }
    };
    

    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content( option )
    );

    if let Err( why ) = interaction_data.create_response( &ctx.http, response).await {
        print!("{}", why);
    }

    None
}
pub async fn handle_autocomplete( interaction_data: &CommandInteraction, ctx: &Context ) {

    // We'll need the id of the calling user
    let invoking_user_id = interaction_data.user.id.get();

    let raw_responses: Vec<(u16, String)> = 'responses: {
        // We need to get the what the user typed into our autocomplete field.
        //
        // We know for a fact that the command has it, so we can just index directly into the option
        // and extract it's value.
        let user_query = match interaction_data.data.options()[0].value {
                                            // We want to be case, insesnitive
            ResolvedValue::Autocomplete { value, .. } => value.to_lowercase(),
            _ => break 'responses vec![]  // We know the type of value, but just in case we'll
                                          // break early with an empty vector
        };

        // Next we'll get a copy of a list of the characters that the characters own. In the form of
        // Vec<( character_id, character_name )>. We must do so in a code block as we'll be grabbing it
        // from a Mutex, to limit the potential of a panic causing a mutex poisoning. For that same
        // reason we'll be cloning the data. The performance penalty is conisdered acceptable.
        let user_characters = {
            let read_data = ctx.data.read().await;
            let character_map_mutex = read_data
                .get::<DatabaseCharactersCache>()
                .expect("Key must be inserted at startup in main.rs")
                .lock();

            let user_characters: Vec<(u16, String)> = match character_map_mutex {
                Ok( character_map ) => {
                    //
                    match character_map.get(&invoking_user_id) {
                        None => {
                            // If we get a None variant here, it means the user doesn't have any
                            // characters. And thus can return an empty list
                            break 'responses vec![]
                        },
                        Some( characters ) => {
                            // As stated before, we will be cloning the information in order to
                            // drop the lock on the mutex. This will allow us to minimise the
                            // possibility of poisoning it, with the performance overhead deamed
                            // acceptable
                            characters.clone()
                        }
                    }
                },
                Err(_) => {
                    // This can only occur if the lock is poisoned. Which it shouldn't be able to
                    // be. So for this very rare edge case we will just give the user an empty list
                    // of options as we can't send an error message. We'll also opt to not log it
                    // as the pure amount of autocomplete interactions would be enough to flood the
                    // console. Instead we'll allow future commands, which would log it to do so
                    // for us
                    break 'responses vec![]
                }
            };

            // Finally we return the new vector
            user_characters
        };

        // Next up on our agenda is filtering the previously gathered characters by our user's
        // query in order to return the ones that match the user's search.
            
        // First of all, we need an empty vec to which we will push
        let mut found_characters = vec![];

        // The data within user_characters is a tuple of (character_id, character_name).
        // Therefore we will index to get the name
        for character in user_characters.iter() {
            if character.1.to_lowercase().starts_with(&user_query) {
                found_characters.push(character.clone());
            }
        }


        // Next up we will package the responses into autocomplete choices. It will be done outside
        // of this block as we have early returns in this block (which return empty vectors) that
        // serve to manage unexpected states
        found_characters
    };

    let autocomplete_choices = {
        
        let mut choices = vec![];

        for ( character_id, character_name ) in raw_responses.iter() {
            choices.push(
                AutocompleteChoice::new(character_name, character_id.to_string())
            );
        }
        choices
    };

    if let Err(why) = interaction_data.create_response( &ctx.http,
        CreateInteractionResponse::Autocomplete(CreateAutocompleteResponse::new().set_choices(autocomplete_choices))
    ).await {
        println!("{}", create_log_message(
                format!("Failed to respond to autocomplete interaction_data: \n\t{why}"),
                LogLevel::Warning
        ))
    }
    
}
