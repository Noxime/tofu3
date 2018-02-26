use super::super::mongo;
use serenity::utils::Colour;
use serenity::model::id::UserId;

// turn our score to a more human friendly level with a fancy algorithm of 
// sqrt(score) / 3, the more difficult part is calculating the progress between
// levels i guess
fn calculate_level(score: i64) -> (i64, f64) {
    if score < 1 { return (0, 0f64) }
    // calculate current level and the level one above us
    let floor_rank = ((score as f64).sqrt() / 3f64).floor() as i64;
    let ceil_rank = floor_rank + 1;
    // the un-leveled scores our progress should be between
    let floor_score = floor_rank * floor_rank * 9; // (x*3)^2
    let ceil_score = ceil_rank * ceil_rank * 9;
    // calculate how far our score is between our floor and ceil scores
    let progress = (score - floor_score) as f64 / 
        (ceil_score - floor_score) as f64;
    (floor_rank, progress)
}

// access the database and print the score for this current server
command!(rank(ctx, msg, args) {
    let _ = msg.channel_id.broadcast_typing();
    let id = args.single::<UserId>().unwrap_or(msg.author.id);

    let user = {
        let data = ctx.data.lock();
        let db = unopt_cmd!(data.get::<mongo::Mongo>(), "*rank no mongo");
        mongo::get_user(db, id)
    };

    let (level, progress) = calculate_level(user.get_score(
        unopt_cmd!(msg.guild_id(), "*rank no gid")));

    let _ = msg.channel_id.send_message(|m| m.embed(|e| e
        .color(Colour::fooyoo())
        .title(&format!("Rank for {}", 
            id.get().map(|v| v.name).unwrap_or("<unknown>".to_string())))
        .description(format!("\
            Current level: **{}**\n\
            Progress: **{:.2}%**",
            level, progress * 100f64))
    ));
});

// find top 10 users for this server and post them in a list
command!(leaderboard(ctx, msg) {
    let id = unopt_cmd!(msg.guild_id(), "*lb no gid");
    let _ = msg.channel_id.broadcast_typing();

    // load users from mongo
    let users = {
        let data = ctx.data.lock();
        let db = unopt_cmd!(data.get::<mongo::Mongo>(), "*lb no mongo");
        mongo::get_top_users(db, id, 10)
    };

    // iterate over users and convert to viewable embed fields
    let mut i = 0;
    let fields: Vec<(String, String, bool)> = users.iter().map(|v| (
        format!("{}: {}",
            { i += 1; i },
            UserId(v.user_id as u64).get().map(|v| v.name)
                .unwrap_or("<none>".to_string())
        ),
        { 
            let (l, p) = calculate_level(v.get_score(id));
            format!("Level: **{}**\nProgress: **{:.2}%**", l, p * 100f64) 
        },
        false
    )).collect();

    // send the embed with the fields constructed earlier
    let _ = msg.channel_id.send_message(|m| m.embed(|e| e
        .color(Colour::fooyoo())
        .title("Top 10 users")
        .fields(fields)
    ));
});