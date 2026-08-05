//! Procedural macros for `tiny_ecs`: the marker derives
//! ([`Component`](macro.Component.html), [`Resource`](macro.Resource.html),
//! [`ScheduleLabel`](macro.ScheduleLabel.html)), the [`system`](macro.system.html)
//! attribute that expands Bevy-style system functions, and the
//! [`define_world!`](macro.define_world.html) builder that lays out the concrete
//! `World` struct per application.
//!
//! The derive macros are deliberately thin: `Component`/`Resource` are plain
//! marker impls, and `ScheduleLabel` just seals the blanketed trait. `system`
//! and `define_world` carry the real work.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, ItemFn};

mod world_macro;
use world_macro::WorldInput;

// ---------------------------------------------------------------------
// --- Marker derives --------------------------------------------------

/// Derives [`tiny_ecs::Component`] for the annotated type.
#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics ::tiny_ecs::Component for #name #ty_generics #where_clause {}
    };
    expanded.into()
}

/// Derives [`tiny_ecs::Resource`] for the annotated type.
#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics ::tiny_ecs::Resource for #name #ty_generics #where_clause {}
    };
    expanded.into()
}

/// Derives [`tiny_ecs::schedule::ScheduleLabel`] for the annotated type.
///
/// The type must be `'static` (e.g. a unit struct). The label's identity is
/// derived from its [`TypeId`](core::any::TypeId), so no additional fields or
/// bounds are required — derive this trait, then pass the unit value as a
/// marker: `app.add_system(MyLabel, system)`.
#[proc_macro_derive(ScheduleLabel)]
pub fn derive_schedule_label(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics ::tiny_ecs::schedule::ScheduleLabel for #name #ty_generics #where_clause {}
    };
    expanded.into()
}

// ---------------------------------------------------------------------
// --- `define_world!` -------------------------------------------------

/// Generates the concrete `World` struct for an application, together with the
/// `ColumnRef`/`ResourceRef`/`ResourceInsRef`/`SpawnRef`/`CommandsRef` impls
/// for every registered component/resource, and the `WorldApi` impl driving
/// schedules and entity lifetimes.
#[proc_macro]
pub fn define_world(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as WorldInput);
    input
        .expand()
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

// ---------------------------------------------------------------------
// --- `#[system]` -----------------------------------------------------

/// Recognises the system param types inside a `fn` signature.
#[derive(Clone)]
enum SystemParam {
    /// `Res<T>`: shared immutable resource.
    Res(syn::Type, bool /*mut binding*/),
    /// `ResMut<T>`: exclusive mutable resource.
    ResMut(syn::Type, bool /*mut binding*/),
    /// `Query<P>`: a query over component columns.
    Query(syn::Type, bool /*mut binding*/),
    /// `Commands`: spawn / despawn / insert_resource handle.
    Commands(bool /*mut binding*/),
}

