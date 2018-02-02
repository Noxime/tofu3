use mongo;

use serenity::utils::Colour;
use serenity::framework::standard::CommandError;
use toml;

use std::error::Error;

// change a configuratio for a guild. there are two things that can happen when
// using this command, depending on if a file was attached
// if file attached, parse it and try to set it as the guild config
// if no file, serialize current config and return it
command!(settings(ctx, msg) {
    match msg.attachments.iter().next() {
        Some(file) => {
            // get the file from discord CDN
            // TODO: Error reporting
            let bytes = file.download()
                .expect("Could not download attachment");
            // parse bytes to string
            let file = match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(why) => {
                    let _ = msg.channel_id.send_message(|f| f.embed(|m| m
                        .color(Colour::red())
                        .title("Invalid configuration file")
                        .description(format!(
                            "The configuration file you uploaded contains \
                            errors: {}", why.utf8_error().description()
                        ))));
                    return Err(CommandError("utf8 failed to parse".into()));
                }
            };

            // deserialize the configuration from uploaded TOML
            let new_change: mongo::Changeable = 
                match toml::from_str(file.as_str()) {
                Ok(v) => v,
                Err(why) => {
                    let _ = msg.channel_id.send_message(|f| f.embed(|m| m
                        .color(Colour::red())
                        .title("Invalid configuration file")
                        .description(format!(
                            "The configuration file you uploaded contains \
                            errors: {}", why
                        ))));
                    return Err(CommandError("settings failed to parse".into()));
                }
            };

            // load the current config
            {
                let data = ctx.data.lock();
                let db = data.get::<mongo::Mongo>().unwrap();
                let mut config = mongo::get_config(db, msg.guild_id().unwrap());
                config.user = new_change;
                mongo::set_config(db, &config);
            };

            // Notify user the change was successful
            let _ = msg.channel_id.send_message(|f| f.embed(|m| m
                .color(Colour::fooyoo())
                .title("Configuration changed")
                .description("Congratulations, the configuration file has been \
                updated!"
                )));
        },
        None => {
            // load the current configuration
            let change = {
                let data = ctx.data.lock();
                let db = data.get::<mongo::Mongo>().unwrap();
                mongo::get_config(db, msg.guild_id().unwrap()).user
            };

            // serialize and turn to both full string and to embeddable
            let string = toml::to_string_pretty(&change).unwrap();
            let mut print = string.replace("`", "\\`");
            if print.len() > 1500 {
                print.truncate(1500);
                print.push_str("\n<preview clipped>");
            }

            // show the current config in embed
            let _ = msg.channel_id.send_message(|f| f.embed(|m| m
                .color(Colour::fooyoo())
                .title("Current configuration file")
                .description(format!(
                    "Use Notepad++ or a similar program to edit this TOML file \
                    \n```toml\n{}\n```",
                    print
                ))));
            let _ = msg.channel_id.send_files(
                vec![(string.as_bytes(), "configuration.toml")],
                |f| f
            );
        }
    };
});