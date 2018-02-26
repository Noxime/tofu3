use time::Duration;

// this is a little convenience macro that basically checks if we can unwrap,
// and if not it warns with the provided message and does an early return with
// whatever we gave it "default value". if we dont give $r, it defaults to ()
macro_rules! unopt {
    ($e:expr, $m:expr, $r:expr) => ({
        match $e {
            Some(v) => v,
            None => {
                warn!("Nothing to unwrap");
                warn!("{}", $m);
                return $r;
            }
        }
    });
    ($e:expr, $m:expr) => ({
        unopt!($e, $m, ())
    });
}

// same as unopt! but for results
macro_rules! unres {
    ($e:expr, $m:expr, $r:expr) => ({
        match $e {
            Ok(v) => v,
            Err(why) => {
                warn!("Unwrap error: {}", why);
                warn!("{}", $m);
                return $r;
            }
        }
    });
    ($e:expr, $m:expr) => ({
        unres!($e, $m, ())
    });
}

// these are the variants that should be used withing commands, that "propagate"
// an Err(CommandError($m))
macro_rules! unopt_cmd {
    ($e:expr, $m:expr) => ({
        use serenity::framework::standard::CommandError;
        match $e {
            Some(v) => v,
            None => {
                return Err(CommandError(format!("unopt_cmd: {}", $m)));
            }
        }
    });
}
macro_rules! unres_cmd {
    ($e:expr, $m:expr) => ({
        use serenity::framework::standard::CommandError;
        match $e {
            Ok(v) => v,
            Err(why) => {
                return Err(CommandError(format!("unres_cmd: {}, {}", $m, why)));
            }
        }
    });
}

pub fn fmt_difference(diff: Duration) -> String {
    format!("{}d {}h {}m {}s", 
        diff.num_days(), 
        diff.num_hours() % 24, 
        diff.num_minutes() % 60,
        diff.num_seconds() % 60)
}