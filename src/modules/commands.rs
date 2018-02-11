use serenity::framework::standard::CommandError;
use serenity::utils::Colour;
use mongo;

use std::collections::HashMap;

// register a new command in the database
command!(new(ctx, msg, args) {
    let _ = msg.channel_id.broadcast_typing();
    // name of our command
    let name = args.single::<String>()?;
    let content = args.full();
    if content.is_empty() {
        return Err(CommandError("No content".into()));
    }

    // check if db contains and if does, error out
    let mut cmds = {
        let data = ctx.data.lock();
        let db = data.get::<mongo::Mongo>().expect("No DB?");
        mongo::get_config(db, msg.guild_id().unwrap()).user.commands
            .unwrap_or(HashMap::new())
    };

    // check if there is already an item with this name
    if cmds.contains_key(&name.to_lowercase()) {
        match msg.channel_id.send_message(|m| m.embed(|e| e
            .color(Colour::red())
            .title("Command exists")
            .description(
                format!("The command you tried add ({}) already exists! \
                Choose a new name or remove the old one.", name)))) {
            Ok(_) => {},
            Err(why) => error!("MSG failed: {}", why)
        }
    } else {
        // Add the newly acquired command, and warn if somehow we overwrote
        if let Some(val) = cmds.insert(name.to_lowercase(), content.into()) {
            warn!("Accidentally rewrote a command {}: {}", name, val);
        }
    }

    { // write back the results
        let data = ctx.data.lock();
        let db = data.get::<mongo::Mongo>().expect("No DB?");
        let mut c = mongo::get_config(db, msg.guild_id().unwrap());
        c.user.commands = Some(cmds);
        mongo::set_config(db, &c);
    }

    // hey it worked, tell the user!
    match msg.channel_id.send_message(|m| m.embed(|e| e
        .color(Colour::fooyoo())
        .title("Command added")
        .description(
            format!("The command **\"{}\"** has been successfuly added.", 
            name)))) {
        Ok(_) => {},
        Err(why) => error!("MSG failed: {}", why)
    }
});

// remove a command from this server
command!(delete(ctx, msg, args) {
    let _ = msg.channel_id.broadcast_typing();
    let name = match args.single::<String>() {
        Ok(s) => s,
        Err(_) => {
            match msg.channel_id.send_message(|m| m.embed(|e| e
                .color(Colour::red())
                .title("Argument error")
                .description("Please provide the name of the command you want \
                to remove."))) {
                Ok(_) => {},
                Err(why) => error!("MSG failed: {}", why)
            }
            return Ok(());
        }
    };

    let mut cmds = {
        let data = ctx.data.lock();
        let db = data.get::<mongo::Mongo>().expect("No DB?");
        mongo::get_config(db, msg.guild_id().unwrap()).user.commands
            .unwrap_or(HashMap::new())
    };

    // try remove from cmds, and based on result notify user
    match cmds.remove(&name.to_lowercase()) {
        None => {
            match msg.channel_id.send_message(|m| m.embed(|e| e
                .color(Colour::red())
                .title("Command doesn't exist")
                .description("Could not find the command you wanted to remove, \
                    are you sure you wrote it right?"))) {
                Ok(_) => {},
                Err(why) => error!("MSG failed: {}", why)
            }
        }
        _ => {
            match msg.channel_id.send_message(|m| m.embed(|e| e
                .color(Colour::fooyoo())
                .title("Command removed")
                .description(format!("Command **{}** has been removed \
                succesfully", name)))) {
                Ok(_) => {},
                Err(why) => error!("MSG failed: {}", why)
            }
        }
    }

    // commit our changes
    {
        let data = ctx.data.lock();
        let db = data.get::<mongo::Mongo>().expect("No DB?");
        let mut config = mongo::get_config(db, msg.guild_id().unwrap());
        config.user.commands = Some(cmds);
        mongo::set_config(db, &config);
    }

    /*
    match msg.channel_id.send_message(|m| m.embed(|e| e
        .color(Colour::red())
        .title("Command doesn't exist")
        .description("Could not find the command you wanted to remove, \
            are you sure you wrote it right?"))) {
        Ok(_) => {},
        Err(why) => error!("MSG failed: {}", why)
    }
    */
});

// list commands available for this guild
command!(list(ctx, msg, args) {
    let _ = msg.channel_id.broadcast_typing();
    // page, but its 1 indexed
    let page = args.single::<u64>().unwrap_or(1);
    let cmds = { // load commands from Mongo, holding lock as little as possible
        let data = ctx.data.lock();
        let db = data.get::<mongo::Mongo>().expect("No DB?");
        mongo::get_config(db, msg.guild_id().unwrap()).user.commands
            .unwrap_or(HashMap::new())
    };

    // pages
    let mut results: Vec<String> = vec![];
    results.push("".into()); // element 0

    // we need to transform these keys to max 2000 character "pages"
    let mut iter = cmds.keys();
    let mut i = 0;
    while let Some(key) = iter.next() {
        if results[i].len() + key.len() >= 2000 { // this page full, move to new
            results.push("".to_string());
            i += 1;
        }

        results[i].push_str(format!("{}\n", key).as_str());
    }

    let pages = results.len();
    let page = (page as usize).max(1).min(pages);

    match msg.channel_id.send_message(|m| m.embed(|e| e
        .color(Colour::fooyoo())
        .title(format!("All commands (page {}/{})", page, pages))
        .description(results[page - 1].clone()))) {
        Err(why) => error!("MSG failed: {}", why),
        _ => {}
    }
});