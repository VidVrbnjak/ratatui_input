ratatui_input
============
[![crate][crates-io-badge]][crate]
[![docs][doc-badge]][doc]
[![CI][ci-badge]][ci]
[![coverage][codecov-badge]][codecov]

[ratatui_input][crate] is a simple input widget like `<input />` in HTML for [ratatui][].

**UNDER CONSTRUCTION**
This crate is still under heavy construction and not ready or use yet

**Features**

- Single line input with baisc operations (insert/delet characters, copy, cut, jumps, ...)
- Windows style shortcuts (`Ctrl-v`, `Ctrl-v`, `Ctrl-x`, `Home`, `End`)
- Text selection
- Does not need terminal cursor capture

**Comming soon**
- Backend agnostic
- Undo/Redo
- Masking
- Autoscroll

[Documentation][doc]

## Exmaple
Running `cargo run --example` in this repository can demonstrate usage of ratatui_input
`TODO: Upload a GIF of the exmaple running`

## Instalation
Add `ratatui_input` crate to dependecies in your `Cargo.toml`

```toml
[dependecies]
ratatui = "*"
ratatui_input = "*"
```

## Key mappings


| Mappings                           | Description                                           |
| -----------------------------------|------------------------------------------------------ |
| `→`                                | Move cursor forawrd by one character                  |
| `←`                                | Move cursor back by one character                     |
| `Shift + →`                        | Select under cursor and move forawrd by one character |
| `Shift + ←`                        | Select under cursor and move back by one character    |
| `Ctrl+C`                           | Copy selected text or whole input                     |
| `Ctrl+V`                           | Replace selected text or insert at cursor             |
| `Ctrl+X`                           | Cut selected text or whole input                      |
| `Home`                             | Jump to start                                         |
| `End`                              | Jump to end                                           |
| `Shift+Home`                       | Select from cursor to start                           |
| `Shift+End`                        | Select from cursor to end                             |
| `Backspace`                        | Delete character before cursor                        |
| `Delete`                           | Delete character under cursor                         |
| `Insert`                           | Toggle insert mode                                    |
| `TODO:` `Ctrl+A`                   | Select everything                                     |
| `TODO:` `Ctrl+W`                   | Select current word                                   |

## License

[ratatui_input][] is distributed under [The MIT License](./LICENSE.txt).

[ratatui]: https://github.com/ratatui-org/ratatui
[crates-io-badge]: https://img.shields.io/crates/v/ratatui_input.svg
[crate]: https://crates.io/crates/ratatui_input
[doc-badge]: https://docs.rs/ratatui_input/badge.svg
[doc]: https://docs.rs/ratatui_input/latest/ratatui_input
[ci-badge]: https://github.com/VidVrbnjak/ratatui_input/actions/workflows/rust.yml/badge.svg?event=push
[ci]: https://github.com/VidVrbnjak/ratatui_input/actions/workflows/rust.yml
[codecov-badge]: https://codecov.io/gh/VidVrbnjak/ratatui_input/graph/badge.svg?token=643UEHDO13
[codecov]: https://codecov.io/gh/VidVrbnjak/ratatui_input
[crossterm]: https://docs.rs/crossterm/latest/crossterm/
[termion]: https://docs.rs/termion/latest/termion/
[termwiz]: https://docs.rs/termwiz/latest/termwiz/
[ratatui-backend]: https://docs.rs/ratatui/latest/ratatui/backend/trait.Backend.html
[repo]: https://github.com/VidVrbnjak/ratatui_input
