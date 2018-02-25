# TofuBot 3
This is a rewrite of a rewrite of a bot that never left ultra-dev phase.

## Settings
TofuBot uses TOML for its configuratio files. Below is an example `configuration.toml` file
```toml
# sets your bot prefix to +
prefix = "+"
# enables logging and sets what channel it should log to.
log_channel = 10221619660849152
# set the roles that are allowed to run admin commands.
staff = [ 
    322547556211621898, 
    287058382093287437,
]
```

### Running on AWS (Amazon Linux 2 on t2.micro)
#### whoops this guide is incomplete. The basic gist: get ur binary on there
#### EDIT: Yea this doesn't actually work. OpenSSL wants some libs that AMI doesn't have. Just clone this repo to your host and `cargo build --release`

run this on your instance
```bash
sudo yum install mongodb-org
mkdir tofu3
cd tofu3
```
this on your host
```bash
cargo build --release
scp -i host.pem target/release/tofubot-rewrite-3 ec2-user@ec2-xxx-xxx-xxx-xxx.xx-xxxx-x.compute.amazonaws.com:~/tofu3
```
again on your instance
```bash
screen -S Tofu3
./tofubot-rewrite-3
```