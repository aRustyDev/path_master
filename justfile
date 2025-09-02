build:
    @cargo build

run:
    @cargo run

clean:
    @rm -rf ./target

install:
    @cargo install --path .

check:
    @goreleaser check

release:
    @goreleaser release --snapshot --clean
    @git tag -a v0.1.0 -m "First release"
    @git push origin v0.1.0
    @goreleaser release
