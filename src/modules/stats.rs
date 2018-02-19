use StatsLock;
use utils;

use serenity::CACHE;
use serenity::model::permissions::Permissions;
use serenity::utils::{Colour, with_cache};
use serenity::http;
use time;
use sys_info;

use std::env;

// show information about tofubot itself
command!(botinfo(ctx, msg) {
    let app = http::get_current_application_info().expect("No bot info");

    // Here are some constants like links
    let icon = "https://i.imgur.com/haeQiIJ.png".to_string();
    let homepage = "http://noxim.xyz/tofubot";
    let trello = "https://trello.com/b/EkzYe3mO";
    let invite = with_cache(|c| c.user.invite_url(Permissions::empty()))
        .expect("No invite?");

    let stats = {
        let data = ctx.data.lock();
        data.get::<StatsLock>().expect("No stats?").clone()
    };
    let load = sys_info::loadavg().expect("No load?");

    match msg.channel_id.send_message(|m| m.embed(|e| e
        .title("Information about TofuBot")
        .color(Colour::fooyoo())
        .thumbnail(&app.icon.unwrap_or(icon))
        .description("TofuBot is a general purpose _feature creep_ discord bot \
        with wide range of functions and tools for keeping your server \
        exciting!")
        .field("Owner", format!("
            Name: **{}#{}**\n\
            Snow: **{}**",
            app.owner.name, app.owner.discriminator, app.owner.id), true)
        .field("Links", format!("
            [Invite]({})\n\
            [Homepage]({})\n\
            [Trello]({})\n",
            invite, homepage, trello), true)
        .field("Instance", format!("
            Build: **{}**\n\
            Uptime: **{}**\n\
            Git: **{}**",
            env::var("CARGO_PKG_VERSION").expect("Cargo what?"),
            utils::fmt_difference(time::now_utc() - stats.start_utc),
            env!("VERSION")), true)
        .field("System", format!("\
            OS: **{}**\n\
            Uptime: **{}**\n\
            Load: **{}/{}/{}**\n\
            ",
            sys_info::os_release().expect("No OS?"),
            utils::fmt_difference(time::Duration::seconds(
                sys_info::boottime().expect("No boot time").tv_sec)),
            load.one, load.five, load.fifteen
        ), true)
        .field("Numbers", format!("
            Guilds: **{}**\n\
            Users: **{}**\n\
            Msg count: **{}**",
            stats.guilds, stats.users, stats.messages), true)
    )) {
        Err(why) => error!("MSG failed: {}", why),
        _ => {},
    }
});