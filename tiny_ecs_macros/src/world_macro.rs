//! Parser and code-generator for the `define_world!` proc-macro.
//!
//! Syntax (see crate docs):
//!
//! ```ignore
//! define_world! {
//!     pub struct World {
//!         label: GameSchedule,
//!         entities: 64,
//!         schedules: 8,
//!         components {
//!             player: Player [64],
//!             debris: Debris [64],
//!         }
//!         resources {
//!             frame: FrameBuffer,
//!             time: Time,
//!         }
//!     }
//! }
//! ```
//!
//! The emitted `World` is a concrete struct generic over the schedule label
//! type `L`, with one field per component column (`Column<T, N>`) and per
//! resource (`Option<R>`), plus the entity free-list, the bounded schedule
//! map, and the command buffer. The companion `ColumnRef`/`ResourceRef`/
//! `ResourceInsRef`/`SpawnRef`/`CommandsRef` and `WorldApi` impls are
//! generated so the `#[system]` macro can split borrows and the app can drive
//! schedules.

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    Token,
};
// Note: `tiny_ecs_macros` is a `proc-macro = true` crate, so it cannot depend
// on `tiny_ecs` for sharing types; the generated code references `::tiny_ecs`
// by absolute path, which resolves in the app crate.

/// One `components { ... }` entry: `field: Type [capacity]`.
struct ComponentEntry {
    /// The snake_case field name on the `World` struct.
    field: syn::Ident,
    /// The component type.
    ty: syn::Type,
    /// The column capacity (`N` const generic).
    capacity: syn::Expr,
}

/// One `resources { ... }` entry: `field: Type`.
struct ResourceEntry {
    /// The snake_case field name on the `World` struct.
    field: syn::Ident,
    /// The resource type.
    ty: syn::Type,
}

/// The whole `define_world!` input.
pub(crate) struct WorldInput {
    /// Visibility of the generated `World` struct.
    vis: syn::Visibility,
    /// The `World` type name to emit.
    name: syn::Ident,
    /// The schedule label type used as the map key.
    label: syn::Type,
    /// The maximum number of live entities.
    entities: syn::Expr,
    /// The maximum number of schedules.
    schedules: syn::Expr,
    /// Registered component columns.
    components: Vec<ComponentEntry>,
    /// Registered resources.
    resources: Vec<ResourceEntry>,
}

impl Parse for WorldInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: syn::Visibility = input.parse()?;
        let _kw: Token![struct] = input.parse()?;
        let name: syn::Ident = input.parse()?;
        let body;
        syn::braced!(body in input);

        let mut label: Option<syn::Type> = None;
        let mut entities: Option<syn::Expr> = None;
        let mut schedules: Option<syn::Expr> = None;
        let mut components: Vec<ComponentEntry> = Vec::new();
        let mut resources: Vec<ResourceEntry> = Vec::new();

        while !body.is_empty() {
            // Distinguish `components {` / `resources {` from `key: value`.
            if body.peek(syn::Ident) && body.peek2(syn::token::Brace) {
                let section: syn::Ident = body.parse()?;
                if section == "components" {
                    components = parse_component_entries(&body)?;
                } else if section == "resources" {
                    resources = parse_resource_entries(&body)?;
                } else {
                    return Err(syn::Error::new(
                        section.span(),
                        "expected `components` or `resources`",
                    ));
                }
                let _ = body.parse::<Token![,]>();
                continue;
            }

            let key: syn::Ident = body.parse()?;
            let _colon: Token![:] = body.parse()?;
            if key == "label" {
                label = Some(body.parse()?);
            } else if key == "entities" {
                entities = Some(body.parse()?);
            } else if key == "schedules" {
                schedules = Some(body.parse()?);
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "unknown field; expected `label`, `entities`, `schedules`, `components`, or `resources`",
                ));
            }
            let _ = body.parse::<Token![,]>();
        }

        let label = label
            .ok_or_else(|| syn::Error::new(name.span(), "missing `label: <ScheduleLabel>`"))?;
        let entities = entities
            .ok_or_else(|| syn::Error::new(name.span(), "missing `entities: <MAX>`"))?;
        let schedules = schedules
            .ok_or_else(|| syn::Error::new(name.span(), "missing `schedules: <MAX>`"))?;

        Ok(WorldInput {
            vis,
            name,
            label,
            entities,
            schedules,
            components,
            resources,
        })
    }
}

