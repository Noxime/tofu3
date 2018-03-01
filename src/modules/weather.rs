use reqwest::Client;
use forecast::{ApiClient, ForecastRequestBuilder, ApiResponse, Lang, Units};
use serenity::utils::Colour;

use utils::deser_string;
use mongo;

use std::env;
use std::f64;

// fetch the weather information for a location
command!(weather(ctx, msg, args) {
    let key = unres_cmd!(env::var("TOFU_DARK_SKY"), "no dark sky key");

    let mut location = args.full().to_string();
    if location.trim().is_empty() {
        let user = {
            let data = ctx.data.lock();
            let db = unopt_cmd!(data.get::<mongo::Mongo>(), "no mongo!");
            mongo::get_user(db, msg.author.id)
        };

        // user has location saved
        if let Some(old) = user.location {
            location = old;
        } else { // wait nope they dont
            unres_cmd!(msg.channel_id.send_message(|m| m.embed(|e| e
                .color(Colour::red())
                .title("Please set your location")
                .description("TofuBot does not know where you are! Please \
                provide the location after this command, and TofuBot will \
                remember it next time."))), "msg failed");
            return Ok(());
        }
    } else { // hey user gave us info, write that to DB
        let data = ctx.data.lock();
        let db = unopt_cmd!(data.get::<mongo::Mongo>(), "no mongo!");
        let mut user = mongo::get_user(db, msg.author.id);
        user.location = Some(location.clone());
        mongo::set_user(db, &user);
    }

    // struct for Nominatim data
    #[derive(Debug, Serialize, Deserialize)]
    struct Nomi {
        licence: String,
        #[serde(with = "deser_string")]
        lat: f64,
        #[serde(with = "deser_string")]
        lon: f64,
        display_name: String,
    }

    let req = Client::new();

    // ask openstreetmap for the lat and long of the given text
    let data: Vec<Nomi> = unres_cmd!(
        unres_cmd!(req.get(&format!("
            https://nominatim.openstreetmap.org/search/kamppi?format=jsonv2&q={}
            ", location)).send(), 
        "nominatim request failed").json(), 
    "nominatim deserialize failed");

    // unparse that shit
    let nomi = unopt_cmd!(data.get(0), "nominatim had 0 results");
    let mut name = nomi.display_name.split(", ");
    let place = name.next().unwrap_or("Somewhere").trim();
    let country = name.last().unwrap_or("Earth").trim();

    let api = ApiClient::new(&req);
    let cast: ApiResponse = unres_cmd!(
        unres_cmd!(
            api.get_forecast(
            ForecastRequestBuilder::new(&key, nomi.lat, nomi.lon)
                .lang(Lang::English)
                .units(Units::SI)
                .build()),
        "forecast request failed").json(),
    "forecast deserialize failed");


    // lets hope we actually have the data
    let cur = unopt_cmd!(cast.currently, "no current data");
    let daily = unopt_cmd!(cast.daily, "no daily data");
    let daily = unopt_cmd!(daily.data.get(0), "no weather data").clone();
    
    // unpack our values
    let temp = cur.temperature.unwrap_or(
        daily.temperature.unwrap_or(f64::NAN));
    let high = cur.temperature_high.unwrap_or(
        daily.temperature_high.unwrap_or(f64::NAN));
    let low = cur.temperature_low.unwrap_or(
        daily.temperature_low.unwrap_or(f64::NAN));
    let wind = cur.wind_speed.unwrap_or(
        daily.wind_speed.unwrap_or(f64::NAN));
    let feels = cur.apparent_temperature.unwrap_or(
        daily.apparent_temperature.unwrap_or(f64::NAN));
    let vis = cur.visibility.unwrap_or(
        daily.visibility.unwrap_or(20f64));
    let humid = cur.humidity.unwrap_or(
        daily.humidity.unwrap_or(f64::NAN));
    let pressure = cur.pressure.unwrap_or(
        daily.pressure.unwrap_or(f64::NAN));

    // generate a nice text
    // example:
    // It is currently -15.8°C, with wind of 3.5 m/s bringing that up to a 
    // freezing -23.1°C. The sky looks clear with a visibility of about 10 km.
    let summary = format!("\
        _It is currently **{:.1}°C**, with wind of **{:.1} m/s** \
        bringing that {} to {}**{:.1}°C**. {} about **{:.0} km**._ ",
        temp,
        wind,
        if feels > temp { "up" } else { "down" },
        match feels as isize {
            -100 ... -40 => "a frightening ",
            -39 ... -20 => "a freezing ",
            -19 ... -5 => "a chilly ",
            -4 ... 11 => "a mild ",
            18 ... 24 => "a warm ",
            25 ... 32 => "a toasty ",
            33 ... 37 => "a burning ",
            38 ... 60 => "a melting hot ",
            _ => "",
        },
        feels,
        cur.summary.map(|v| 
            format!("The sky looks {} with a visibility of", v.to_lowercase()))
            .unwrap_or("The visibility is".into()),
        vis,
    );

    unres_cmd!(msg.channel_id.send_message(|m| m.embed(|e| e
        .color(Colour::fooyoo())
        .title(format!("Weather in **{}**, **{}**", 
            place, country))
        .description(summary)
        .footer(|f| f.text("Forecast by Dark Sky"))
        .field("Temperature", format!("\
            Current: **{:.2}°C**\nHigh/low: **{:.2}°C**/**{:.2}°C**",
            temp,
            high, 
            low
            ), true)
        .field("Wind chill", format!("\
            Feels like: **{:.2}°C**\nWind speed: **{:.2} m/s**",
            feels,
            wind
            ), true)
        .field("Atmosphere", format!("\
            Humidity: **{:.2}%**\nPressure: **{:.2} hPa**",
            humid * 100f64,
            pressure), true)
    )), "msg failed");
});