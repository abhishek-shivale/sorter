# sorter

macOS background daemon that sorts downloaded files into category subfolders of one directory.

```
~/Downloads/report.PDF  ->  ~/Downloads/Documents/report.PDF
~/Downloads/shot.png    ->  ~/Downloads/Images/shot.png
```

Categories: `Images`, `Videos`, `Audio`, `Documents`, `Archives`. Files with any other extension stay where they are.

## Install

Requires macOS. Installs the latest release to `~/.local/bin`:

```sh
curl -fsSL https://raw.githubusercontent.com/abhishek-shivale/sorter/main/install.sh | sh
```

From source, with Rust 1.90 or newer: `cargo install --path .`

`sorter start` records the path of the binary that ran it. Run it from a permanent location, not `target/`, because `cargo clean` would stop the daemon launching.

## Use

```sh
sorter set --dir ~/Downloads
sorter start
sorter stop
```

`start` sorts everything already in the directory on the first pass, then checks again every 60 seconds. The daemon restarts after a crash and at login until you run `stop`.

macOS protects `~/Downloads`, so the daemon may need access in System Settings > Privacy & Security. `start` reports success even if access is denied, so check the log after starting. Denied access shows `Operation not permitted`.

## Behaviour

- Never overwrites: a name clash becomes `report (1).pdf`.
- Skips hidden files, folders and symlinks.
- Skips files modified in the last 60 seconds, and files with a matching `.part` file, because they may still be downloading.
- Only looks at the top level of the directory.
- Refuses the home directory and its parents.

## Files

| Path | Purpose |
|---|---|
| `~/.config/sorter/config.toml` | Configuration |
| `~/.local/share/sorter/sorter.log` | Log of moves and errors |
| `~/Library/LaunchAgents/io.github.abhishek-shivale.sorter.plist` | launchd agent, present while started |

Configuration, reloaded on every pass:

```toml
url = "/Users/you/Downloads"
ignore = ["zip", "pdf"]  # optional: extensions to leave in place
```

## Uninstall

```sh
sorter stop
rm ~/.local/bin/sorter
rm -r ~/.config/sorter ~/.local/share/sorter
```

Sorted files stay where they are.
