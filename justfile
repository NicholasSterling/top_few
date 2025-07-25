#!/usr/bin/env just --justfile
set dotenv-load := true

# Output this list.
list:
    @just --list

# Output this list.
help:
    @just list

# Apply strict formatting.
fmt *FLAGS:
    cargo +nightly fmt  --all 

# Run clippy on codesbase, tests, examples, while testing all features.
check *FLAGS:
    cargo clippy --tests --examples --all-targets --all-features --workspace 

# Run tests.
test *FLAGS:
    cargo nextest run --all-features --workspace 

# Build and run.
run *FLAGS:
    cargo run 

# Generate documentation. Add '-- open' to open the docs in a web page.
doc *FLAGS:
    cargo doc --no-deps --all-features  --document-private-items --workspace --examples 

# Calculate coverage and open page with the results.
coverage *FLAGS:
    cargo llvm-cov 

# Benchmark codebase with criterion.
benchmark *FLAGS:
    cargo criterion 

# Check for unused dependencies, audit for vulnerabilities,
# and check if newer version of depenedencies is available.
thorough-check:
    cargo +nightly udeps --all-targets
    cargo audit
    cargo upgrades

# Check for unusead features. Opens results in a browser.
unused-features:
    unused-features analyze
    unused-features build-report --input report.json
    rm report.json
    mv report.html /tmp
    xdg-open /tmp/report.html

# Check build timings.
build-timings:
    cargo clean
    cargo build --release --quiet --timings
    xdg-open /target/cargo-timings/cargo-timing.html

# Runs all checks necessary before commit.
# Checks formating, code quality, tests, documentation, spellcheck and more.
pre-commit:
    @just fmt
    @just check -- -D warnings
    @just test
    @just doc
    @just thorough-check
    @just unused-features
    cargo spellcheck fix
    cargo spellcheck reflow

# Similar to `pre-commit` command, but is not interactive and doesn't modify the codebase.
# Suitable for automated CI pipelines.
ci:
    @just fmt --check
    @just check -- -D warnings
    @just test
    @just doc
    @just thorough-check
    cargo spellcheck check

# Initializes the project, installing all tools necessary. Should be run once before begining of development.
init:
    echo # installing git hooks
    pre-commit --version || pip install pre-commit
    pre-commit install || echo "failed to install git hooks!" 1>&2
    echo # installing nightly used by `just fmt` and `cargo udeps`
    rustup install nightly
    echo # installing cargo-binstall for faster setup time
    cargo binstall -V || cargo install cargo-binstall
    echo # things required by `just test`
    cargo binstall cargo-nextest --no-confirm
    echo # things required by `just watch`
    cargo binstall cargo-watch --no-confirm
    echo # things required by `just pre-commit`
    cargo binstall cargo-spellcheck --no-confirm
    echo # things required by `just coverage`
    rustup component add llvm-tools-preview
    cargo binstall cargo-llvm-cov --no-confirm
    echo # things required by `just benchmark`
    cargo binstall cargo-criterion --no-confirm
    echo # things required by `just thorough-check`
    cargo binstall cargo-udeps --no-confirm
    cargo binstall cargo-audit --no-confirm
    cargo binstall cargo-upgrades --no-confirm
    cargo binstall cargo-unused-features --no-confirm
