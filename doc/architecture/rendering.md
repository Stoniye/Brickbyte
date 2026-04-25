We do not render every single block. Instead, `world.rs` constructs a mesh containing only the visible exterior faces of the blocks.

* Textures are pulled from a single Texture Atlas (BLOCK_ATLAS).

* A custom Vertex and Fragment shader (vertex.glsl, fragment.glsl) handles basic shading and texture mapping.
