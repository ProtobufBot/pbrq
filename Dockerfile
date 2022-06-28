FROM rust as builder
RUN sed -i "s@http://deb.debian.org@http://mirrors.aliyun.com@g" /etc/apt/sources.list \
 && sed -i "s@http://security.debian.org@http://mirrors.aliyun.com@g" /etc/apt/sources.list \
 && apt update \
 && apt install -y musl-tools \
 && rustup default nightly \
 && rustup target add x86_64-unknown-linux-musl
WORKDIR app
COPY . .
RUN cargo +nightly build --release --target=x86_64-unknown-linux-musl

FROM alpine as runtime
WORKDIR app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/main /usr/local/bin/pbrq
RUN wget https://github.com/lz1998/ricq-react-ui/releases/latest/download/static.zip && unzip static.zip && rm static.zip
CMD ["pbrq","--bind-addr", "0.0.0.0:9000", "--static-dir","static"]
