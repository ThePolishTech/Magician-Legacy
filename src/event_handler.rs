use crate::utils::{
    create_log_message,
    LogLevel,
    EmbedColours
};

use serenity::{
     all::Interaction,
     async_trait,
     builder::{
        CreateEmbed, CreateMessage
     },
     client::{
        Context, EventHandler
     },
     model::{
        application::Command, gateway::Ready, id::ChannelId, Timestamp
     }
};

use crate::commands;


pub struct DiscordBot {
    pub database_connection: sqlx::SqlitePool
}

#[async_trait]
impl EventHandler for DiscordBot {
    
    async fn ready( &self, ctx: Context, _ready: Ready ) {

        // Notify terminal that the bot has connected to gateway
        println!( "{}",
            create_log_message( "Connected to Gateway; Bot Online", LogLevel::Info )
        );


        // --== CREATE WAKEUP MESSAGE ==-- //
            
            // To make the wakeup message pretty, we'll use an embed.
            // And so, we're gonna need a colour and timestamp for it
            // and also the channel to send it to
            let ( embed_colour, embed_timestamp, wakeup_channel ) = (
                EmbedColours::INFO,  // A nice ocean blue
                Timestamp::now(),
                ChannelId::from( 1314610244223238317 )  // For now I'm just hard coding it in
            );

            // Next up, we need to construct the embed that we will send
            // and put it into a message
            let embed = CreateEmbed::new()
                .title("Magician Online")
                .colour(embed_colour)
                .timestamp(embed_timestamp);

            let wakeup_message = CreateMessage::new().embed(embed);
        // ==--
        
        // --== SEND WAKEUP MESSAGE ==-- //
            
            // This is our final step, send the message. If that fails just report it
            if wakeup_channel.send_message( &ctx.http, wakeup_message ).await.is_err() {
                
                println!( "{}",
                    create_log_message( "Failed to send Wakeup Message", LogLevel::Warning )
                );
            }
        // ==--

        // --== REGISTER SLASH COMMANDS ==-- //
        
            let slash_commands = vec![
                commands::register::build(),
                commands::deregister::build(),
                commands::build_character::build(),
                commands::tmp::build(),
                commands::dump_cache::build()
            ];

            if let Err( why ) = Command::set_global_commands( &ctx.http, slash_commands ).await {
                println!( "{}",
                    create_log_message( format!("Failed to register slash commands:\n\t{why}"), LogLevel::Fatal )
                )
            }
        // ==--
    }

    #[allow(clippy::single_match)]
    async fn interaction_create( &self, ctx: Context, interaction_data: Interaction ) {
        // Here we see *what* kind of interaction we recived. Based upon that we de what we can and
        // can't do.
        match interaction_data {
                
            // --== COMMAND INTERACTIONS ==-- //
                
                Interaction::Command( inbound_command_data ) => {

                    // Depending on which command was called, execute the right code. We expect a
                    // return type of Option<CreateInteractionRespone> after .await'ing. Theoretically
                    // we could use a Result<T, E> enum to log errors, but I've decided that it would
                    // be better to handle that inside of the function to not force a structure upon
                    // ourseleves. Besdies, we could respond to the interaction inside of the function,
                    // therefore not doing anything more here would be optimal.
                    let response_opt = match inbound_command_data.data.name.clone().as_str() {

                        "register" => commands::register::run(
                                &inbound_command_data, &ctx, self
                        ).await,

                        "deregister" => commands::deregister::run(
                                &inbound_command_data, &ctx, self
                        ).await,

                        "tmp" => commands::tmp::run(
                                &inbound_command_data, &ctx
                        ).await,

                        "build_character" => commands::build_character::run(
                                &inbound_command_data, &ctx
                        ).await,

                        "dump_cache" => commands::dump_cache::run(
                            &inbound_command_data, &ctx
                        ).await,

                        _ => { None }
                    };

                    if let Some( response ) = response_opt {
                        // If there is a response message given to us, atempt to send it as a response
                        // to the interaction. If that fails, log it.
                        // Although not being a fatal error, not responding to an interaction is still
                        // unwanted as the user may be missing out on important information
                        if let Err( why ) = inbound_command_data.create_response( &ctx.http, response ).await {
                            println!( "{}", create_log_message(
                                    format!("Failed to send response to command interaction:\n\t{why}"),
                                    LogLevel::Error
                            ))
                        }
                    }

                    
                },
            // ==--

            // --== AUTOCOMPLETE INTERACTIONS ==-- //
            
                Interaction::Autocomplete( inbound_autocomplete_data ) => {
                    let interaction_name = inbound_autocomplete_data.data.name.clone();

                    match interaction_name.as_str() {
                        "tmp" => { commands::tmp::handle_autocomplete( &inbound_autocomplete_data, &ctx ).await },
                        _ => {
                            println!( "{}", create_log_message(
                                    format!("Recived unknown autocomplete interaction. Name: {interaction_name}"),
                                    LogLevel::Warning
                            ))
                        }
                    }
                },
            // ==--

            // --== MODAL INTERACTIONS ==-- //
                
                Interaction::Modal( inbound_modal_data ) => {
                    let modal_id: String = inbound_modal_data.data.custom_id.clone();
                    let modal_id_components = modal_id
                        .split(':')
                        .collect::<Vec<&str>>();

                    // If the ID of the modal is mangled, i.e. doesn't contain a ':' character to
                    // seperate the name of the interaction that called it, we will print the
                    // mangled ID to the terminal in an error message and then break as we can't do
                    // anything more with it unfortunetly
                    let modal_name = modal_id_components.first();
                    if modal_name.is_none() {
                        println!( "{}", create_log_message(
                                format!("Recived modal interaction with mangled id: {}", modal_id ),
                                LogLevel::Warning
                        ));
                        // If the ID is mangled, we can't do anything with it so we will just break
                        // early
                        return;
                    }
                    let modal_name = modal_name
                        .expect("Handled None case already")
                        .to_owned();  // Looks better if we later match on a new String than a
                                      // &&str

                    match modal_name {
                        "build_character" => commands::build_character::handle_modal(
                                &inbound_modal_data, &ctx, self
                        ).await,
                        _ => {
                            println!( "{}", create_log_message(
                                format!("Recived unknown modal interaction. Name: {}", modal_name ),
                                LogLevel::Warning
                            ));

                        }
                    };
                },
            // ==--

            _ => {
                println!( "{}", create_log_message(
                        format!("Recived unknown interaction: {:?}", interaction_data.kind() ),
                        LogLevel::Warning
                ))
            }
        }
    }

}

