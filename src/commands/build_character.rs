use std::{any::Any, borrow::BorrowMut};

use serenity::{
    all::{ActionRow, ActionRowComponent, CreateActionRow, CreateInputText, CreateModal, CreateSelectMenu, CreateSelectMenuOption, InputText, InputTextStyle, ModalInteraction}, builder::{
        CreateCommand, CreateEmbed,
        CreateInteractionResponse,
        CreateInteractionResponseMessage,
    }, client::Context, model::application::CommandInteraction
};
use crate::{sql_scripts::characters, utils::{
    create_log_message, DatabaseConnectionContainer, EmbedColours, LogLevel
}};



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
pub async fn handle_modal( interaction_data: &ModalInteraction, ctx: &Context ) {
    
    // We don't have to worry about out-of-bounds here, as we know the inbound modal will have 3
    // components, that being the character's name, species, and backstory
    let character_data = {
        let mut data = vec![];

        for item in interaction_data.data.components.iter() {
            let result = match item.components[0].clone() {
                ActionRowComponent::InputText(data) => data.value.clone(),
                _ => return
            };

            if result.is_none() {
                return
            }

            data.push( result.expect("None case already handled") )
        }
        ( data[0].clone(), data[1].clone(), data[2].clone() )
    };

    println!("{:#?}", character_data)

}

