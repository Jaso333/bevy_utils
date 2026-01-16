use core::marker::PhantomData;
use std::mem::{self, MaybeUninit};

use bevy::{
    ecs::{
        bundle::DynamicBundle,
        component::{ComponentId, Components, ComponentsRegistrator, StorageType},
        system::IntoObserverSystem,
    },
    prelude::*,
    ptr::{MovingPtr, OwningPtr},
};

pub struct Construct<C: Component, F: Fn(&World) -> C> {
    func: F,
}

impl<C: Component, F: Fn(&World) -> C> DynamicBundle for Construct<C, F> {
    type Effect = Self;

    #[inline]
    unsafe fn get_components(
        ptr: MovingPtr<'_, Self>,
        _func: &mut impl FnMut(StorageType, OwningPtr<'_>),
    ) {
        // SAFETY: We must not drop the pointer here, or it will be uninitialized in `apply_effect`
        // below.
        mem::forget(ptr);
    }

    unsafe fn apply_effect(ptr: MovingPtr<'_, MaybeUninit<Self>>, entity: &mut EntityWorldMut) {
        // SAFETY: The pointer was not dropped in `get_components`, so the allocation is still
        // initialized.
        let construct = unsafe { ptr.assume_init() };
        let construct = construct.read();
        entity.insert((construct.func)(entity.world()));
    }
}

unsafe impl<C: Component, F: Fn(&World) -> C + Send + Sync + 'static> Bundle for Construct<C, F> {
    #[inline]
    fn component_ids(
        _components: &mut ComponentsRegistrator,
    ) -> impl Iterator<Item = ComponentId> + use<C, F> {
        // SAFETY: Empty iterator
        core::iter::empty()
    }

    #[inline]
    fn get_component_ids(_components: &Components) -> impl Iterator<Item = Option<ComponentId>> {
        // SAFETY: Empty iterator
        core::iter::empty()
    }
}

pub trait ComponentConstruct<F: Fn(&World) -> Self> {
    fn construct(func: F) -> Construct<Self, F>
    where
        Self: Component + Sized;
}

impl<C: Component, F: Fn(&World) -> C> ComponentConstruct<F> for C {
    fn construct(func: F) -> Construct<Self, F> {
        Construct { func }
    }
}

/// Helper struct that adds an observer when inserted as a [`Bundle`].
pub struct AddObserver<E: EntityEvent, B: Bundle, M, I: IntoObserverSystem<E, B, M>> {
    observer: I,
    marker: PhantomData<(E, B, M)>,
}

// SAFETY: Empty method bodies.
unsafe impl<
    E: EntityEvent,
    B: Bundle,
    M: Send + Sync + 'static,
    I: IntoObserverSystem<E, B, M> + Send + Sync,
> Bundle for AddObserver<E, B, M, I>
{
    #[inline]
    fn component_ids(
        _components: &mut ComponentsRegistrator,
    ) -> impl Iterator<Item = ComponentId> + use<E, B, M, I> {
        // SAFETY: Empty iterator
        core::iter::empty()
    }

    #[inline]
    fn get_component_ids(_components: &Components) -> impl Iterator<Item = Option<ComponentId>> {
        // SAFETY: Empty iterator
        core::iter::empty()
    }
}

impl<E: EntityEvent, B: Bundle, M, I: IntoObserverSystem<E, B, M>> DynamicBundle
    for AddObserver<E, B, M, I>
{
    type Effect = Self;

    #[inline]
    unsafe fn get_components(
        ptr: MovingPtr<'_, Self>,
        _func: &mut impl FnMut(StorageType, OwningPtr<'_>),
    ) {
        // SAFETY: We must not drop the pointer here, or it will be uninitialized in `apply_effect`
        // below.
        mem::forget(ptr);
    }

    #[inline]
    unsafe fn apply_effect(ptr: MovingPtr<'_, MaybeUninit<Self>>, entity: &mut EntityWorldMut) {
        // SAFETY: The pointer was not dropped in `get_components`, so the allocation is still
        // initialized.
        let add_observer = unsafe { ptr.assume_init() };
        let add_observer = add_observer.read();
        entity.observe(add_observer.observer);
    }
}

/// Adds an observer as a bundle effect.
pub fn observe<E: EntityEvent, B: Bundle, M, I: IntoObserverSystem<E, B, M>>(
    observer: I,
) -> AddObserver<E, B, M, I> {
    AddObserver {
        observer,
        marker: PhantomData,
    }
}

pub trait SceneBuilder {
    fn build(self) -> impl Bundle;
}
