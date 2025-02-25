// xxxxxxxxxxxxxxxxx //
// --== CRATES == -- //
// xxxxxxxxxxxxxxxxx //
use std::env;

use serenity::{
    model::gateway::GatewayIntents,
    Client
};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

mod sql_scripts;
mod event_handler;
mod commands;
mod utils;


// xxxxxxxxxxxxxx //
// --== MAIN ==-- //
// xxxxxxxxxxxxxx //
#[tokio::main]
async fn main() {

    println!( "{}", utils::TITLE );

    let bot_client: Result< serenity::Client, i32 > = 'main: {

        // --== LOAD/CREATE DATABASE ==-- //

            print!("Opening Connection to Database...");
            let sqlite_connection_options = SqliteConnectOptions::new()
                .filename("kermmaw_db")
                .create_if_missing(true);

            let database_connection = SqlitePool::connect_with(sqlite_connection_options).await;

            
            let sqlx_connection = match database_connection {
                Ok(conn) => {
                    println!("Ok");
                    conn
                },
                Err(why) => {
                    println!("Error: {why}");
                    break 'main Err( 1 );
                }
            };
        // ==--

        // --== RUN MIGRATION INIT SCRIPT ==-- //

            print!("Running Table Creation Script...");
            let migration = sqlx::migrate!("./migrations")
                .run(&sqlx_connection);

            match migration.await {
                Ok(()) => {
                    println!("Ok")
                },
                Err(why) => {
                    println!("Error: {why}");
                    break 'main Err( 1 );
                }
            };
        // ==--

        // --== READ BOT TOKEN FROM ENVIROMENT ==-- //

            print!("Reading Bot Token From Enviromental Variable..."); 
            let bot_token = match env::var("BOT_TOKEN") {
                Ok(token) => {
                    println!("Ok");
                    token
                },
                Err(_) => {
                    println!("Error: Missing Token in Enviroment");
                    break 'main Err( 1 );
                }
            };
        // ==--

        // --== SETUP CONNECTION TO GATEWAY ==-- //   
            
            let gateway_intents = GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::MESSAGE_CONTENT
                | GatewayIntents::GUILD_MESSAGE_REACTIONS;

            print!("Building Client...");

            let client = event_handler::DiscordBot {
                database_connection: sqlx_connection
            };

            match Client::builder( bot_token, gateway_intents ).event_handler(client).await {
                Ok(client_builder) => {
                    println!("Ok");
                    break 'main Ok( client_builder )
                },
                Err(why) => {
                    println!("Error: {}", why);
                    break 'main Err( 1 );
                }
            }
        // ==--
    };

    // --== START CLIENT ==-- //
        
        print!("Starting Client...");
        match bot_client {
            Ok( mut client ) => {
                println!("Ok\n");

                // Start the client
                let _ = client.start().await;
            },
            Err(code) => {
                println!("Error: Code {}\n", code);
            }
        }
    // ==--
}

