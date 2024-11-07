# Tauri + React

This template should help get you started developing with Tauri and React in Vite.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Ubuntu Additional Setup
Run `unset GIO_MODULE_DIR` and `unset GTK_PATH` before usage 

## Command to start
npm run tauri dev

## Setting up cargo run file path
When trying to upload a file we use a cargo run in the rust code to execute the `DevilArm` library in order to properly execute this verify that the file path in `lib.rs` is correct it currently is under the assumption that you are running from `DevilArmTrainer/src-tauri` and goes back to Github and navigates to `src` in `devil-embedded`

You will need to check that this `let directory = "../../DevilArm/devil-embedded/src";` is correct