scli
====

Stream [SoundCloud](https://soundcloud.com)... in your terminal!

![screenshot](./docs/screenshot.png)

## How?

`scli` just needs a SoundCloud URL to start playing a track. :fire:

```
$ scli https://soundcloud.com/trippycode/boris-brejcha-art-of-minimal-techno-tripping-the-mad-doctor-by-rttwlr
```

**Please note, this interacts with SoundCloud's public API, and therefore does not support subscription content.**

### Controls

```
* Play/Pause: spacebar
* Quit:       q
```

## Install

Make sure you have the latest `cargo` toolchain [installed](https://www.rust-lang.org/tools/install).

Then,

```
cargo install --git https://github.com/ngmiller/scli.git --bin scli
```

Set your SoundCloud OAuth token as an environment variable. Ideally inside your bash/terminal configuration
so you don't leak it in your shell history.

```
export SC_TOKEN=<oauth token here>
```

See `How?` above for running and controls.
