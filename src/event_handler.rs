use crate::utils::{
    create_log_message,
    LogLevel
};

use serenity::{
    async_trait,
    builder::{
        CreateMessage, CreateEmbed
    },
    model::{
        Colour,
        Timestamp
    },
    client::{
        Context, EventHandler
    },
    model::{
        gateway::Ready,
        id::ChannelId
    }
};



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
                Colour::from_rgb( 0, 127, 255 ),  // A nice ocean blue
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
    
    }
    
}

