//! The deferred despawn queue held by the `World`.

use heapless::Vec as HVec;

use crate::entity::Entity;

/// Maximum number of deferred operations queued per flush.
///
/// Sized comfortably for the debris game; bump if a system ever needs more.
pub const COMMAND_CAPACITY: usize = 64;

/// A single deferred world mutation.
#[derive(Debug)]
pub enum Command {
    /// Despawn an entity, removing it from every column.
    Despawn(Entity),
}

/// A bounded queue of pending [`Command`]s awaiting flush.
#[derive(Default)]
pub struct CommandBuffer {
    /// The pending commands, in insertion order.
    queue: HVec<Command, COMMAND_CAPACITY>,
}

impl CommandBuffer {
    /// Creates an empty buffer.
    pub const fn new() -> Self {
        Self {
            queue: HVec::new(),
        }
    }

    /// Pushes a command; saturates silently if the buffer is full.
    pub fn push(&mut self, cmd: Command) {
        let _ = self.queue.push(cmd);
    }

    /// Drains the buffer, yielding pending commands.
    ///
    /// Order is LIFO; despawns are independent of one another so this is safe.
    pub fn drain(&mut self) -> Drain<'_> {
        Drain { buf: self }
    }

    /// Returns `true` when no commands are pending.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/// Owning drain iterator that pops pending commands until the buffer empties.
pub struct Drain<'a> {
    /// The owning buffer.
    buf: &'a mut CommandBuffer,
}

impl Iterator for Drain<'_> {
    type Item = Command;
    fn next(&mut self) -> Option<Command> {
        self.buf.queue.pop()
    }
}