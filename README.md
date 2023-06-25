# Lights Out Game and Solver

An implementation of the [Lights Out](https://en.wikipedia.org/wiki/Lights_Out_(game)) game and solver.

## Game Rules
The Game consists of a square grid of lights. Each light can be pressed. Doing so toggles it and the 4 adjacent lights individually. The goal is to switch on all lights.

## Controls
Besides clicking on the lights there are four controls on the right of the screen.
1. ### Reset
    Resets all lights and switches to their off position.
2. ### Field size entry
    Allows specifying the size of the game.
3. ### Solve
    Shows the lights to press to activate all lights.
4. ### Toggle click action
    ### Switch activations on click: 
    Toggles the clicked light and the ones surrounding it according to the games rules.
    ### Switch lights on click:
    Toggles the individual clicked light without the surrounding ones. This mode allows recreating game states to play or solve them.

## Build from source
The game is written entirely in [Rust](https://www.rust-lang.org/), which makes it :rocket:blazingly fast:rocket:. Building it is incredibly easy if Rust is already available:
1. Open terminal in the root of the project directory
2. Run `cargo build --release`
3. The executable can be found at `./target/release/lights-out.<exectuable file extention>`

If Rust is not available it can be installed with very little effort by [following the official instruction here](https://www.rust-lang.org/tools/install).

## About

This project was created to solve puzzles in the game [Heroes of Hammerwatch](https://store.steampowered.com/app/677120/Heroes_of_Hammerwatch/) and gain experience with [Slint](https://slint.dev/) UI development.