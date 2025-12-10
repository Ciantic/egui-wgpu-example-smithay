# Wayapp

No winit was used during creation of this thing.

This repository aims to not use cross-platform libraries for handling windows, instead it uses just wayland APIs via Smithay's libraries. If you target just Linux then adding cross-platform overhead is not necessary.

## Examples

```bash
cargo run --example egui_example --features egui-wgpu
cargo run --example egui_layer_shell_example.rs --features egui-wgpu
cargo run --example iced_example --features iced-wgpu
cargo run --example single-color-buffer
```

## EGUI

Currently uses only EGUI WGPU rendering.

## ICED

This is not yet implemented, plan is to integrate first iced-wgpu.

## Future changes

Maybe change `Application` to hold only `Weak` references to the `WindowContainer`/`LayerSurfaceContainer`/`PopupContainer`/`SubsurfaceContainer`, because it's not the responsibility of the `Application` to keep those alive, it's the responsibility of the main.
