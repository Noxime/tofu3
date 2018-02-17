use typemap::Key;
use mongodb::{Client, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use mongodb::coll::options::{FindOneAndUpdateOptions, FindOptions};
use bson;
use bson::Bson;
use serenity::model::id::{GuildId, UserId, RoleId, ChannelId};

use std::collections::HashMap;

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
    pub prefix: Option<String>, // guild prefix
    pub staff: Option<Vec<i64>>, // admin roles
    pub commands: Option<HashMap<String, String>>, // custom commands
    pub log_channel: Option<i64>, // the channel we output our logging to
}
impl GuildConfig {
    fn new(id: i64) -> Self {
        Self {
            guild_id: id,
            user: Changeable {
                prefix: None,
                staff: None,
                commands: None,
                log_channel: None,
            }
        }
    }

    // check if provided role id is set as a staff role. 
    pub fn staff(&self) -> Vec<RoleId> {
        let mut ret = vec![];
        if let Some(ref roles) = self.user.staff {
            for role in roles {
                ret.push(RoleId(*role as u64));
            }
        }
        ret
    }

    pub fn log(&self) -> Option<ChannelId> {
        self.user.log_channel.map(|v| ChannelId::from(v as u64))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserConfig {
    #[serde(rename = "_id")]
    pub user_id: i64,
    pub scores: HashMap<String, i64> // guild_id, score
}
impl UserConfig {
    fn new(id: i64) -> Self {
        Self {
            user_id: id,
            scores: HashMap::new()
        }
    }

    pub fn get_score(&self, id: GuildId) -> i64 {
        *self.scores.get(&(id.0 as i64).to_string()).unwrap_or(&0i64)
    }

    pub fn set_score(&mut self, id: GuildId, score: i64) {
        self.scores.insert((id.0 as i64).to_string(), score);
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
    let collection = db.collection("configs");
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
    let collection = db.collection("configs");
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

pub fn get_user(db: &Database, id: UserId) -> UserConfig {
    let collection = db.collection("users");
    match collection.find_one(Some(doc! { "_id": id.0 as i64 }), None) {
        Ok(option) => {
            match option {
                Some(value) => bson::from_bson(Bson::Document(value))
                    .expect(format!("Failed to deserialize user config {}", 
                        id.0 as i64).as_str()),
                None => {
                    let user = UserConfig::new(id.0 as i64);
                    set_user(db, &user);
                    user
                }
            }
        },
        Err(why) => {
            error!("Failed to access MongoDB: {:#?}", why);
            let user = UserConfig::new(id.0 as i64);
            set_user(db, &user);
            user
        }
    }
}

pub fn set_user(db: &Database, user: &UserConfig) {
    // turn on upsert
    let options = FindOneAndUpdateOptions {
        return_document: None,
        max_time_ms: None,
        projection: None,
        sort: None,
        upsert: Some(true),
        write_concern: None
    };
    let collection = db.collection("users");
    if let Bson::Document(document) = bson::to_bson(&user).unwrap() {
        match collection.find_one_and_replace(
            doc!{ "_id" => user.user_id }, 
            document,
            Some(options)) {
            Ok(_) => {},
            Err(why) => {
                error!("Failed to set a user config for {}: {:#?}", 
                    user.user_id, 
                    why);
            }
        }
    }
}

// find the users with top score in certain guild
pub fn get_top_users(db: &Database, id: GuildId, limit: i64) -> Vec<UserConfig> {
    let mut results: Vec<UserConfig> = vec![];
    // set a sorting mode based on the guild id
    let mut options = FindOptions::new();
    options.sort = Some(doc! {
        format!("scores.{}", id): -1 // -1 means biggest first
    });
    options.limit = Some(limit);

    // we seach in users collection
    let collection = db.collection("users");
    debug!("scores.{}", id);
    // our results
    let cursor = collection.find(Some(doc! {
        format!("scores.{}", id): doc! { "$exists": true }
    }), Some(options)).unwrap();

    // deserialize all our results
    for item in cursor {
        let doc = item.unwrap();

        let parsed = bson::from_bson(Bson::Document(doc)).expect(
            &format!("Failed to deserialize user"));

        results.push(parsed);
    }

    return results;
}
