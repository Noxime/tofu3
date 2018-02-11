use serenity::framework::standard::CommandError;
use serenity::utils::Colour;
use mongo;

use std::collections::HashMap;

// register a new command in the database
command!(new(ctx, msg, args) {
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