//! Deferred and synchronous world mutations exposed to systems via
//! [`Commands`].
//!
//! `Commands` carries a raw `*mut W` (set by the `#[system]` macro) plus the
//! world's [`CommandBuffer`] for deferred despawns. Because the concrete
//! `World` implements [`SpawnRef<T>`](crate::system::SpawnRef) and
//! [`ResourceInsRef<R>`](crate::system::ResourceInsRef) per registered
//! component/resource, `Commands::spawn::<T>(value)` and
//! `insert_resource::<R>(value)` enqueue typed operations into *distinct*
//! `World` fields — disjoint from any column a `Query` is currently borrowing
//! immutably — so spawning during a read-only query system stays sound.
//! The pending values are drained into the columns by `World::flush_commands`,
//! called between systems.

use core::marker::PhantomData;

use crate::commands_buffer::{Command, CommandBuffer};
use crate::entity::Entity;
use crate::system::{ResourceInsRef, SpawnRef};

/// A handle systems use to spawn entities, insert resources, and defer
/// despawns, mirroring Bevy's `Commands`.
pub struct Commands<'a, W> {
    /// Raw pointer to the owning concrete `World`.
    world: *mut W,
    /// Borrowed command buffer (for despawns).
    buffer: &'a mut CommandBuffer,
    /// Captures the borrow of the buffer.
    _life: PhantomData<&'a mut CommandBuffer>,
}

impl<'a, W> Commands<'a, W> {
    /// Creates a `Commands` from a raw world pointer and a buffer borrow.
    ///
    /// # Safety
    /// `world` must be valid for the system's duration and distinct from any
    /// column borrow currently held by the same system.
    pub unsafe fn new(world: *mut W, buffer: &'a mut CommandBuffer) -> Self {
        Self {
            world,
            buffer,
            _life: PhantomData,
        }
    }

    /// Enqueues a spawn of `value` of component type `T`, returning the new
    /// entity id. The component is actually inserted into its column on the
    /// next [`World::flush_commands`](crate::world::WorldApi::flush_commands).
    pub fn spawn<T: 'static>(&mut self, value: T) -> Entity
    where
        W: SpawnRef<T>,
    {
        // SAFETY: `world` is valid; `SpawnRef` mutates only the pending queue
        // for `T`, which is disjoint from any column borrowed by this system.
        unsafe { W::enqueue_spawn(self.world, value) }
    }

    /// Synchronously inserts `value` as the resource `R`.
    pub fn insert_resource<R: 'static>(&mut self, value: R)
    where
        W: ResourceInsRef<R>,
    {
        // SAFETY: `world` is valid; `ResourceInsRef` writes only the `R` slot.
        unsafe { W::insert_resource(self.world, value) }
    }

    /// Queues the despawn of `entity`, mirrored by Bevy's
    /// `commands.entity(e).despawn()`.
    pub fn entity(&mut self, entity: Entity) -> EntityTarget {
        self.buffer.push(Command::Despawn(entity));
        EntityTarget { entity }
    }
}

/// A fluent handle returned by [`Commands::entity`] for chained despawns.
#[derive(Debug, Clone, Copy)]
pub struct EntityTarget {
    /// The entity being targeted.
    pub entity: Entity,
}

impl EntityTarget {
    /// Marks the entity for despawn (already queued by [`Commands::entity`]).
    pub fn despawn(&self) {
        // The despawn was already queued when `Commands::entity` was called;
        // this exists to mirror Bevy's `commands.entity(e).despawn()` chain.
    }
}