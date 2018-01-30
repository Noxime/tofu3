use typemap::Key;
use mongodb::{Client, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use mongodb::coll::options::FindOneAndUpdateOptions;
use bson;
use bson::Bson;
use serenity::model::id::GuildId;

// This stores our mongo database in our framework
pub struct Mongo;
impl Key for Mongo {
    type Value = Database;
}

// de/serialize this from db.guild_configs
#[derive(Serialize, Deserialize, Debug)]
pub struct GuildConfig {
    #[serde(rename = "_id")]
    pub guild_id: i64,
    pub user: Changeable,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Changeable {
    pub prefix: Option<String>
}

impl GuildConfig {
    fn new(id: i64) -> Self {
        Self {
            guild_id: id,
            user: Changeable {
                prefix: None
            }
        }
    }
}

// connect to a mongodb instance running on this machine and return a database
// from it
pub fn connect() -> Database {
    let client = Client::connect("localhost", 27017)
        .expect("Failed to connect to MongoDB");
    info!("Connected to MongoDB");
    client.db("tofu3")
}

// fetch a config from mongo
pub fn get_config(db: &Database, id: GuildId) -> GuildConfig {
    let collection = db.collection("guild_configs");
    match collection.find_one(Some(doc! { "_id": id.0 as i64 }), None) {
        Ok(option) => {
            match option {
                Some(value) => bson::from_bson(Bson::Document(value))
                    .expect("Failed to deserialize guild config"),
                None => {
                    let config = GuildConfig::new(id.0 as i64);
                    set_config(db, &config);
                    config
                }
            }
        },
        Err(why) => {
            error!("Failed to access MongoDB: {:#?}", why);
            let config = GuildConfig::new(id.0 as i64);
            set_config(db, &config);
            config
        }
    }
}

pub fn set_config(db: &Database, config: &GuildConfig) {
    // turn on upsert
    let options = FindOneAndUpdateOptions {
        return_document: None,
        max_time_ms: None,
        projection: None,
        sort: None,
        upsert: Some(true),
        write_concern: None
    };
    let collection = db.collection("guild_configs");
    if let Bson::Document(document) = bson::to_bson(&config).unwrap() {
        match collection.find_one_and_replace(
            doc!{ "_id" => config.guild_id }, 
            document,
            Some(options)) {
            Ok(_) => {},
            Err(why) => {
                error!("Failed to set a guild config: {:#?}", why);
            }
        }
    }
}