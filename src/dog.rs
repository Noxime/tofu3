use dogstatsd::{Client as DogClient, Options};

lazy_static! {
    pub static ref DOG: DogClient = DogClient::new(Options::default()).unwrap();
}

fn vec() -> Vec<String> {
    vec!["tofubot".to_string(), format!("profile:{}", env!("CARGO_PKG_PROFILE"))]
}

#[allow(dead_code)]
pub fn event(title: &str, content: &str, tags: Vec<String>) {
    DOG.event(title, content, [&vec()[..], &tags[..]].concat()).unwrap();
}

#[allow(dead_code)]
pub fn incr(name: &str, tags: Vec<String>) {
    DOG.incr(name, [&vec()[..], &tags[..]].concat()).unwrap();
}
#[allow(dead_code)]
pub fn decr(name: &str, tags: Vec<String>) {
    DOG.decr(name, [&vec()[..], &tags[..]].concat()).unwrap();
}

#[allow(dead_code)]
pub fn set(name: &str, val: i64, tags: Vec<String>) {
    DOG.gauge(name, &val.to_string(), [&vec()[..], &tags[..]].concat()).unwrap();
}

#[allow(dead_code)]
pub fn time<F: FnOnce()>(name: &str, tags: Vec<String>, block: F) {
    DOG.time(name, tags, block).unwrap();
}