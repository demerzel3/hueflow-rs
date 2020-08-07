# hueflow.rs

An autopilot for my home lights based on the time of day.
Meant to run 24/7 on a Raspberry PI.

## Build instructions

```sh
cross build --target=armv7-unknown-linux-musleabihf --release
mkdir -p ./bin/armv7-unknown-linux-musleabihf/
cp ./target/armv7-unknown-linux-musleabihf/release/hueflow-rs ./bin/armv7-unknown-linux-musleabihf/
docker build -t demerzel3/hueflow-rs .
docker push demerzel3/hueflow-rs:latest
```
