use serenity::{
    all::CreateInteractionResponse, builder::{
        CreateCommand, CreateEmbed,
        CreateAttachment,
        CreateInteractionResponseMessage
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
