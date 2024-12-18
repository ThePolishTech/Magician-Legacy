use serenity::{
    client::Context,
    builder::CreateCommand,
    model::application::CommandInteraction
};

pub fn build() -> CreateCommand {
    CreateCommand::new("register")
        .description("Add your profile to the database")
}
pub fn run( command_data: &CommandInteraction, ctx: &Context ) {
    println!("{:#?}", command_data.user.id.get() )
}

