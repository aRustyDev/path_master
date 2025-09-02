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
    # @goreleaser release --snapshot --clean
    # @git tag -a v0.1.0 -m "First release"
    # @git push origin v0.1.0
    # @goreleaser release
    @rm -f assets/cosign.key
    @git add .goreleaser.yaml
    @git commit --amend -S --no-edit
    @git tag -f v1.0.0 main
    @FORCE_COLOR=1 op run --env-file=.env -- goreleaser build --clean
