use reqwest::Client;
use forecast::{ApiClient, ForecastRequestBuilder, ApiResponse};
use serenity::utils::Colour;

use utils::deser_string;

use std::env;

// fetch the weather information for a location
command!(weather(_ctx, msg, args) {
    let key = unres_cmd!(env::var("TOFU_DARK_SKY"), "no dark sky key");

    // struct for Nominatim data
    #[derive(Debug, Serialize, Deserialize)]
    struct Nomi {
        #[serde(with = "deser_string")]
        place_id: usize,
        licence: String,
        #[serde(with = "deser_string")]
        osm_id: usize,
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
            ", args.full())).send(), 
        "nominatim request failed").json(), 
    "nominatim deserialize failed");

    // unparse that shit
    let nomi = unopt_cmd!(data.get(0), "nominatim had 0 results");
    let mut name = nomi.display_name.split(", ");
    let place = name.next().unwrap_or("unknown").trim();
    let country = name.last().unwrap_or("unknown").trim();

    let api = ApiClient::new(&req);
    let cast: ApiResponse = unres_cmd!(
        unres_cmd!(
            api.get_forecast(
            ForecastRequestBuilder::new(&key, nomi.lat, nomi.lon).build()),
        "forecast request failed").json(),
    "forecast deserialize failed");
    let cur = unopt_cmd!(cast.currently, "no weather data");

    unres_cmd!(msg.channel_id.send_message(|m| m.embed(|e| e
        .color(Colour::fooyoo())
        .title("Weather search results")
        .description(format!("Showing results for **{}**, **{}**", 
            place, country))
        .field("Temperature", format!("\
            Current: **{}**\nHigh/low: **{}**/**{}**",
            cur.temperature.unwrap_or(0f64),
            cur.temperature_high.unwrap_or(0f64), 
            cur.temperature_low.unwrap_or(0f64)), true)
    )), "msg failed");
});