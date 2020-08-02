FROM balenalib/rpi-alpine:latest
COPY ./bin/armv7-unknown-linux-musleabihf/hueflow-rs /usr/local/bin/hueflow-rs
CMD ["hueflow-rs"]
