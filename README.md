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
* Play/Pause:  spacebar
* Quit:        q
* Volume up:   up (max 100%)
* Volume down: down (min 0%)
```

## Install

Make sure you have the latest `cargo` toolchain [installed](https://www.rust-lang.org/tools/install).

Then,

```
cargo install --git https://github.com/ngmiller/scli.git --bin scli
```

See `How?` above for running and controls.
