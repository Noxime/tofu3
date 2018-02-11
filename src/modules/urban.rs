use reqwest::Client;
use urbandictionary::ReqwestUrbanDictionaryRequester; // jesus this is like java
use serenity::utils::Colour;

// search urban dictionary for a given string
command!(urban(_ctx, msg, args) {
    let _ = msg.channel_id.broadcast_typing();
    // Search term
    let query = args.full();
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
            // discord only accepts 2000 characters. 1800 should give us enough
            // headroom for our example field to fit
            let mut s = def.definition.clone();
            if s.len() > 1800 {
                s.truncate(1800);
            }

            // discord doesn't allow empty fields, so add placeholder incase no 
            // example
            // TODO: I guess we just shouldnt send a field incase this is empty
            let mut e = def.example.clone();
            if e.is_empty() {
                e = "<none>".into();
            }
            
            match msg.channel_id.send_message(|f| f.embed(|m| m
                .color(Colour::fooyoo())
                .title(&format!("Definition of {}", &def.word))
                .url(&def.permalink)
                .description(s)
                .field("Example", e, true)
                .field("Votes", format!("👍: **{}** 👎: **{}**", 
                    &def.thumbs_up, &def.thumbs_down), true)
                .footer(|f| f
                    .text(&format!("Defined by {}", def.author)))
                )) {
                Ok(_) => {},
                Err(why) => error!("Sending UB failed: {:#?}", why)
            }
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
});