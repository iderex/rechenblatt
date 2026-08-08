# rechenblatt

Reliable execution of complex spreadsheet macros and pixel-accurate rendering fidelity for complex spreadsheet files, so documents from the incumbent office suite open and run faithfully on your own machine.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

## From a clone to a green run

```
git clone https://github.com/iderex/rechenblatt.git
cd rechenblatt
cargo build --locked --workspace
cargo test --locked --workspace
```

Those two commands are the whole build and the whole suite. Nothing else has to
be installed and nothing has to be configured first.

The compiler version is pinned in `rust-toolchain.toml`. rustup reads that file
and fetches the pinned version on the first build, so two machines building this
commit use the same compiler whatever each has set as default. Without rustup,
install the version that file names; an older one is refused at the start of the
build rather than failing later inside a dependency.

`--locked` is part of both commands rather than an option on them. It makes a
build that would change `Cargo.lock` fail instead of quietly updating it, which
is what keeps the dependency set the same on your machine and on the one that
built it before you.

The binary runs and does nothing, which is the honest state of it:

```
cargo run --locked --bin rechenblatt
```

The engine is five library crates under `crates/`, and the command is a sixth
that depends on all of them. `Cargo.toml` at the root says which of them may
depend on which, and why.

`CONTRIBUTING.md` has the guards that read the tracked tree, and what a change
here is judged by.

See [NOTICE.md](NOTICE.md) for the intended-use notice,
[docs/intended-use.md](docs/intended-use.md) for what the maintainers consider
outside this project and what the software does not prevent, and
[SECURITY.md](SECURITY.md) for the private reporting route.
