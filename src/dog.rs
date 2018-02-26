use dogstatsd::{Client as DogClient, Options};

lazy_static! {
    pub static ref DOG: DogClient = DogClient::new(Options::default()).unwrap();
}

fn vec(tags: Vec<String>) -> Vec<String> {
    let v = vec![
        "tofubot".to_string(), 
        format!("profile:{}", env!("CARGO_PKG_PROFILE"))
    ];
    [&v[..], &tags[..]].concat()
}

#[allow(dead_code)]
pub fn event(title: &str, content: &str, tags: Vec<String>) {
    if let Err(why) = DOG.event(title, content, vec(tags)) {
        error!("DataDog event failed: {}", why);
        error!("Title and content: {}, {}", title, content);
    }
}

#[allow(dead_code)]
pub fn incr(name: &str, tags: Vec<String>) {
    if let Err(why) = DOG.incr(name, vec(tags)) {
        error!("DataDog increment failed: {}", why);
        error!("Name: {}", name);
    }
}
#[allow(dead_code)]
pub fn decr(name: &str, tags: Vec<String>) {
    if let Err(why) = DOG.decr(name, vec(tags)) {
        error!("DataDog decrement failed: {}", why);
        error!("Name: {}", name);
    }
}

#[allow(dead_code)]
pub fn set(name: &str, val: i64, tags: Vec<String>) {
    if let Err(why) = DOG.gauge(name, &val.to_string(), vec(tags)) {
        error!("DataDog set failed: {}", why);
        error!("Name and value: {}, {}", name, val);
    }
}

#[allow(dead_code)]
pub fn time<F: FnOnce()>(name: &str, tags: Vec<String>, block: F) {
    if let Err(why) = DOG.time(name, vec(tags), block) {
        error!("DataDog time failed: {}", why);
        error!("Title: {}", name);
    }
}

#[allow(dead_code)]
pub fn timing(name: &str, val: i64, tags: Vec<String>) {
    if let Err(why) = DOG.timing(name, val, vec(tags)) {
        error!("DataDog timing failed: {}", why);
        error!("Title and value: {}, {}", name, val);
    }
}