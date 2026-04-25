# Engine Architecture

Brickbyte uses a custom engine built on top of `winit` (for windowing), `glow` (for OpenGL bindings), and `glam` (for 3D math). 

## The Game Loop

The application entry point is in `main.rs`, which initializes a `Brickbyte` struct. We use `winit`'s event loop to catch window events, keyboard/mouse inputs, and to request redraws.

The actual heavy lifting is delegated to the `GameState` (`gamestate.rs`):

1. **Input:** Captures device events (mouse motion for camera) and window events (clicks for breaking/placing blocks).
2. **Update:** Calculates delta time, updates the player's position and physics.
3. **Render:** Clears the screen, calculates the View-Projection matrix, draws the world chunks, and overlays the `egui` user interface.

## Coordinate System

Brickbyte uses a standard 3D coordinate system via `glam`:

* **Y-axis:** Up/Down (Vertical)
* **X-axis / Z-axis:** The horizontal plane.
