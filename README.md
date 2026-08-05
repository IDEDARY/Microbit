# Bevy Microbit

This workspace runs a Bevy game on a BBC micro:bit V1 (nRF51822) using a
`no_std` backend built on the `bevy_ecs`, `bevy_app` and `bevy_time` crates.

## Layout

* `bevy_microbit/` — the library crate. Every piece of Microbit hardware
  interaction is hidden behind a Bevy-idiomatic API: `App`, `Plugin`, the
  `Startup`/`PreFrame`/`Update` schedules, `Res<Time>`, `Res<ButtonInput>`,
  and a hardware-agnostic `ResMut<FrameBuffer>`.
* `examples/game/` — the ported debris-dodging game, written as a plain desktop
  Bevy app. It imports `bevy_microbit::prelude::*` and nothing Microbit-specific.

## Sizing (release, `thumbv6m-none-eabi`)

* Flash: ~229 KiB of 256 KiB.
* RAM: ~6 KiB (heap) + ~40 B static, leaving room for the stack in 16 KiB.

## Build & run with `probe-rs`

```sh
cargo build --release
cargo embed --release
```

## Fitting Bevy on 16 KiB of RAM

| BBC Micro:bit | V1              | V2              |
| ------------- | --------------- | --------------- |
| Processor     | Nordic nRF51822 | Nordic nRF52833 |
| Flash memory  | 256 KB          | 512 KB          |
| RAM           | 16 KB           | 128 KB          |
| Speed         | 16 MHz          | 64 MHz          |

| Bevy 0.19    | `full no-std` | `ecs + app + time` |
| ------------ | ------------- | ------------------ |
| Flash memory | 339 KB        | 58 KB              |
| RAM          | ~ 2.5 KB      | ~ 2.5 KB           |

The full `bevy` crate is far too large for the microbit v1, largely because
`bevy_internal` unconditionally enables `bevy_reflect`. Depending on
`bevy_ecs`/`bevy_app`/`bevy_time` directly keeps the whole app
under 256 KiB flash and ~6 KiB RAM.
