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
    sql_scripts::{self, characters},
    utils::{
        self, create_log_message, EmbedColours, LogLevel
    }
};

pub fn build() -> CreateCommand {
    CreateCommand::new("dump_cache")
        .description("Debug command to dump cache data")
}

pub async fn run( interaction_data: &CommandInteraction, ctx: &Context ) -> Option<CreateInteractionResponse> {
    
    let response = 'response_data: {
        let mut response_builder = String::new();
        let data_read = ctx.data.read().await;

        let character_map = match data_read.get::<utils::DatabaseCharactersCache>() {
            None => break 'response_data "Missing key in map".to_owned(),
            Some(data) => data
        };

        let map = match character_map.lock() {
            Ok(data) => data,
            Err(_) => break 'response_data String::from("Poisoned Lock")
        };
        
        for key in map.keys() {
            
            response_builder.push_str(format!("\nUser ID: {key}\nData:\n").as_str());
            let characters_data = map.get(key);
            for character in characters_data.unwrap() {
                response_builder.push_str(
                    format!("\t{:?}\n", character).as_str()
                );
            }
        }

        if response_builder.is_empty() {
            response_builder = "No data".to_owned()
        }
        response_builder
    };

    let response_payload = CreateInteractionResponse::Message(
       CreateInteractionResponseMessage::new().content(response)
    );

    let _ = interaction_data.create_response( &ctx.http, response_payload ).await;

    None
}