/// Parses `field: Type [capacity]` entries inside a `components { ... }` block.
fn parse_component_entries(body: ParseStream) -> syn::Result<Vec<ComponentEntry>> {
    let inner;
    syn::braced!(inner in body);
    let mut out = Vec::new();
    while !inner.is_empty() {
        let field: syn::Ident = inner.parse()?;
        let _colon: Token![:] = inner.parse()?;
        let ty: syn::Type = inner.parse()?;
        let cap_content;
        syn::bracketed!(cap_content in inner);
        let capacity: syn::Expr = cap_content.parse()?;
        out.push(ComponentEntry {
            field,
            ty,
            capacity,
        });
        let _ = inner.parse::<Token![,]>();
    }
    Ok(out)
}

/// Parses `field: Type` entries inside a `resources { ... }` block.
fn parse_resource_entries(body: ParseStream) -> syn::Result<Vec<ResourceEntry>> {
    let inner;
    syn::braced!(inner in body);
    let mut out = Vec::new();
    while !inner.is_empty() {
        let field: syn::Ident = inner.parse()?;
        let _colon: Token![:] = inner.parse()?;
        let ty: syn::Type = inner.parse()?;
        out.push(ResourceEntry { field, ty });
        let _ = inner.parse::<Token![,]>();
    }
    Ok(out)
}

