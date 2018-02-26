use StatsLock;
use utils;

use serenity::model::permissions::Permissions;
use serenity::utils::{Colour, with_cache};
use serenity::http;
use time;
use sys_info;

// show information about tofubot itself
command!(botinfo(ctx, msg) {
    let app = unres_cmd!(http::get_current_application_info(),"botinfo no app");
    let app_id = app.id.clone();

    // Here are some constants like links
    let icon = "https://i.imgur.com/haeQiIJ.png".to_string();
    let homepage = "http://noxim.xyz/tofubot";
    let trello = "https://trello.com/b/EkzYe3mO";
    let invite = unres_cmd!(
        with_cache(|c| c.user.invite_url(Permissions::empty())),
        "botinfo no invite");

    let stats = {
        let data = ctx.data.lock();
        unopt_cmd!(data.get::<StatsLock>(), "botinfo no stats").clone()
    };
    let load = unres_cmd!(sys_info::loadavg(), "No load?");
    let os = unres_cmd!(sys_info::os_release(), "botinfo no os");
    let host_uptime = utils::fmt_difference(time::Duration::seconds(
        unres_cmd!(sys_info::boottime(), "No boot time").tv_sec));

    match msg.channel_id.send_message(|m| m.embed(|e| e
        .title("Information about TofuBot")
        .color(Colour::fooyoo())
        .thumbnail(&app.icon.map(|v| 
            format!("https://cdn.discordapp.com/app-icons/{}/{}.png", app_id,v))
            .unwrap_or(icon))
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
            env!("CARGO_PKG_VERSION"),
            utils::fmt_difference(time::now_utc() - stats.start_utc),
            env!("VERSION")), true)
        .field("System", format!("\
            OS: **{}**\n\
            Uptime: **{}**\n\
            Load: **{}/{}/{}**\n\
            ",
            os,
            host_uptime,
            load.one, load.five, load.fifteen
        ), true)
        .field("Numbers", format!("
            Guilds: **{}**\n\
            Users: **{}**\n\
            Msg count: **{}**",
            stats.guilds, stats.users, stats.messages), true)
            
    )) {
        Err(why) => error!("MSG failed: {:#?}", why),
        _ => {},
    }
    
});