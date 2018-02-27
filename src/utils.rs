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

// turn duration to a nice 0d 4h 21m 53s
pub fn fmt_difference(diff: Duration) -> String {
    format!("{}d {}h {}m {}s", 
        diff.num_days(), 
        diff.num_hours() % 24, 
        diff.num_minutes() % 60,
        diff.num_seconds() % 60)
}


// this lets us deserialize values like "5128923.2" into f64 etc. Numbers from
// strings where some json api's return wrong types
pub mod deser_string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Serializer, Deserialize, Deserializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
        where T: Display,
              S: Serializer
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
        where T: FromStr,
              T::Err: Display,
              D: Deserializer<'de>
    {
        String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
    }
}
