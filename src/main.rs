#[macro_use]
extern crate serenity;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate bson;
extern crate kankyo;
extern crate pretty_env_logger;
extern crate mongodb;
extern crate typemap;

use serenity::prelude::{Client as DiscordClient, EventHandler};
use serenity::framework::StandardFramework;

mod mongo;
mod modules;

struct DiscordHandler;

impl EventHandler for DiscordHandler {

}

fn main() {
    // load environment variables
    if let Err(why) = kankyo::load() {
        error!("Could not load .env file: {:#?}", why);
    }
    // initialize a pretty logger
    pretty_env_logger::init();

    // note unwrap is safe here, because these are always set by cargo for us
    info!("Starting {} v{}", 
        kankyo::key("CARGO_PKG_NAME").unwrap(), 
        kankyo::key("CARGO_PKG_VERSION").unwrap());

    // connect to the discord endpoint
    let token = kankyo::key("TOFU_DISCORD").expect("TOFU_DISCORD missing!");
    let mut client = DiscordClient::new(&token, DiscordHandler).unwrap();

    // set up client data
    {
        let mut data = client.data.lock();
        data.insert::<mongo::Mongo>(mongo::connect());
    }

    // configure our discord framework
    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .prefix("*")
            .dynamic_prefix(|ctx, msg| {
                // load a prefix from our mongodb
                let data = ctx.data.lock();
                let db = data.get::<mongo::Mongo>().unwrap();
                let config = mongo::get_config(db, msg.guild_id().unwrap());
                config.user.prefix
            }))
        .command("stats", |c| c.cmd(modules::stats::stats))
        .command("rank", |c| c.cmd(modules::ranks::rank))
    );

    if let Err(why) = client.start() {
        eprintln!("Could not start serenity: {:?}", why);
    }
}