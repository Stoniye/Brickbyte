# Player Mechanics

This document describes the implementation of the player controller, including movement physics, collision detection, and camera management.

## Player State

The player's state is managed in `src/world/player.rs` via the `Player` struct. Key attributes include:

* **Position (`pos`):** A `Vec3` representing the player's current coordinates in the world.
* **Orientation:** Controlled via `yaw` and `pitch` values, which are converted into a `camera_front` vector for rendering.
* **Physics State:** Includes `vertical_velocity` for jumping/falling and `was_grounded` to handle jump resets.

## Movement & Controls

The player movement is calculated in `update_pos` and is influenced by keyboard input captured in `GameState`.

### Keyboard Mapping

| Key          | Action                                     |
| ------------ | ------------------------------------------ |
| `W` / `S`    | Move Forward / Backward                    |
| `A` / `D`    | Strafe Left / Right                        |
| `Space`      | Jump                                       |
| `Left Shift` | Sprint (Increases speed from 4 to 6 units) |

### Movement Constants

These values are defined in `player.rs` to tune the "feel" of the game:

* **Speed:** `4` (Walk), `6` (Sprint)
* **Gravity:** `23` units/s²
* **Jump Strength:** `8` units/s
* **Player Height:** `1.8` blocks

## Camera System

The camera uses a First-Person perspective. 

* **Rotation:** Mouse movement updates the `yaw` and `pitch`. 
* **Clamping:** The `pitch` is clamped between `-89.0` and `89.0` degrees to prevent the camera from flipping over at the poles.
* **View Matrix:** The `camera_front` vector is used by the renderer to calculate the View-Projection matrix, ensuring the world is rendered from the player's eyes.

## Physics & Collision Detection

Brickbyte uses a simplified AABB (Axis-Aligned Bounding Box) collision system.

### Gravity

Every frame, `vertical_velocity` is decreased by the `GRAVITY` constant multiplied by delta time. If the player is on the ground, this velocity is reset.

### Collision Logic

Before moving the player, the system checks for block collisions using the `is_block_at` helper:

1. **Foot Level:** Checks if there is a solid block at the player's feet.
2. **Head Level:** Checks if there is a solid block at the player's head height (`1.5` units above feet).
3. **Horizontal Clipping:** If a movement on the X or Z axis would result in the player being inside a block, that specific axis movement is zeroed out.

## World Interaction

Interaction is handled in `gamestate.rs` using a **Raycasting** system.

* **Left Click:** Triggers a raycast from the player's eye position in the direction of `camera_front`. If it hits a block within range, that block is removed (set to 0).
* **Right Click:** Uses the same raycast but identifies the *previous* empty air position before the hit to place a new block.
* **Hotbar:** The player can switch between block types using the **Mouse Wheel**. The currently selected block is highlighted in the UI and used during placement.
