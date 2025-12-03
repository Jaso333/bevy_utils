use core::marker::PhantomData;

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

    unsafe fn get_components(
        _: MovingPtr<'_, Self>,
        _: &mut impl FnMut(StorageType, OwningPtr<'_>),
    ) {
        // SAFETY: Empty function body
    }

    unsafe fn apply_effect(
        ptr: MovingPtr<'_, std::mem::MaybeUninit<Self>>,
        entity: &mut EntityWorldMut,
    ) {
        // SAFETY: `get_components` does nothing, value was not moved.
        let construct = unsafe { ptr.assume_init() };
        let construct = construct.read();
        entity.insert((construct.func)(entity.world()));
    }
}

unsafe impl<C: Component, F: Fn(&World) -> C + Send + Sync + 'static> Bundle for Construct<C, F> {
    fn component_ids(_: &mut ComponentsRegistrator, _: &mut impl FnMut(ComponentId)) {
        // SAFETY: Empty function body
    }

    fn get_component_ids(_: &Components, _: &mut impl FnMut(Option<ComponentId>)) {
        // SAFETY: Empty function body
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
    fn component_ids(_: &mut ComponentsRegistrator, _: &mut impl FnMut(ComponentId)) {
        // SAFETY: Empty function body
    }

    fn get_component_ids(_: &Components, _: &mut impl FnMut(Option<ComponentId>)) {
        // SAFETY: Empty function body
    }
}

impl<E: EntityEvent, B: Bundle, M, I: IntoObserverSystem<E, B, M>> DynamicBundle
    for AddObserver<E, B, M, I>
{
    type Effect = Self;

    #[inline]
    unsafe fn get_components(
        _ptr: MovingPtr<'_, Self>,
        _func: &mut impl FnMut(StorageType, OwningPtr<'_>),
    ) {
        // SAFETY: Empty function body
    }

    #[inline]
    unsafe fn apply_effect(
        ptr: MovingPtr<'_, core::mem::MaybeUninit<Self>>,
        entity: &mut EntityWorldMut,
    ) {
        // SAFETY: `get_components` does nothing, value was not moved.
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
