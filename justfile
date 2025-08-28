build:
    @cargo build

run:
    @cargo run

clean:
    @rm -rf ./target

install:
    @cargo install --path .
