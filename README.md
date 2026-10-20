# NexusRun Studio (desktop)

A small Tauri shell around the `nexus` engine. It starts `nexus studio --workspace ~/NexusRun`,
reads the local address the engine prints, and shows it in a native window. Closing the window
stops the engine.

## Build

1. Build the engine for your platform and place it in `src-tauri/binaries/` named
   `nexus-<target-triple>` (add `.exe` on Windows), e.g. `nexus-x86_64-unknown-linux-gnu`.
2. `cd src-tauri && cargo build --release`

Linux needs `libwebkit2gtk-4.1-dev`, `libgtk-3-dev` and `librsvg2-dev`.
