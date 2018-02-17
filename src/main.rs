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
extern crate time;
extern crate toml;
extern crate reqwest;
extern crate urbandictionary;
extern crate dogstatsd;
#[macro_use]
extern crate lazy_static;

use serenity::prelude::{Client as DiscordClient, EventHandler, Context};
use serenity::framework::standard::{
    StandardFramework, HelpBehaviour, help_commands, Args, CommandOptions, 
    CommandError,
};
use serenity::utils::Colour;
use serenity::model::channel::Message;
use serenity::model::user::User;
use serenity::model::guild::Member;
use serenity::model::id::{UserId, GuildId, ChannelId, MessageId};
use typemap::Key;

use std::collections::HashMap;

mod mongo;
mod modules;
mod dog;

// this makes sure we don't give too much score too often. The hashmap contains
// a user id and then the last unix timestamp they got some score. So we wait 2
// minutes before we can give more again
struct RankLock;
impl Key for RankLock {
    type Value = HashMap<UserId, u64>;
}

struct DiscordHandler;

impl EventHandler for DiscordHandler {

    // logging
    fn guild_ban_addition(&self, ctx: Context, id: GuildId, user: User) {
        modules::logging::banned(&ctx, &id, &user);
    }
    fn guild_ban_removal(&self, ctx: Context, id: GuildId, user: User) {
        modules::logging::unbanned(&ctx, &id, &user);
    }
    fn guild_member_addition(&self, ctx: Context, id: GuildId, member: Member) {
        modules::logging::user_join(&ctx, &id, &member);
    }
    fn guild_member_removal(&self, ctx: Context, id: GuildId, user: User,
    _: Option<Member>) {
        modules::logging::user_leave(&ctx, &id, &user);
    }
    fn message_delete(&self, ctx: Context, id: ChannelId, msg: MessageId) {
        modules::logging::message_delete(&ctx, &id, &msg);
    }

    fn message(&self, ctx: Context, msg: Message) {
        // fancy graphs on datadog
        if msg.is_own() {
            dog::incr("messages.sent", vec![]);
        } else {
            dog::incr("messages.received", vec![]);
        }

        // save the message in mongo
        {
            let data = ctx.data.lock();
            let db = data.get::<mongo::Mongo>().expect("No DB?");
            // wait wtf
            mongo::set_message(db, &mongo::MongoMessage::from(msg.clone()));
        }

        // don't activate for bots
        if msg.author.bot {
            return;
        }


        // for our ranks, we need to add the score from this message to the db
        let incr = { // first check if even should give user score (2min passed)
            let mut data = ctx.data.lock(); // we want to release this asap
            let lock = data.get_mut::<RankLock>().unwrap();
            let last = lock.entry(msg.author.id).or_insert(0);
            let now = time::now().to_timespec().sec;
            // check 2 minutes passed
            if *last + 120 < now as u64 {
                *last = now as u64;
                true
            } else {
                false
            }
        };

        if incr {// increase the authors rank by 5
            let data = ctx.data.lock(); // we want to release this asap
            let db = data.get::<mongo::Mongo>().unwrap(); // mongo access
            let mut user = mongo::get_user(db, msg.author.id);
            let score = user.get_score(msg.guild_id().unwrap()) + 5; //incr xp 5
            user.set_score(msg.guild_id().unwrap(), score);
            mongo::set_user(db, &user);
        }

        if let Some(cmds) = { // activate for commandeeros
            let data = ctx.data.lock();
            let db = data.get::<mongo::Mongo>().unwrap();
            mongo::get_config(db, msg.guild_id().unwrap()).user.commands
        } { // we have commands
            if let Some(v) = cmds.get(&msg.content.to_lowercase()) {
                match msg.channel_id.send_message(|m| m.content(v)) {
                    Err(why) => error!("MSG failed: {}", why),
                    _ => {}
                }
            }
        }
    }
}

// make sure user is allowed to run these admin level commands
fn admin_check(ctx: &mut Context, msg: &Message, _: &mut Args, 
_: &CommandOptions) -> bool {

    let id = msg.guild_id().unwrap();
    // get admin roles
    let roles = {
        let data = ctx.data.lock();
        let db = data.get::<mongo::Mongo>().unwrap();
        mongo::get_config(db, id).staff()
    };
    // check if the author is the owner of this guild
    // OR check if the message author has any of the required staff roles
    id.get().unwrap().owner_id == msg.author.id
    || roles.iter().any(|r| msg.author.has_role(id, *r))
}

