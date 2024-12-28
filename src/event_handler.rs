use crate::utils::{
    create_log_message,
    LogLevel,
    EmbedColours
};

use serenity::{
     all::{Interaction}, async_trait, builder::{
        CreateEmbed, CreateMessage
    }, client::{
        Context, EventHandler
    }, model::{
        application::Command, gateway::Ready, id::ChannelId, Timestamp
    }
};

use crate::commands;


pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    
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
                commands::deregister::build()
            ];

            if let Err( why ) = Command::set_global_commands( &ctx.http, slash_commands ).await {
                println!( "{}",
                    create_log_message( format!("Failed to register slash commands:\n\t{why}").as_str(), LogLevel::Fatal )
                )
            }
        // ==--
    }

    #[allow(clippy::single_match)]
    async fn interaction_create( &self, ctx: Context, interaction_data: Interaction ) {
        // Here we see *what* kind of interaction we recived. Based upon that we de what we can and
        // can't do.
        match interaction_data {
            Interaction::Command( inbound_command_data ) => {

                // Depending on which command was called, execute the right code. We expect a
                // return type of Option<CreateInteractionRespone> after .await'ing. Theoretically
                // we could use a Result<T, E> enum to log errors, but I've decided that it would
                // be better to handle that inside of the function to not force a structure upon
                // ourseleves. Besdies, we could respond to the interaction inside of the function,
                // therefore not doing anything more here would be optimal.
                let response_opt = match inbound_command_data.data.name.clone().as_str() {
                    "register" => { commands::register::run( &inbound_command_data, &ctx ).await },
                    "deregister" => { commands::deregister::run( &inbound_command_data, &ctx).await },
                    _ => { None }
                };

                if let Some( response ) = response_opt {
                    // If there is a response message given to us, atempt to send it as a response
                    // to the interaction. If that fails, log it.
                    // Although not being a fatal error, not responding to an interaction is still
                    // unwanted as the user may be missing out on important information
                    if let Err( why ) = inbound_command_data.create_response( &ctx.http, response ).await {
                        println!( "{}", create_log_message(
                                format!("Failed to send response to command interaction:\n\t{why}").as_str(), LogLevel::Error
                        ))
                    }
                }

                
            },
            // Other Stuff will go here. Autocomplete is something I plan to eventually implement 
            _ => {}
        }
    }

}

// some comment
// for showcase
