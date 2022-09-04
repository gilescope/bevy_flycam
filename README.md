# bevy_flycam
[![Crates.io](https://img.shields.io/crates/v/bevy_flycam)](https://crates.io/crates/bevy_flycam)
![Crates.io](https://img.shields.io/crates/l/bevy_flycam)
![docs.rs](https://img.shields.io/docsrs/bevy_flycam)


A not so basic first-person fly camera for Bevy 0.8 - this is a fork rounding out the features of the original.
(pretty sure many of the things done here arn not wanted in upstream.)

## Controls
* WASD to move horizontally
* SPACE to ascend
* LSHIFT to run
* ESC to grab/release cursor. (for wasm you'll have to link up to a button - see examples/scrolling)

## Comparison
There are a few notable differences from [bevy_fly_camera](https://github.com/mcpar-land/bevy_fly_camera)...

* No linear interpolation
* Cursor grabbing
* Shorter code
* Single-line setup
* A tiny bit faster?

and differences from upstream:

* Scroll moves the camera forwards / backwards
* run (shift) and slow motion (s)
* wasm support
* hold down mouse button allows movement even if cursor not grabbed. 

## Usage
1. Add to `Cargo.toml` or copy `lib.rs` to your own file
```toml
[dependencies]
bevy = "0.8"
bevy_flycam = "*"
```

or

```toml
[dependencies]
bevy = "0.8"
bevy_flycam = { git = "https://github.com/sburris0/bevy_flycam" }
```

2. Include the `PlayerPlugin`
```rust
use bevy_flycam::PlayerPlugin;
```
This will spawn a camera for you. 
Use `NoCameraPlayerPlugin` if you do not want this and make sure to use `.insert(FlyCam)` on your own camera or else this plugin won't know what to move.

3. Add the `PlayerPlugin`:
```rust
#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .run();
}
```

Alternatively you can see the example `basic.rs` or `scroll.rs` located in the examples folder.
You can run the example by cloning this repository and run the command: `cargo run --release --example basic`

## Customization
To modify player movement speed or mouse sensitivity, import `bevy_flycam::MovementSettings` and add it as a resource:
```Rust
#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 12.0, // default: 12.0
        })
        .run();
}
```

# Support
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

bevy_flycam's crate version follows bevy's X version as shown:
|bevy|bevy_flycam|
|---|---|
|0.X.Y|0.X|

## Contributing
PRs are very welcome.