// run before any command
fn before(_: &mut Context, _: &Message) -> bool {
    dog::incr("commands.calls", vec![]);
    true
}

// after any command, to report internal errors
fn after(_: &mut Context, _: &Message, ret: &Result<(), CommandError>) {
    if ret.is_err() {
        dog::incr("commands.errors", vec![]);
        error!("Error in command: {:?}", ret);
    } else {
        dog::incr("commands.executes", vec![]);
    }
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
        data.insert::<RankLock>(HashMap::new());
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
        // a nice help command for our users
        .customised_help(help_commands::with_embeds, |c| c
            .individual_command_tip("Hello! Thanks for using TofuBot. If you \
            want to learn more about a specific command, just add the command \
            name after `help`")
            .command_not_found_text("Command {} could not be found, are you \
            sure you spelled it right")
            .suggestion_text("Are you looking for {}?")
            // hide commands that user can't call
            .lacking_permissions(HelpBehaviour::Hide)
            .lacking_role(HelpBehaviour::Nothing)
            .wrong_channel(HelpBehaviour::Strike)
            // colors are nice, at least for those who aren't blind
            .embed_success_colour(Colour::fooyoo())
            .embed_error_colour(Colour::red())
        )
        // misc
        .group("Miscellaneous", |c| c
            .command("stats", |c| c
                .cmd(modules::stats::stats)
                .desc("System information about TofuBot")
                .before(before)
                .after(after))
            .command("urban", |c| c
                .cmd(modules::urban::urban)
                .desc("Search urban dictionary for a word or a sentence.")
                .usage("<search term>")
                .example("bodge")
                .known_as("ub")
                .known_as("urbandictionary")
                .min_args(1)
                .before(before)
                .after(after)))
        // ranks
        .group("Ranking", |c| c
            .command("rank", |c| c
                .cmd(modules::ranks::rank)
                .bucket("ranking")
                .desc("Your current level and progress in this discord server. \
                    You can add a mention or a snowflake ID in the end to see \
                    someone else's rank.")
                .usage("[mention or snowflake]")
                .example("@noxim#6410")
                .max_args(1)
                .before(before)
                .after(after))
            .command("leaderboard", |c| c
                .cmd(modules::ranks::leaderboard)
                .bucket("ranking")
                .known_as("lb")
                .desc("See the top 10 users for this discord server. The \
                levels are calculated with `√x ÷ 3`, where x is your XP. 5 XP \
                is given for every 2 minutes of active chatting.")
                .max_args(0)
                .before(before)
                .after(after)))
        .group("Admin", |c| c
            .command("settings", |c| c
                .cmd(modules::settings::settings)
                .check(admin_check)
                .bucket("admin")
                .desc("Change settings for this discord server. You can call \
                this command without a file to see and download your current \
                configuration. You can set a new configuration by attaching \
                a file to this command. The configuration format is called \
                TOML and can be opened in programs such as Notepad++. See the \
                TofuBot webpage for extra help.")
                .usage("[file]")
                .max_args(1)
                .before(before)
                .after(after)))
        .group("Commands", |c| c
            .command("new", |c| c
                .cmd(modules::commands::new)
                .check(admin_check)
                .bucket("admin")
                .min_args(2)
                .example("!tunes https://youtu.be/XfR9iY5y94s")
                .usage("<name> <content>")
                .desc("Create a new custom command for this guild. Make sure \
                you remember to include the prefix you want to use for that \
                command, for example `*` or `!`.")
                .before(before)
                .after(after))
            .command("delete", |c| c
                .cmd(modules::commands::delete)
                .check(admin_check)
                .bucket("admin")
                .min_args(1)
                .example("!tunes")
                .usage("<name>")
                .desc("Remove a previousley created custom command. Make sure \
                you write the command name correctly.")
                .before(before)
                .after(after))
            .command("list", |c| c
                .cmd(modules::commands::list)
                .bucket("commands")
                .max_args(1)
                .example("2")
                .usage("<page>")
                .desc("Use this command to see all the custom commands for \
                this server. In case all the commands don't fit on the same \
                page, you can provide a page number.")
                .before(before)
                .after(after)))
    );

    if let Err(why) = client.start() {
        eprintln!("Could not start serenity: {:?}", why);
    }
}