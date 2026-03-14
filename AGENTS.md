# AGENTS.md

## Project identity

`fpt-cli` is a **CLI-first** interface for Autodesk Flow Production Tracking (ShotGrid / FPT).
It is designed for both humans and automation systems, especially agent workflows such as **OpenClaw**.

The project goal is not only to provide a practical CLI for ShotGrid/FPT operations, but also to explore a broader workflow idea:
**use a stable CLI contract as the primary agent integration surface**, so agents can execute precise commands with structured output and lower token overhead than an MCP-heavy approach.

This repository is therefore a practical validation and exploration of the ideas discussed in:
- [MCP Is Dead, Long Live the CLI](https://ejholmes.github.io/2026/02/28/mcp-is-dead-long-live-the-cli.html)

## Language convention

All **development-facing descriptions** in this project should be written in **English**.
This includes, when applicable:
- design notes
- architecture descriptions
- code comments
- test descriptions
- developer documentation
- agent instructions
- future AGENTS-style guidance

User-facing examples may still reflect real-world usage needs, but the default development language for repository artifacts is English.

Public-facing repository entry documentation must remain bilingual:
- `README.md` is the English version
- `README_zh.md` is the Simplified Chinese version
- Both files should stay aligned in structure, examples, and feature coverage


## Core product principles

- Prefer a **CLI contract** that is stable, scriptable, and easy for agents to call.
- Prefer **structured JSON output** for machine consumption.
- Keep the command surface friendly to **OpenClaw and similar agents**.
- Reduce unnecessary **MCP token consumption** by moving repeatable operations into explicit CLI commands.
- Keep transport details behind the CLI/domain boundary so the external contract remains stable.
- Preserve safe workflows for write operations, including `--dry-run` and explicit confirmation where appropriate.

## Development environment

All environments and toolchains in this repository should be executed through **`vx`**.
Command collections should be exposed through the repository **`justfile`**.

Examples:

```bash
vx just test
vx just check
vx just lint
vx just fmt
vx just run capabilities --output json
```

When invoking language or tool binaries directly, prefer the `vx` wrapper form, for example:

```bash
vx cargo test --workspace
vx cargo run -p fpt-cli -- capabilities --output json
```

Do not assume globally installed toolchains when a `vx`-managed command is available.

## Agent-oriented development guidance

When extending this project, optimize for the following:

- **Low-friction automation**: commands should be easy to compose from shells, scripts, and agents.
- **Predictable machine-readable output**: JSON output should remain explicit and stable.
- **Clear capability boundaries**: document what each command does and what contract it exposes.
- **Operational efficiency**: prefer designs that reduce redundant auth or repeated protocol overhead.
- **OpenClaw compatibility**: new functionality should be convenient for agent orchestration, not only for interactive human usage.

## Testing expectations

- Add or update tests whenever ShotGrid/FPT-facing behavior changes.
- Prefer separate test files under `tests/` for integration-style coverage.
- Keep coverage focused on command contracts, app orchestration, and transport mapping.
- Maintain confidence for the interfaces that OpenClaw or other agents rely on.

## Scope reminder

This project is not just a generic ShotGrid utility.
It is a **CLI platform for ShotGrid/FPT operations** and an **agent integration experiment**:
practical for day-to-day automation, and intentional about validating whether a CLI-first interface can be more efficient than a token-expensive MCP-centered workflow.
