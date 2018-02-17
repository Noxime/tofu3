use mongo;
use dog;

use time;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::user::User;
use serenity::model::guild::Member;
use serenity::client::Context;
use serenity::utils::Colour;

// our convinence method for loading the logging channel
fn log(ctx: &Context, id: u64) -> Option<ChannelId> {
    let data = ctx.data.lock();
    let db = data.get::<mongo::Mongo>().expect("No DB?");
    mongo::get_config(db, GuildId(id)).log()
}

// when user gets banned in a guild
// TODO: Serenity does not give us ban reason & banner, so its missing info
pub fn banned(ctx: &Context, id: &GuildId, user: &User) {
    dog::incr("users.banned", // log nicely
        vec![format!("guild:{}", id), format!("user:{}", user.id)]); 

    if let Some(c) = log(ctx, id.0) {
        match c.send_message(|m| m.embed(|e| e
            .title("User banned")
            .description(format!("Name: **{}#{}**\nSnow: **{}**", 
                user.name, user.discriminator, 
                user.id))
            .color(Colour::fooyoo())
            .footer(|f| f.text(time::now_utc().rfc3339()))
        )) {
            Err(why) => error!("MSG failed: {}", why),
            _ => {},
        }
    }
}

// when user gets unbanned in a guild
pub fn unbanned(ctx: &Context, id: &GuildId, user: &User) {
    dog::incr("users.unbanned", // log nicely
        vec![format!("guild:{}", id), format!("user:{}", user.id)]); 

    if let Some(c) = log(ctx, id.0) {
        match c.send_message(|m| m.embed(|e| e
            .title("User unbanned")
            .description(format!("Name: **{}#{}**\nSnow: **{}**", 
                user.name, user.discriminator, 
                user.id))
            .color(Colour::fooyoo())
            .footer(|f| f.text(time::now_utc().rfc3339()))
        )) {
            Err(why) => error!("MSG failed: {}", why),
            _ => {},
        }
    }
}

// user just joined this cord
pub fn user_join(ctx: &Context, id: &GuildId, member: &Member) {
    let user = member.user.read();
    dog::incr("users.joins", vec![
        format!("guild:{}", id), format!("user:{}", user.id)]);

    if let Some(c) = log(ctx, id.0) {
        match c.send_message(|m| m.embed(|e| e
            .title("User joined")
            .description(format!("Name: **{}#{}**\nSnow: **{}**", 
                user.name, user.discriminator, 
                user.id))
            .color(Colour::fooyoo())
            .footer(|f| f.text(time::now_utc().rfc3339()))
        )) {
            Err(why) => error!("MSG failed: {}", why),
            _ => {},
        }
    }
}

// user just left this cord
pub fn user_leave(ctx: &Context, id: &GuildId, user: &User) {
    dog::incr("users.leaves", vec![
        format!("guild:{}", id), format!("user:{}", user.id)]);

    if let Some(c) = log(ctx, id.0) {
        match c.send_message(|m| m.embed(|e| e
            .title("User left")
            .description(format!("Name: **{}#{}**\nSnow: **{}**", 
                user.name, user.discriminator, 
                user.id))
            .color(Colour::fooyoo())
            .footer(|f| f.text(time::now_utc().rfc3339()))
        )) {
            Err(why) => error!("MSG failed: {}", why),
            _ => {},
        }
    }
}