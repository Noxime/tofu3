use time::Duration;

pub fn fmt_difference(diff: Duration) -> String {
    format!("{}d {}h {}m {}s", 
        diff.num_days(), 
        diff.num_hours() % 24, 
        diff.num_minutes() % 60,
        diff.num_seconds() % 60)
}