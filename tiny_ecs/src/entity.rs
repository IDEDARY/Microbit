//! Lightweight entity identifiers.
//!
//! An `Entity` is just a `u32` index into the `World`'s columns. There is no
//! generation counter: holding an `Entity` after it has been despawned and the
//! index recycled is documented undefined behaviour, traded for the storage
//! savings of skipping per-entity metadata (acceptable for a micro:bit game
//! that despawns only at well-defined frame boundaries).

/// A handle to an entity in the world.
///
/// Obtained from `World::spawn` / `Commands::spawn`. The wrapped `u32` is the
/// entity's index into every component column; it is `Copy` and `Eq`/`Hash` so
/// it can be used as a map key or compared directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity(
    /// The raw index.
    pub u32,
);

impl Entity {
    /// Creates an entity from a raw index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw index.
    pub const fn index(self) -> u32 {
        self.0
    }
}