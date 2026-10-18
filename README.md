# Fly Away, Fly!

A small endless flyer made with [Bevy](https://bevy.org). You're a crow in a
forest that won't stop scrolling. Hold to climb, let go to drop, stay clear of
the owls and the branches, and eat as many grapes as you can on the way.

## Controls

- **Hold Space, the left mouse button, or a finger on the screen** to fly up.
- **Escape** pauses and resumes.

## Running it

```sh
cargo run --release
```

This uses Bevy 0.20, which needs Rust 1.97.1 or newer (`rustup update` if
`cargo` complains about the version). On Linux you'll also want the
usual Bevy system packages; on Debian/Ubuntu that's

```sh
sudo apt install pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
```

## Building for the web

The browser build renders with **WebGPU**, so it needs a browser that has it:
current Chrome, Edge or Firefox, or Safari 26+. The page says so if it isn't
available.

One-time setup:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129   # must match Cargo.lock
```

[binaryen](https://github.com/WebAssembly/binaryen)'s `wasm-opt` is optional,
but if it's on your PATH the build uses it to shrink the download.

Then:

```sh
./web/build.sh
python3 -m http.server --directory web/dist 8080
```

and open <http://localhost:8080>. WebGPU only works over https or on
localhost, so opening `index.html` straight from disk won't work. Upload the
contents of `web/dist` to any static host to put it online.

## How it's put together

Each file in `src/` handles one part of the game and registers its own
systems as a small plugin:

- `main.rs`: app setup, the `GameState` machine and system ordering
- `run.rs`: starting a run, distance scoring, pausing
- `player.rs`: the crow's flight and animation
- `obstacles.rs`: owls, branches, and the scrolling everything else shares
- `grapes.rs`: grapes and eating them
- `background.rs`: the looping forest backdrop
- `hud.rs`, `menu.rs`: in-game readouts and all the menus
- `audio.rs`: music switching and sound effects
- `save.rs`: saves to a JSON file, or localStorage in the browser
- `display.rs`: window, camera, UI scaling and fullscreen
- `animation.rs`, `assets.rs`, `collision.rs`: shared bits

Everything is laid out for a 1280x720 screen. The camera and UI scale from
there, so the game looks the same at any window size.

Saves live in `~/.local/share/flyaway/save.json` on Linux,
`~/Library/Application Support/flyaway/` on macOS and `%APPDATA%\flyaway\` on
Windows.

## Credits

A game by Silly Crow Studios.
Music by [FarBeyond Studio](https://farbeyond-studio.carrd.co/).
