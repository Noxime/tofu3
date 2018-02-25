use serenity::model::channel::Message;
use serenity::model::user::User;
use serenity::client::Context;
use serenity::utils::Colour;
use perspective::model::*;

use PerspectiveLock;
use mongo;

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

// go through a users message history and sum up their values
command!(analyze_cmd(ctx, msg, args) {
    let user = args.single::<User>().unwrap_or(msg.author.clone());
    let msgs = {
        let data = ctx.data.lock();
        let db = data.get::<mongo::Mongo>().expect("No DB?");
        mongo::get_messages(db, user.id)
    };

    let mut p = Perspectives::default();
    let mut s = 0;

    for m in msgs {
        if let Some(m) = m.analysis {
            if let Some(v) = m.perspective {
                p = Perspectives {
                    toxic: p.toxic + v.toxic,
                    extra_toxic: p.extra_toxic + v.extra_toxic,
                    reject: p.reject + v.reject,
                    obscene: p.obscene + v.obscene,
                    spam: p.spam + v.spam,
                    unsubstancial: p.unsubstancial + v.unsubstancial,
                    incoherent: p.incoherent + v.incoherent,
                    inflammatory: p.inflammatory + v.inflammatory,
                    attack_author: p.attack_author + v.attack_author,
                    attack_commenter: p.attack_commenter + v.attack_commenter,
                };
                s += 1;
            }
        }
    }

    match msg.channel_id.send_message(|m| m.embed(|e| e
        .color(Colour::fooyoo())
        .title("Analysis")
        .description(format!("\
            Based on an analysis of **{}** messages, **{}** is:",
            s, user.name))
        .field("Toxicity", format!("\
            Casual: **{:.1}%**\nExtra: **{:.1}%**", 
            p.toxic       / s as f64 * 100f64, 
            p.extra_toxic / s as f64 * 100f64), true)
        .field("Message types", format!("\
            Spam: **{:.1}%**\nUnsubstancial: **{:.1}%**\nIncoherent: **{:.1}%**", 
            p.spam          / s as f64 * 100f64, 
            p.unsubstancial / s as f64 * 100f64,
            p.incoherent    / s as f64 * 100f64), true)
        .field("Writing style", format!("\
            Reject: **{:.1}%**\nObscene: **{:.1}%**\nInflammatory: **{:.1}%**",
            p.reject       / s as f64 * 100f64, 
            p.obscene      / s as f64 * 100f64, 
            p.inflammatory / s as f64 * 100f64), true)
        )) {
        Err(why) => error!("MSG failed: {:#?}", why),
        _ => {},
    }
});