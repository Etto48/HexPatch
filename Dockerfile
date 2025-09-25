FROM rust

WORKDIR /app
COPY ./ /app/
RUN apt update && apt install -y build-essential cmake python3-dev 

RUN cargo build --release
CMD ["cargo", "run", "--release"]