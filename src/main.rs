use core::panic;
// xxxxxxxxxxxxxxxxx //
// --== CRATES == -- //
// xxxxxxxxxxxxxxxxx //
use std::{
    collections::HashMap,
    env, sync::{Arc, Mutex}
};

use serenity::{
    model::gateway::GatewayIntents, Client
};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool, Row};
use utils::DatabaseCharactersCache;

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
                .filename("kerm-maw_db")
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

            print!("Setting up Client...");

            let client = event_handler::DiscordBot {
                database_connection: sqlx_connection
            };
            println!("Ok");
        // ==--
        
        // --== CREATE AND POPULATE CACHE ==--- //

            // Firstly, we grab all the characters that currently exist in the database
            print!("Syncing Cache to Database..." );
            let query_result = sqlx::query( sql_scripts::characters::SELECT_ALL_CHARACTER_IDS_AND_NAME )
                .fetch_all( &client.database_connection )
                .await;

            let characters_cache = match query_result {
                Ok(query_data) => {

                    let mut user_characters_map: HashMap<u64, Vec<(u16, String)>> = HashMap::new();
                    for entry in query_data.iter() {
                        let ( user_id, character_id, character_name ): (u64,u16,String) = (
                            entry.get(0),
                            entry.get(1),
                            entry.get(2)
                        );

                        // If user isn't in the hashmap, insert them with character data
                        // Else appened character data
                        match user_characters_map.get_mut(&user_id) {
                            None => {
                                user_characters_map.insert(
                                    user_id,
                                    vec![(character_id, character_name)]
                                );
                            },
                            Some(characters) => {
                                characters.push(
                                    ( character_id, character_name )
                                );
                            }
                        };
                        
                    }

                    // And finally we return it
                    println!("Ok");
                    user_characters_map
                },
                Err(why) => {
                    println!("Error: {}", why);
                    break 'main Err( 1 );
                }
            };
        // ==--

        // --== BUILD CLIENT ==-- // 

            print!("Building Client...");
            let bot_client = match Client::builder( bot_token, gateway_intents ).event_handler(client).await {
                Ok(client_builder) => {
                    println!("Ok");

                    // We insert the cache to allow it to be used in the future
                    {
                        let mut data_write = client_builder.data.write().await;
                        data_write.insert::<DatabaseCharactersCache>(
                            Arc::new(Mutex::new(  characters_cache  ))
                        );
                    }
                    client_builder
                },
                Err(why) => {
                    println!("Error: {}", why);
                    break 'main Err( 1 );
                }
            };
        // ==--
        
        Ok( bot_client )
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

