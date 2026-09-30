# agent-workbench

A workbench for development with AI agents: a harness, a development pipeline with stage artifacts,
agent and terminal setup, and a command center in the terminal and the browser. Works with Claude
Code, Codex and other agents, for a single developer and for a team. Installs with one command.

Goal and scope are in the [brief](docs/brief.md), owner decisions in
[decisions](docs/decisions.md), research in [docs/research](docs/research/README.md). Tasks live in
this repository's GitHub Issues.

## Install

The `workbench` binary installs the harness into a project. It builds from source, so the machine
needs a Rust toolchain:

```bash
cargo install --locked --git https://github.com/KirillSachkov/agent-workbench --tag v0.1.0 workbench
```

Then, in a project's git repository:

```bash
workbench init --from KirillSachkov/agent-workbench@v0.1.0   # skills, AGENTS.md block, lock
workbench update                                             # a newer release, on its own branch
```

After `init`, run the `setup-harness` skill once in your agent to configure the tracker and labels.
The command center installs separately; see [command-center/README.md](command-center/README.md).
Releases and their notes: [CHANGELOG.md](CHANGELOG.md), process in
[docs/releasing.md](docs/releasing.md).

## License

[MIT](LICENSE)
