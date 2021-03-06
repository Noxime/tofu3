use mongo;
use dog;

use time;
use serenity::model::id::{ChannelId, GuildId, MessageId};
use serenity::model::user::User;
use serenity::model::guild::Member;
use serenity::model::event::MessageUpdateEvent;
use serenity::client::Context;
use serenity::utils::Colour;
use serenity::CACHE;

// our convinence method for loading the logging channel
fn log(ctx: &Context, id: u64) -> Option<ChannelId> {
    let data = ctx.data.lock();
    let db = unopt!(data.get::<mongo::Mongo>(), "log helper no mongo", None);
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

// user removed a message. Unfortunately the function doesn't give the original
// message content, so we have to save them in our own database. This in turn
// means we might not have all messages aka we can't show what exactly was dletd
pub fn message_delete(ctx: &Context, id: &ChannelId, msg: &MessageId) {
    dog::incr("messages.deletes", vec![]);
    debug!("Message {} deleted in {}", msg, id);
    
    let log = match log(ctx, CACHE.read().channels.get(id)
        .map(|c| c.read().guild_id.0).unwrap_or(0)) {
        Some(v) => v,
        None => { return; }
    };

    let s = if let Some(old) = {
        let data = ctx.data.lock();
        let db = unopt!(data.get::<mongo::Mongo>(), "log del no mongo");
        mongo::get_message(db, *msg)
    } {
        // Hey we have the message in our database, show it
        let user = old.user().get();
        let username = match user {
            Ok(ref u) => u.name.clone(),
            Err(_) => "<unknown>".to_string(),
        };
        let discr = match user {
            Ok(ref u) => u.discriminator.to_string(),
            Err(_) => "<unknown>".to_string(),
        };
        let user_id = user.map(|u| u.id.to_string())
            .unwrap_or("<unknown>".to_string());
        let channel_name = id.get().map(|c| c.guild()
            .map(|c| c.read().name.clone())
            .unwrap_or("<unknown>".to_string()))
            .unwrap_or("<unknown>".to_string());

        log.send_message(|m| m.embed(|e| e
            .title("Message deleted")
            .description(old.content)
            .field("User", format!("Name: **{}#{}**\nSnow: **{}**",
                username, discr, user_id), true)
            .field("Message", format!("Snow: **{}**\nChannel: **{}**",
                id, channel_name), true)
            .color(Colour::fooyoo())
            .footer(|f| f.text(time::now_utc().rfc3339()))
        ))


    } else { // oof we didnt have that message in our database
        let channel_name = id.get().map(|c| c.guild()
            .map(|c| c.read().name.clone())
            .unwrap_or("<unknown>".to_string()))
            .unwrap_or("<unknown>".to_string());

        log.send_message(|m| m.embed(|e| e
            .title("Message deleted")
            .description("A message was deleted but unfortunately TofuBot does \
            not know what it was. Sorry about that!")
            .field("Info", format!("Snow: **{}**\nChannel: **{}**",
                msg, channel_name), true)
            .color(Colour::fooyoo())
            .footer(|f| f.text(time::now_utc().rfc3339()))
        ))
    };

    // this is final check we actually delivered the message
    match s {
        Ok(_) => {},
        Err(why) => error!("MSG failed: {}", why)
    }
}

// user edited a message, show the old and new text
pub fn message_edit(ctx: &Context, msg: &MessageUpdateEvent) {
    dog::incr("messages.edits", vec![]);
    debug!("Message {} edited", msg.id);

    let log = match log(ctx, CACHE.read().channels.get(&msg.channel_id)
        .map(|c| c.read().guild_id.0).unwrap_or(0)) {
        Some(v) => v,
        None => { return; }
    };

    if let Some(new) = msg.content.clone() {
        let s = if let Some(old) = {
            let data = ctx.data.lock();
            let db = unopt!(data.get::<mongo::Mongo>(), "log edit no mongo");
            mongo::get_message(db, msg.id)
        } {
            // Hey we have the message in our database, show it
            let user = old.user().get();
            let username = match user {
                Ok(ref u) => u.name.clone(),
                Err(_) => "<unknown>".to_string(),
            };
            let discr = match user {
                Ok(ref u) => u.discriminator.to_string(),
                Err(_) => "<unknown>".to_string(),
            };
            let user_id = user.map(|u| u.id.to_string())
                .unwrap_or("<unknown>".to_string());
            let channel_name = msg.channel_id.get().map(|c| c.guild()
                .map(|c| c.read().name.clone())
                .unwrap_or("<unknown>".to_string()))
                .unwrap_or("<unknown>".to_string());

            log.send_message(|m| m.embed(|e| e
                .title("Message edited")
                .description(format!("**Old content**\n{}\n**New content**\n{}",
                old.content, new))
                .field("User", format!("Name: **{}#{}**\nSnow: **{}**",
                    username, discr, user_id), true)
                .field("Message", format!("Snow: **{}**\nChannel: **{}**",
                    msg.channel_id, channel_name), true)
                .color(Colour::fooyoo())
                .footer(|f| f.text(time::now_utc().rfc3339()))
            ))


        } else { // oof we didnt have that message in our database
            let channel_name = msg.channel_id.get().map(|c| c.guild()
                .map(|c| c.read().name.clone())
                .unwrap_or("<unknown>".to_string()))
                .unwrap_or("<unknown>".to_string());

            log.send_message(|m| m.embed(|e| e
                .title("Message edited")
                .description(format!(
                "A message was edited but unfortunately TofuBot does not know \
                what it originally was. Sorry about that!\n**New content**\n{}",
                new))
                .field("Info", format!("Snow: **{}**\nChannel: **{}**",
                    msg.id, channel_name), true)
                .color(Colour::fooyoo())
                .footer(|f| f.text(time::now_utc().rfc3339()))
            ))
        };

        // this is final check we actually delivered the message
        match s {
            Ok(_) => {},
            Err(why) => error!("MSG failed: {}", why)
        }
    }
}