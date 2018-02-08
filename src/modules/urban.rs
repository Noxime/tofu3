use reqwest::Client;
use urbandictionary::ReqwestUrbanDictionaryRequester; // jesus this is like java
use serenity::utils::Colour;

// search urban dictionary for a given string
command!(urban(_ctx, msg, args) {
    let _ = msg.channel_id.broadcast_typing();
    // Search term
    if let Ok(query) = args.single::<String>() {

        // use Zeyla's lib for accessing UB nicely
        // TODO: See if this will be a performance issue (probably not tho)
        let client = Client::new();
        let response = match client.define(&query) {
            Ok(r) => r,
            Err(why) => {
                error!("Failed to access UB: {:#?}", why);
                None // TODO: Tell user about error rather than returning none
            }
        };

        // did we even find anything
        match response {
            Some(def) => {
                let _ = msg.channel_id.send_message(|f| f.embed(|m| m
                    .color(Colour::fooyoo())
                    .title(&format!("Definition of {}", &def.word))
                    .url(&def.permalink)
                    .description(&def.definition)
                    .field("Example", &def.example, true)
                    .field("Votes", format!("👍: **{}** 👎: **{}**", 
                        &def.thumbs_up, &def.thumbs_down), true)
                    .footer(|f| f
                        .text(&format!("Defined by {}", def.author)))
                    ));
            },
            None => {
                let _ = msg.channel_id.send_message(|f| f.embed(|m| m
                    .color(Colour::gold())
                    .title(format!("Could not find \"{}\"", query))
                    .description(format!(
                        "Could not find \"{}\" on Urban Dictionary. Are you \
                        sure you wrote it correctly?",
                        query))));
            }
        }
    } else {
        let _ = msg.channel_id.send_message(|f| f.embed(|m| m
            .color(Colour::red())
            .title("Incorrect usage")
            .description("Please provide a search term!")));
    }
});