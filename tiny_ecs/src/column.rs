//! Bounded, entity-indexed component storage.
//!
//! Each `Component` type owns exactly one [`Column<T, N>`]: a fixed
//! `[Option<T>; N]` indexed by entity id, with a free-list managed centrally by
//! the `World`. There are no archetypes and no sparse sets — iteration simply
//! walks the slots and yields the populated ones.

/// A single component column: `N` slots, each `Option<T>`, indexed by entity id.
///
/// `N` is a per-component compile-time capacity exported via the const generic,
/// so each component type can be given just the budget it needs.
pub struct Column<T, const N: usize> {
    /// The per-entity slots; `slots[i]` is `Some` when entity `i` has `T`.
    slots: [Option<T>; N],
}

impl<T, const N: usize> Column<T, N> {
    /// Creates a column where every slot is empty.
    pub const fn new() -> Self {
        Self {
            slots: [const { None }; N],
        }
    }

    /// Returns the compile-time capacity.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns an immutable reference to the component at `index`, if present.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < N {
            self.slots[index].as_ref()
        } else {
            None
        }
    }

    /// Returns a mutable reference to the component at `index`, if present.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < N {
            self.slots[index].as_mut()
        } else {
            None
        }
    }

    /// Inserts `value` at `index`, replacing any existing component and
    /// returning the previous one.
    pub fn insert(&mut self, index: usize, value: T) -> Option<T> {
        if index >= N {
            return None;
        }
        self.slots[index].replace(value)
    }

    /// Removes the component at `index`, if present.
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index < N {
            self.slots[index].take()
        } else {
            None
        }
    }
}

impl<T, const N: usize> Default for Column<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Object-safe operations shared by every component column, used so queries
/// can hold a single `*const dyn ColumnOps<T>` regardless of the column's
/// concrete capacity `N`.
pub trait ColumnOps<T> {
    /// Returns the column's capacity.
    fn capacity(&self) -> usize;
    /// Borrows the component at `index`, if present.
    fn get(&self, index: usize) -> Option<&T>;
    /// Mutably borrows the component at `index`, if present.
    fn get_mut(&mut self, index: usize) -> Option<&mut T>;
    /// Inserts `value` at `index`, returning any previous component.
    fn insert(&mut self, index: usize, value: T) -> Option<T>;
    /// Removes the component at `index`, if present.
    fn remove(&mut self, index: usize) -> Option<T>;
}

impl<T, const N: usize> ColumnOps<T> for Column<T, N> {
    fn capacity(&self) -> usize {
        N
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.get_mut(index)
    }

    fn insert(&mut self, index: usize, value: T) -> Option<T> {
        self.insert(index, value)
    }

    fn remove(&mut self, index: usize) -> Option<T> {
        self.remove(index)
    }
}