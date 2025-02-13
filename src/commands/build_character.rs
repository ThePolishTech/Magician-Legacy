use serenity::{
    all::{CreateActionRow, CreateInputText, CreateModal, InputTextStyle, ModalInteraction}, builder::{
        CreateCommand, CreateEmbed,
        CreateInteractionResponse,
        CreateInteractionResponseMessage,
    }, client::Context, model::application::CommandInteraction
};
use crate::utils::{
    create_log_message, LogLevel,
    DatabaseConnectionContainer,
    EmbedColours
};



pub fn build() -> CreateCommand {
    CreateCommand::new("build_character")
        .description("Build your character")
}

pub async fn run( interaction_data: &CommandInteraction, ctx: &Context ) -> Option<CreateInteractionResponse> {



    let modal_components = vec![
        CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Short, "Name", "name")),
        CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Paragraph, "description", "desc"))
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

pub async fn handle_modal( interaction_data: &ModalInteraction, ctx: &Context ) {
    println!("Modal ID {:?}\n{:#?}", interaction_data.data.custom_id.as_str(), interaction_data.data )
}

