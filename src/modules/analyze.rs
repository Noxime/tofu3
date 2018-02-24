use serenity::model::channel::Message;
use serenity::client::Context;
use perspective::model::*;

use PerspectiveLock;

#[derive(Serialize, Deserialize, Debug)]
struct Perspectives {
    toxic: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Analysis {
    perspective: Option<Perspectives>,
}

impl Default for Analysis {
    fn default() -> Self {
        Self { perspective: None }
    }
}

pub fn analyze(ctx: &Context, msg: &Message) -> Analysis {
    
    //let x = {
        let data = ctx.data.lock();
        let perspective = data.get::<PerspectiveLock>().expect("No perspective");
    //};

    let analysis = match perspective.analyze(msg.content.as_str()) {
        Ok(v) => v,
        Err(PerspectiveError::EmptyInput) => { return Analysis::default() },
        Err(err) => {
            warn!("Analyzing message {} failed: {:?}", msg.id, err);
            return Analysis::default();
        }
    };

    Analysis {
        perspective: Some(Perspectives {
            toxic: analysis.scores.get(&ValueType::TOXICITY)
                .expect("No toxicity value").summary.value
        })
    }
}