# Cosmic Notifications

Layer Shell notifications daemon which integrates with COSMIC.

> **This fork** ([jym272/cosmic-notifications](https://github.com/jym272/cosmic-notifications))
> tracks upstream `master` (what Pop!_OS actually ships) and carries our customizations.
>
> - **Start here (humans and agents):** [CLAUDE.md](CLAUDE.md) — architecture summary, gotchas,
>   and the mandatory issue → PR → review → authorized-merge workflow.
> - **Internals:** [docs/architecture.md](docs/architecture.md)
> - **Using the daemon from other tools (D-Bus contract + runnable scripts):**
>   [docs/contract.md](docs/contract.md)
>
> Quick start on Pop!_OS 24.04 (deps below already satisfied by a stock install + rustup + just):
> `just build-release`, test with
> `pkill -x -f cosmic-notifications; RUST_LOG=info ./target/release/cosmic-notifications`
> (plain `pkill cosmic-notifications` never matches — see CLAUDE.md), deploy with
> `sudo $(which just) deploy` (then `sudo apt-mark hold cosmic-notifications` so updates don't
> overwrite it).

# Building

Cosmic Notifications is set up to build a deb and a Nix flake, but it can be built using just.

Some Build Dependencies:
```
  cargo,
  just,
  intltool,
  appstream-util,
  desktop-file-utils,
  libxkbcommon-dev,
  pkg-config,
  desktop-file-utils,
```

## Build Commands

For a typical install from source, use `just` followed with `sudo just install`.
```sh
just
sudo just install
```

If you are packaging, run `just vendor` outside of your build chroot, then use `just build-vendored` inside the build-chroot. Then you can specify a custom root directory and prefix.
```sh
# Outside build chroot
just clean-dist
just vendor

# Inside build chroot
just build-vendored
sudo just rootdir=debian/cosmic-notifications prefix=/usr install
```

# Debugging & Profiling

## Profiling async tasks with tokio-console

To debug issues with asynchronous code, install [tokio-console](https://github.com/tokio-rs/console) and run it within a separate terminal. Then kill the **cosmic-notifications** process a couple times in quick succession to prevent **cosmic-session** from spawning it again. Then you can start **cosmic-notifications** with **tokio-console** support either by running `just tokio-console` from this repository to test code changes, or `env TOKIO_CONSOLE=1 cosmic-notifications` to enable it with the installed version of **cosmic-notifications**.
