use typemap::Key;
use mongodb::{Client, ThreadedClient};
use mongodb::db::{Database, ThreadedDatabase};
use mongodb::coll::options::{FindOneAndUpdateOptions, FindOptions};
use bson;
use bson::Bson;
use serenity::model::id::{GuildId, UserId, RoleId, ChannelId, MessageId};
use serenity::model::channel::Message;

use std::convert::From;
use std::collections::HashMap;

use dog;
use modules::analyze::Analysis;

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

// we like to store all discord messages locally too because someone might
// delete or edit and we want to possibly run analysis on these messages so
// instead of hitting discord api constantly we can just look at our own shit
// and wow this is a long sentance but who cares really its just me reading the
// source
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MongoMessage {
    #[serde(rename = "_id")]
    pub message_id: i64,
    pub channel_id: i64,
    pub user_id: i64,
    pub content: String,
    pub analysis: Option<Analysis>
}
impl From<(Message, Option<Analysis>)> for MongoMessage {
    fn from(msg_scan: (Message, Option<Analysis>)) -> Self {
        let (msg, scan) = msg_scan;
        Self {
            message_id: msg.id.0 as i64,
            channel_id: msg.channel_id.0 as i64,
            user_id: msg.author.id.0 as i64,
            content: msg.content,
            analysis: scan,
        }
    }
}
impl MongoMessage {
    pub fn user(&self) ->  UserId {
        UserId(self.user_id as u64)
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

// fetch a stored message from MongoDB with message id
pub fn get_message(db: &Database, id: MessageId) -> Option<MongoMessage> {
    let mut ret: Option<MongoMessage> = None;
    dog::incr("db.message.get.count", vec![]);
    dog::time("db.message.get", vec![], || { // profiling is nice

        let collection = db.collection("messages");
        ret = match collection.find_one(Some(doc!{ "_id": id.0 as i64 }), None){
            Ok(option) => {
                match option {
                    Some(value) => Some(bson::from_bson(Bson::Document(value))
                        .expect("Failed to deserialize message")),
                    None => None
                }
            },
            Err(why) => {
                error!("Failed to access MongoDB: {:#?}", why);
                None
            }
        };
    });

    ret
}

// put a new message in the db
pub fn set_message(db: &Database, msg: &MongoMessage) {
    dog::incr("db.message.set.count", vec![]);
    dog::time("db.message.set", vec![], || { // profiling is nice
        let options = FindOneAndUpdateOptions {
            return_document: None,
            max_time_ms: None,
            projection: None,
            sort: None,
            upsert: Some(true),
            write_concern: None
        };

        let collection = db.collection("messages");
        if let Bson::Document(document) = bson::to_bson(msg).unwrap() {
            match collection.find_one_and_replace(
                doc! { "_id" => msg.message_id },
                document,
                Some(options)) {
                Ok(_) => {},
                Err(why) => {
                    error!("Failed to write mongo message: {}", why);
                }
            }
        }
    });
}

// fetch a config from mongo
pub fn get_config(db: &Database, id: GuildId) -> GuildConfig {
    let mut ret = GuildConfig::new(id.0 as i64);
    dog::incr("db.config.get.count", vec![]);
    dog::time("db.config.get", vec![], || {

        let collection = db.collection("configs");
        ret = match collection.find_one(Some(doc!{ "_id": id.0 as i64 }), None){
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
    });

    ret
}

pub fn set_config(db: &Database, config: &GuildConfig) {
    dog::incr("db.config.set.count", vec![]);
    dog::time("db.config.set", vec![], || {
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
    });
}

pub fn get_user(db: &Database, id: UserId) -> UserConfig {
    let mut ret = UserConfig::new(id.0 as i64);
    dog::incr("db.user.get.count", vec![]);
    dog::time("db.user.get", vec![], || {

        let collection = db.collection("users");
        ret = match collection.find_one(Some(doc!{ "_id": id.0 as i64 }), None){
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
        });

    ret
}

pub fn set_user(db: &Database, user: &UserConfig) {
    dog::incr("db.user.set.count", vec![]);
    dog::time("db.user.set", vec![], || {
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
    });
}

// find the users with top score in certain guild
pub fn get_top_users(db: &Database, id: GuildId, limit: i64) -> Vec<UserConfig> {
    let mut results: Vec<UserConfig> = vec![];

    dog::incr("db.leaderboard.count", vec![]);
    dog::time("db.leaderboard", vec![format!("guild:{}", id)], || {
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

            let parsed = bson::from_bson(Bson::Document(doc))
                .expect("Failed to deserialize user");

            results.push(parsed);
        }
    });

    results
}

// fetch all messages for a single user
pub fn get_messages(db: &Database, id: UserId) -> Vec<MongoMessage> {
    let mut results: Vec<MongoMessage> = vec![];

    dog::incr("db.messages.count", vec![]);
    dog::time("db.messages", vec![format!("user:{}", id)], || {
        let collection = db.collection("messages");
        let cursor = collection.find(Some(doc! {
            "user_id" => id.0 as i64
        }), None).unwrap();

        for item in cursor {
            let doc = item.unwrap();

            let parsed = bson::from_bson(Bson::Document(doc))
                .expect("Failed to deserialize message");

            results.push(parsed)
        }
    });

    results
}