/// The `#[system]` attribute macro.
#[proc_macro_attribute]
pub fn system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    expand_system(item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_system(item: ItemFn) -> syn::Result<TokenStream2> {
    let sig = &item.sig;
    if sig.asyncness.is_some() {
        return Err(syn::Error::new_spanned(sig, "async systems are not supported"));
    }
    if !sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &sig.generics,
            "system functions may not declare their own generic parameters",
        ));
    }
    let inputs = &sig.inputs;
    let body = &item.block;
    let vis = &item.vis;
    let user_name = sig.ident.clone();
    let sys_fn_name = format_ident!("{}_sys", user_name);
    let sys_struct_name = format_ident!("{}", user_name);

    // Parse each parameter.
    let mut params: Vec<(SystemParam, syn::Ident, bool)> = Vec::new();
    for arg in inputs {
        match arg {
            syn::FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(arg, "self receivers are not allowed"));
            }
            syn::FnArg::Typed(pt) => {
                let pat = &pt.pat;
                let ty = &pt.ty;
                let (ident, mut_b) = pat_ident(pat)?;
                let parsed = parse_param_type(ty)
                    .ok_or_else(|| syn::Error::new_spanned(ty, "unsupported system parameter type"))?;
                params.push((parsed, ident, mut_b));
            }
        }
    }

    let w = format_ident!("__W");
    // Build generic bounds: `__W: ColumnRef<T> + SpawnRef<T> + ResourceRef<R> +
    // ResourceInsRef<R> + ...`. Adding `SpawnRef<T>` next to every queried
    // component (and `ResourceInsRef<R>` next to every read resource) lets a
    // system call `commands.spawn::<T>(value)` for a component it also queries
    // and `commands.insert_resource::<R>(value)` for a resource it reads,
    // without the macro inspecting the body for the exact spawn/insert types.
    let mut bounds: Vec<syn::TypeParamBound> = Vec::new();
    bounds.push(parse_quote!(Sized));
    for (p, _, _) in &params {
        match p {
            SystemParam::Query(ty, _) => {
                for comp in components_in_fetch(ty) {
                    bounds.push(parse_quote!(::tiny_ecs::system::ColumnRef<#comp>));
                    bounds.push(parse_quote!(::tiny_ecs::system::SpawnRef<#comp>));
                }
            }
            SystemParam::Res(ty, _) | SystemParam::ResMut(ty, _) => {
                bounds.push(parse_quote!(::tiny_ecs::system::ResourceRef<#ty>));
                bounds.push(parse_quote!(::tiny_ecs::system::ResourceInsRef<#ty>));
            }
            SystemParam::Commands(_) => {
                bounds.push(parse_quote!(::tiny_ecs::system::CommandsRef));
            }
        }
    }

    // Build parameter bindings emitted at the top of the generated fn body.
    let mut bindings: Vec<TokenStream2> = Vec::new();
    for (p, ident, mut_b) in &params {
        let mut_kw = if *mut_b { quote!(mut) } else { quote!() };
        match p {
            SystemParam::Res(ty, _) => {
                bindings.push(quote! {
                    let #mut_kw #ident: ::tiny_ecs::system::Res<'_, #ty> = {
                        // SAFETY: `__w` is valid for this system's duration.
                        let __ptr = unsafe { <#w as ::tiny_ecs::system::ResourceRef<#ty>>::res_ref_raw(__w) };
                        assert!(!__ptr.is_null(), concat!("resource not inserted: ", stringify!(#ty)));
                        // SAFETY: the non-null check above guarantees validity.
                        unsafe { ::tiny_ecs::system::Res::new(&*__ptr) }
                    };
                });
            }
            SystemParam::ResMut(ty, _) => {
                bindings.push(quote! {
                    let #mut_kw #ident: ::tiny_ecs::system::ResMut<'_, #ty> = {
                        // SAFETY: `__w` is valid for this system's duration.
                        let __ptr = unsafe { <#w as ::tiny_ecs::system::ResourceRef<#ty>>::res_mut_raw(__w) };
                        assert!(!__ptr.is_null(), concat!("resource not inserted: ", stringify!(#ty)));
                        // SAFETY: the non-null check above guarantees validity.
                        unsafe { ::tiny_ecs::system::ResMut::new(&mut *__ptr) }
                    };
                });
            }
            SystemParam::Query(ty, _) => {
                bindings.push(quote! {
                    let #mut_kw #ident: ::tiny_ecs::system::Query<'_, #ty, #w> =
                        // SAFETY: `__w` is a valid `*mut #w`.
                        unsafe { ::tiny_ecs::system::Query::from_world(__w) };
                });
            }
            SystemParam::Commands(_) => {
                bindings.push(quote! {
                    let #mut_kw #ident: ::tiny_ecs::commands::Commands<'_, #w> = {
                        // SAFETY: `__w` is valid and the command buffer is distinct.
                        let __buf = unsafe { <#w as ::tiny_ecs::system::CommandsRef>::commands_raw(__w) };
                        // SAFETY: the buffer lives for the system's duration.
                        unsafe { ::tiny_ecs::commands::Commands::new(__w, &mut *__buf) }
                    };
                });
            }
        }
    }

    let expanded = quote! {
        #[doc(hidden)]
        #vis fn #sys_fn_name<#w: #(#bounds)+*>(__world: *mut ()) {
            // SAFETY: the runner guarantees `__world` is a valid `*mut #w`.
            let __w: *mut #w = __world as *mut #w;
            #(#bindings)*
            #body
        }

        #[doc(hidden)]
        #vis struct #sys_struct_name;

        impl<#w: #(#bounds)+*> ::tiny_ecs::system::IntoSystem<#w> for #sys_struct_name {
            fn into_system(self) -> ::tiny_ecs::system::System {
                #sys_fn_name::<#w>
            }
        }
    };
    Ok(expanded)
}

/// Extracts the binding ident and `mut`-ness from a function argument pattern.
fn pat_ident(pat: &syn::Pat) -> syn::Result<(syn::Ident, bool)> {
    if let syn::Pat::Ident(pi) = pat {
        return Ok((pi.ident.clone(), pi.mutability.is_some()));
    }
    Err(syn::Error::new_spanned(pat, "expected a simple identifier pattern"))
}

/// Recognises a system param type and returns its [`SystemParam`] kind.
fn parse_param_type(ty: &syn::Type) -> Option<SystemParam> {
    let path = match ty {
        syn::Type::Path(p) if p.qself.is_none() => &p.path,
        _ => return None,
    };
    let seg = path.segments.last()?;
    let ident = seg.ident.to_string();
    if ident == "Commands" {
        return Some(SystemParam::Commands(false));
    }
    // Single-segment generic: `Res<...>`, `ResMut<...>`, `Query<...>`.
    let args = match &seg.arguments {
        syn::PathArguments::AngleBracketed(a) => a,
        _ => return None,
    };
    let inner = args.args.first()?;
    let inner_ty = match inner {
        syn::GenericArgument::Type(t) => t.clone(),
        _ => return None,
    };
    match ident.as_str() {
        "Res" => Some(SystemParam::Res(inner_ty, false)),
        "ResMut" => Some(SystemParam::ResMut(inner_ty, false)),
        "Query" => Some(SystemParam::Query(inner_ty, false)),
        _ => None,
    }
}

/// Walks a query parameter type and yields the component types referenced via
/// `&T` / `&mut T` (those are the columns that must be borrowable).
fn components_in_fetch(ty: &syn::Type) -> Vec<syn::Type> {
    let mut out = Vec::new();
    walk_refs(ty, &mut out);
    out
}

fn walk_refs(ty: &syn::Type, out: &mut Vec<syn::Type>) {
    match ty {
        syn::Type::Reference(r) => {
            // `&T` or `&mut T` — the inner type (without lifetime) is a column.
            out.push((*r.elem).clone());
        }
        syn::Type::Paren(p) => walk_refs(&p.elem, out),
        syn::Type::Tuple(t) => {
            for e in &t.elems {
                walk_refs(e, out);
            }
        }
        _ => {} // Entity, etc. — not a column.
    }
}

use syn::parse_quote;
// a tiny re-export so the `parse_quote!` calls above resolve.
