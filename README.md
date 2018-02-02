# TofuBot 3
This is a rewrite of a rewrite of a bot that never left ultra-dev phase.

### Running on AWS (Linux 2 micro)
#### whoops this guide is incomplete. The basic gist: get ur binary on there

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