impl WorldInput {
    /// Emits the `World` struct and all associated trait impls.
    pub(crate) fn expand(&self) -> syn::Result<TokenStream2> {
        let WorldInput {
            vis,
            name,
            label,
            entities,
            schedules,
            components,
            resources,
        } = self;

        // Field declarations: one `Column<T, N>` per component.
        let comp_fields = components.iter().map(|c| {
            let f = &c.field;
            let t = &c.ty;
            let cap = &c.capacity;
            quote! { #f: ::tiny_ecs::column::Column<#t, #cap> }
        });
        // Resource fields: `Option<R>` (None until inserted).
        let res_fields = resources.iter().map(|r| {
            let f = &r.field;
            let t = &r.ty;
            quote! { #f: ::core::option::Option<#t> }
        });

        // Initialise columns and resources in `new()`.
        let comp_inits = components.iter().map(|c| {
            let f = &c.field;
            quote! { #f: ::tiny_ecs::column::Column::new() }
        });
        let res_inits = resources.iter().map(|r| {
            let f = &r.field;
            quote! { #f: ::core::option::Option::None }
        });

        // Per-component pending-spawn queue fields.
        let comp_pendings = components.iter().map(|c| {
            let pf = format_ident!("pending_{}", c.field);
            let t = &c.ty;
            let cap = &c.capacity;
            // Each spawn pairs an entity index with the component value.
            quote! { #pf: ::heapless::Vec<(u32, #t), #cap> }
        });
        let comp_pending_inits = components.iter().map(|c| {
            let pf = format_ident!("pending_{}", c.field);
            quote! { #pf: ::heapless::Vec::new() }
        });

        // `ColumnRef<T>` impls.
        let column_ref_impls = components.iter().map(|c| {
            let f = &c.field;
            let t = &c.ty;
            quote! {
                impl<L: ::tiny_ecs::system::ScheduleLabel>
                    ::tiny_ecs::system::ColumnRef<#t> for #name<L>
                {
                    unsafe fn col_ref_raw(world: *mut Self) -> *const dyn ::tiny_ecs::column::ColumnOps<#t> {
                        unsafe { &(*world).#f as &dyn ::tiny_ecs::column::ColumnOps<#t> as *const _ }
                    }
                    unsafe fn col_mut_raw(world: *mut Self) -> *mut dyn ::tiny_ecs::column::ColumnOps<#t> {
                        unsafe { &mut (*world).#f as &mut dyn ::tiny_ecs::column::ColumnOps<#t> as *mut _ }
                    }
                    unsafe fn col_capacity(world: *mut Self) -> usize {
                        (*world).#f.capacity()
                    }
                }
            }
        });

        // `ResourceRef<R>` impls.
        let resource_ref_impls = resources.iter().map(|r| {
            let f = &r.field;
            let t = &r.ty;
            quote! {
                impl<L: ::tiny_ecs::system::ScheduleLabel>
                    ::tiny_ecs::system::ResourceRef<#t> for #name<L>
                {
                    unsafe fn res_ref_raw(world: *mut Self) -> *const #t {
                        unsafe { (*world).#f.as_ref().map_or(::core::ptr::null(), |r| r as *const #t) }
                    }
                    unsafe fn res_mut_raw(world: *mut Self) -> *mut #t {
                        unsafe { (*world).#f.as_mut().map_or(::core::ptr::null_mut(), |r| r as *mut #t) }
                    }
                }
            }
        });

        // `ResourceInsRef<R>` impls.
        let resource_ins_impls = resources.iter().map(|r| {
            let f = &r.field;
            let t = &r.ty;
            quote! {
                impl<L: ::tiny_ecs::system::ScheduleLabel>
                    ::tiny_ecs::system::ResourceInsRef<#t> for #name<L>
                {
                    unsafe fn insert_resource(world: *mut Self, value: #t) {
                        unsafe { (*world).#f = ::core::option::Option::Some(value); }
                    }
                }
            }
        });

        // `SpawnRef<T>` impls (per-component pending queue).
        let spawn_ref_impls = components.iter().map(|c| {
            let pf = format_ident!("pending_{}", c.field);
            let t = &c.ty;
            quote! {
                impl<L: ::tiny_ecs::system::ScheduleLabel>
                    ::tiny_ecs::system::SpawnRef<#t> for #name<L>
                {
                    unsafe fn enqueue_spawn(world: *mut Self, value: #t) -> ::tiny_ecs::entity::Entity {
                        // SAFETY: caller guarantees `world` is valid.
                        let w = unsafe { &mut *world };
                        let idx = w.alloc_entity();
                        let _ = w.#pf.push((idx, value));
                        ::tiny_ecs::entity::Entity::new(idx)
                    }
                }
            }
        });

        // `CommandsRef` impl.
        let commands_ref_impl = quote! {
            impl<L: ::tiny_ecs::system::ScheduleLabel> ::tiny_ecs::system::CommandsRef for #name<L> {
                unsafe fn commands_raw(world: *mut Self) -> *mut ::tiny_ecs::commands_buffer::CommandBuffer {
                    unsafe { &mut (*world).commands as *mut _ }
                }
            }
        };

        // Despawn touches every column + clears pending queues.
        let comp_despawns = components.iter().map(|c| {
            let f = &c.field;
            quote! { self.#f.remove(idx as usize); }
        });
        // Flush inserts pending spawns into their columns.
        let comp_flushes = components.iter().map(|c| {
            let f = &c.field;
            let pf = format_ident!("pending_{}", c.field);
            quote! {
                while let ::core::option::Option::Some((idx, value)) = self.#pf.pop() {
                    self.#f.insert(idx as usize, value);
                }
            }
        });

        let expanded = quote! {
            /// Auto-generated `World` produced by `tiny_ecs::define_world!`.
            #vis struct #name<L: ::tiny_ecs::system::ScheduleLabel = #label> {
                /// Next entity index to hand out (monotonic until recycle).
                next_index: u32,
                /// Recycled entity ids available for reuse.
                free_list: ::heapless::Vec<u32, { #entities }>,
                /// Live entity count.
                alive: u32,
                #( #comp_fields, )*
                #( #comp_pendings, )*
                #( #res_fields, )*
                /// Bounded schedule registry, keyed by the app's label type.
                schedules: ::heapless::LinearMap<L, ::tiny_ecs::schedule::Schedule, { #schedules }>,
                /// Deferred despawn queue and pending-spawn drain buffer.
                commands: ::tiny_ecs::commands_buffer::CommandBuffer,
            }

            impl<L: ::tiny_ecs::system::ScheduleLabel> #name<L> {
                /// Creates an empty `World` with zero entities and no resources.
                #vis fn new() -> Self {
                    Self {
                        next_index: 0,
                        free_list: ::heapless::Vec::new(),
                        alive: 0,
                        #( #comp_inits, )*
                        #( #comp_pending_inits, )*
                        #( #res_inits, )*
                        schedules: ::heapless::LinearMap::new(),
                        commands: ::tiny_ecs::commands_buffer::CommandBuffer::new(),
                    }
                }

                /// Inserts a resource, replacing any previous instance.
                #vis fn insert_resource<R: 'static>(&mut self, resource: R)
                where
                    Self: ::tiny_ecs::system::ResourceInsRef<R>,
                {
                    let world_ptr: *mut Self = self as *mut Self;
                    // SAFETY: `world_ptr` is a valid `*mut Self`.
                    unsafe { <Self as ::tiny_ecs::system::ResourceInsRef<R>>::insert_resource(world_ptr, resource); }
                }

                /// Allocates a fresh entity id, reusing a recycled one when
                /// available; returns `None` when the entity budget is full.
                #vis fn alloc_entity(&mut self) -> u32 {
                    if let ::core::option::Option::Some(idx) = self.free_list.pop() {
                        self.alive += 1;
                        return idx;
                    }
                    if self.next_index >= #entities {
                        // Entity budget exhausted; reuse slot 0 defensively.
                        return 0;
                    }
                    let idx = self.next_index;
                    self.next_index += 1;
                    self.alive += 1;
                    idx
                }

                /// Inserts a component onto `entity`'s slot of column `T`.
                #vis fn set_component<T: 'static>(&mut self, entity: ::tiny_ecs::entity::Entity, value: T)
                where
                    Self: ::tiny_ecs::system::ColumnRef<T>,
                {
                    let world_ptr: *mut Self = self as *mut Self;
                    // SAFETY: bound guarantees the column exists; world_ptr valid.
                    let col = unsafe { <Self as ::tiny_ecs::system::ColumnRef<T>>::col_mut_raw(world_ptr) };
                    unsafe { (*col).insert(entity.index() as usize, value); }
                }
            }

            impl<L: ::tiny_ecs::system::ScheduleLabel> ::core::default::Default for #name<L> {
                fn default() -> Self {
                    Self::new()
                }
            }

            impl<L: ::tiny_ecs::system::StandardSchedules>
                ::tiny_ecs::world::WorldApi for #name<L>
            {
                type Label = L;

                fn add_schedule(&mut self, label: L) {
                    let _ = self.schedules.insert(label, ::tiny_ecs::schedule::Schedule::new());
                }

                fn add_system(&mut self, label: L, system: ::tiny_ecs::system::System) {
                    // Insert into an existing schedule when present, otherwise
                    // create one. Done without an entry API by trying `get_mut`
                    // first and falling back to a fresh `insert`.
                    if let ::core::option::Option::Some(sched) = self.schedules.get_mut(&label) {
                        sched.add(system);
                        return;
                    }
                    let mut sched = ::tiny_ecs::schedule::Schedule::new();
                    sched.add(system);
                    let _ = self.schedules.insert(label, sched);
                }

                fn run_schedule(&mut self, label: &L) {
                    // Derive the raw world pointer first, releasing any borrow
                    // before the immutable `&self.schedules` lookup below.
                    let world_ptr: *mut () = self as *mut Self as *mut ();
                    if let ::core::option::Option::Some(sched) = self.schedules.get(label) {
                        // Systems mutate disjoint fields through the raw
                        // pointer (invisible to the borrow checker), so holding
                        // the immutable `&self.schedules` borrow during `run`
                        // is sound.
                        sched.run(world_ptr);
                    }
                }

                fn spawn_empty(&mut self) -> ::core::option::Option<::tiny_ecs::entity::Entity> {
                    if self.next_index >= #entities && self.free_list.is_empty() {
                        return ::core::option::Option::None;
                    }
                    ::core::option::Option::Some(::tiny_ecs::entity::Entity::new(self.alloc_entity()))
                }

                fn insert_resource<R: 'static>(&mut self, resource: R)
                where
                    Self: ::tiny_ecs::system::ResourceInsRef<R>,
                {
                    let world_ptr: *mut Self = self as *mut Self;
                    // SAFETY: `world_ptr` is valid; the bound ensures the slot exists.
                    unsafe { <Self as ::tiny_ecs::system::ResourceInsRef<R>>::insert_resource(world_ptr, resource); }
                }

                fn despawn(&mut self, entity: ::tiny_ecs::entity::Entity) {
                    let idx = entity.index();
                    #( #comp_despawns )*
                    let _ = self.free_list.push(idx);
                    self.alive = self.alive.saturating_sub(1);
                }

                fn entity_count(&self) -> usize {
                    self.alive as usize
                }

                fn flush_commands(&mut self) {
                    // First, move every pending spawn value into its column.
                    #( #comp_flushes )*
                    // Then apply deferred despawns. Pop each command in its own
                    // statement so the `&mut self.commands` borrow ends before
                    // `self.despawn` reborrowsthe whole world mutably.
                    loop {
                        let cmd = self.commands.drain().next();
                        match cmd {
                            ::core::option::Option::Some(
                                ::tiny_ecs::commands_buffer::Command::Despawn(e),
                            ) => self.despawn(e),
                            ::core::option::Option::None => break,
                        }
                    }
                }

                fn commands_ptr(&mut self) -> *mut ::tiny_ecs::commands_buffer::CommandBuffer {
                    &mut self.commands as *mut _
                }
            }

            #( #column_ref_impls )*
            #( #resource_ref_impls )*
            #( #resource_ins_impls )*
            #( #spawn_ref_impls )*
            #commands_ref_impl
        };

        Ok(expanded)
    }
}