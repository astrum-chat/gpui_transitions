//! Originally reading/evaluating transitions used a similar api to `with_animations` but
//! it has since been replaced with a more ergonomic `transition.evaluate` method.
//!
//! This feature-gated module allows consumers who are still using the `with_transitions`
//! api (mainly Tesserae) to gradually adopt `transition.evaluate`.
//! This will probably be removed soon, so it should not be used for new work.

#![allow(deprecated)]

use gpui::{
    AnyElement, App, Bounds, ElementId, GlobalElementId, InspectorElementId, Interactivity,
    LayoutId, Pixels, StyleRefinement, Window, prelude::*,
};

use crate::{Lerp, Transition};

#[deprecated(
    since = "0.1.0",
    note = "The `element.with_transitions(...)` API is now deprecrated. Use `transition.read(...)` instead."
)]
/// An extension trait for adding the transition wrapper to both Elements and Components
pub trait TransitionExt {
    /// Render this component or element with transitions
    fn with_transitions<'a, R, T>(
        self,
        transitions: T,
        animator: impl Fn(&mut App, Self, T::Values) -> R + 'static,
    ) -> TransitionElement<'a, Self, R, T>
    where
        T: TransitionValues<'a>,
        Self: Sized,
    {
        TransitionElement {
            element: Some(self),
            animator: Box::new(animator),
            transitions,
        }
    }
}

impl<E: IntoElement + 'static> TransitionExt for E {}

#[deprecated(
    since = "0.1.0",
    note = "The `element.with_transitions(...)` API is now deprecrated. Use `transition.read(...)` instead."
)]
/// A GPUI element that applies a transition to another element
pub struct TransitionElement<'a, E, R, T: TransitionValues<'a>> {
    element: Option<E>,
    transitions: T,
    animator: Box<dyn Fn(&mut App, E, T::Values) -> R + 'a>,
}

impl<E: IntoElement + 'static, R: IntoElement + 'static, T: TransitionValues<'static> + 'static>
    Element for TransitionElement<'static, E, R, T>
{
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let (request_frame, evaluated_values) = self.transitions.evaluate(cx);

        let element = self.element.take().expect("should only be called once");
        let mut element = (self.animator)(cx, element, evaluated_values).into_any_element();

        if request_frame {
            window.request_animation_frame();
        }

        (element.request_layout(window, cx), element)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx)
    }
}

impl<E: IntoElement + 'static, R: IntoElement + 'static, T: TransitionValues<'static> + 'static>
    IntoElement for TransitionElement<'static, E, R, T>
{
    type Element = TransitionElement<'static, E, R, T>;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<
    E: IntoElement + Styled + 'static,
    R: IntoElement + Styled + 'static,
    T: TransitionValues<'static> + 'static,
> Styled for TransitionElement<'static, E, R, T>
{
    fn style(&mut self) -> &mut StyleRefinement {
        self.element.as_mut().unwrap().style()
    }
}

impl<
    E: IntoElement + InteractiveElement + 'static,
    R: IntoElement + InteractiveElement + 'static,
    T: TransitionValues<'static> + 'static,
> InteractiveElement for TransitionElement<'static, E, R, T>
{
    fn interactivity(&mut self) -> &mut Interactivity {
        self.element.as_mut().unwrap().interactivity()
    }
}

impl<
    E: IntoElement + ParentElement + 'static,
    R: IntoElement + ParentElement + 'static,
    T: TransitionValues<'static> + 'static,
> ParentElement for TransitionElement<'static, E, R, T>
{
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.element.as_mut().unwrap().extend(elements);
    }
}

impl<
    E: IntoElement + StatefulInteractiveElement + 'static,
    R: IntoElement + StatefulInteractiveElement + 'static,
    T: TransitionValues<'static> + 'static,
> StatefulInteractiveElement for TransitionElement<'static, E, R, T>
{
}

#[deprecated(
    since = "0.1.0",
    note = "The `element.with_transitions(...)` API is now deprecrated. Use `transition.read(...)` instead."
)]
// A group of values that can be transitioned.
pub trait TransitionValues<'a> {
    /// The underlying type of the values.
    type Values;

    /// Evaluates the values for the transitions based on the start and end goals.
    fn evaluate(&self, cx: &mut App) -> (bool, Self::Values);
}

// Workaround for variadic generics as Rust doesn't support them.
// The main downside to this is that each tuple length needs its own implementation.
macro_rules! impl_with_transitions {
        ($first:ident $(, $rest:ident)*) => {
            impl_with_transitions!(@recurse () $first $(, $rest)*);
        };

        // Nothing left.
        (@recurse ($($prefix:ident),*) ) => {};

        // Generates an impl for the current prefix + head,
        // then recurses to include the next identifier in the prefix.
        (@recurse ($($prefix:ident),*) $head:ident $(,$tail:ident)*) => {
            impl_with_transitions!(@gen ($($prefix,)* $head));
            impl_with_transitions!(@recurse ($($prefix,)* $head) $($tail),*);
        };

        (@gen ($($names:ident),+)) => {
            #[allow(non_snake_case, unused_parens)]
            impl<'a, $($names),+> TransitionValues<'a> for ( $( Transition<$names> ),+, )
            where
                $( $names: Lerp + Clone + PartialEq + 'static ),+
            {
                type Values = ( $( $names ),+);

                fn evaluate(&self, cx: &mut App) -> (bool, Self::Values)
                {
                    let ( $( $names ),+ ,) = self;
                    let mut request_frame = false;

                    let evaluated_values = ($({
                        let (this_request_frame, transioned_value) = $names.raw_evaluate(cx);
                        request_frame = this_request_frame || request_frame;
                        transioned_value
                    }),+);

                    (request_frame, evaluated_values)
                }
            }
        };
    }

impl_with_transitions!(A, B, C, D, E, F);

impl<'a, A> TransitionValues<'a> for Transition<A>
where
    A: Lerp + Clone + PartialEq + 'static,
{
    type Values = A;

    fn evaluate(&self, cx: &mut App) -> (bool, Self::Values) {
        Transition::raw_evaluate(&self, cx)
    }
}
