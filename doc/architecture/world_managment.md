# World Generation & Chunks

The world is divided into manageable pieces called **Chunks**. This allows us to only render and update the parts of the world the player is currently near.

## Chunk Dimensions

Chunks in Brickbyte are fixed in size, defined in `chunk.rs`:

* **Width (X):** 16 blocks (`CHUNK_DIMENSION`)
* **Depth (Z):** 16 blocks (`CHUNK_DIMENSION`)
* **Height (Y):** 208 blocks (`CHUNK_HEIGHT`)

Because chunks span the entire vertical height of the world, we index them using a 2D vector (`IVec2`) in the `World`'s `HashMap`.

## Data Storage

Inside a `Chunk`, block data is stored in a flat 1D array (`Vec<u8>`). To convert a 3D local coordinate `(x, y, z)` to a 1D index, we use:

```rust
index = x + z * CHUNK_DIMENSION + y * (CHUNK_DIMENSION * CHUNK_DIMENSION)
```
