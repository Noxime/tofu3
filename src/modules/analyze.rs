use serenity::model::channel::Message;
use serenity::client::Context;
use perspective::model::*;
use perspective::PerspectiveClient;

use PerspectiveLock;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Perspectives {
    toxic: f64,
    extra_toxic: f64,
    reject: f64,
    obscene: f64,
    spam: f64,
    unsubstancial: f64,
    incoherent: f64,
    inflammatory: f64,
    attack_author: f64,
    attack_commenter: f64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Analysis {
    perspective: Option<Perspectives>,
}

pub fn analyze(ctx: &Context, msg: &Message) -> Analysis {
    let perspective = {
        let data = ctx.data.lock();
        data.get::<PerspectiveLock>().unwrap().clone()
    };

    let analysis = match perspective.analyze(msg.content.as_str(), vec![
        ValueType::AttackOnAuthor,
        ValueType::AttackOnCommenter,
        ValueType::Incoherent,
        ValueType::Inflammatory,
        ValueType::LikelyToReject,
        ValueType::Obscene,
        ValueType::SevereToxicity,
        ValueType::Spam,
        ValueType::Toxicity,
        ValueType::Unsubstantial,
    ]) {
        Ok(v) => v,
        Err(PerspectiveError::EmptyInput) => { return Analysis::default() },
        Err(err) => {
            warn!("Analyzing message {} failed: {:?}", msg.id, err);
            return Analysis::default();
        }
    };

    Analysis {
        perspective: Some(Perspectives {
            toxic: analysis.scores.get(&ValueType::Toxicity)
                .map(|v| v.summary.value).unwrap_or(0f64),
            extra_toxic: analysis.scores.get(&ValueType::SevereToxicity)
                .map(|v| v.summary.value).unwrap_or(0f64),
            reject: analysis.scores.get(&ValueType::LikelyToReject)
                .map(|v| v.summary.value).unwrap_or(0f64),
            obscene: analysis.scores.get(&ValueType::Obscene)
                .map(|v| v.summary.value).unwrap_or(0f64),
            spam: analysis.scores.get(&ValueType::Spam)
                .map(|v| v.summary.value).unwrap_or(0f64),
            unsubstancial: analysis.scores.get(&ValueType::Unsubstantial)
                .map(|v| v.summary.value).unwrap_or(0f64),
            incoherent: analysis.scores.get(&ValueType::Incoherent)
                .map(|v| v.summary.value).unwrap_or(0f64),
            inflammatory: analysis.scores.get(&ValueType::Inflammatory)
                .map(|v| v.summary.value).unwrap_or(0f64),
            attack_author: analysis.scores.get(&ValueType::AttackOnAuthor)
                .map(|v| v.summary.value).unwrap_or(0f64),
            attack_commenter: analysis.scores.get(&ValueType::AttackOnCommenter)
                .map(|v| v.summary.value).unwrap_or(0f64),
        })
    }
}