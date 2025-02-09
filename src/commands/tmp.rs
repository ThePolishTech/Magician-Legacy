use std::any::Any;

use serenity::{
    all::{AutocompleteChoice, CommandOptionType, CreateAutocompleteResponse, CreateCommandOption, CreateInteractionResponse}, builder::{
        CreateAttachment, CreateCommand, CreateEmbed, CreateInteractionResponseMessage
    }, client::Context, model::application::CommandInteraction
};
use crate::utils::{
    create_log_message, LogLevel,
    DatabaseConnectionContainer,
    EmbedColours
};
use toml::Table;
use crate::sql_scripts::discord_users;

pub fn build() -> CreateCommand {
    CreateCommand::new("tmp")
        .description("Testing some stuff")
        .add_option(CreateCommandOption::new(CommandOptionType::String, "test", "Autocomplete? Please?").set_autocomplete(true))
}
pub async fn run( interaction_data: &CommandInteraction, ctx: &Context ) -> Option<CreateInteractionResponse> {
    
    let toml = r#"
    character_name = "Test"

    "#;


    let attachment = CreateAttachment::bytes("Hello World", "Test File.txt");
    

    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content("Test")
            .add_file(attachment)
    );

    if let Err( why ) = interaction_data.create_response( &ctx.http, response).await {
        print!("{}", why);
    }

    None
}
pub async fn handle_autocomplete( interaction_data: &CommandInteraction, ctx: &Context ) {
    
    let autocomplete_option = interaction_data.data.autocomplete().unwrap();

    println!("{:?}",autocomplete_option.value);

    let autocomplete_response = vec![
        AutocompleteChoice::new("A", "1"),
        AutocompleteChoice::new("B", "2"),
        AutocompleteChoice::new("C", "3")
    ];

    let response_payload = CreateInteractionResponse::Autocomplete(CreateAutocompleteResponse::new()
        .set_choices(autocomplete_response));

    let send_response_payload = interaction_data.create_response( &ctx.http, response_payload );

    if let Err( why ) = send_response_payload.await {
        println!("{}", create_log_message(
                format!("Failed to send autocomplete response in /tmp:\n\t{}",why).as_str(),
                LogLevel::Warning
        ))
    }
}
