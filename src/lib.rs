use std::{
    borrow::BorrowMut,
    cell::{Ref, RefCell},
    rc::Rc,
    time::{Duration, Instant},
};

use gpui::{App, Entity, EntityId, Window, linear, prelude::*};

mod lerp;
pub use lerp::Lerp;

mod window;
pub use window::WindowUseTransition;

mod state;
pub use state::TransitionState;

/// A transition that can be applied to an element.
#[derive(Clone)]
pub struct Transition<T: Lerp + Clone + PartialEq + 'static> {
    /// The amount of time for which this transtion should run.
    duration_secs: f32,

    /// A function that takes a delta between 0 and 1 and returns a new delta
    /// between 0 and 1 based on the given easing function.
    easing: Rc<dyn Fn(f32) -> f32>,

    state: Entity<TransitionState<T>>,

    /// A cached version of the transition's value.
    cached_value: RefCell<Option<T>>,
}

impl<T: Lerp + Clone + PartialEq + 'static> Transition<T> {
    /// Create a new transition with the given duration using the specified state.
    pub fn new(state: Entity<TransitionState<T>>, duration: Duration) -> Self {
        Self {
            duration_secs: duration.as_secs_f32(),
            easing: Rc::new(linear),
            state,
            cached_value: RefCell::new(None),
        }
    }

    /// Set the easing function to use for this transition.
    /// The easing function will take a time delta between 0 and 1 and return a new delta
    /// between 0 and 1
    pub fn with_easing(mut self, easing: impl Fn(f32) -> f32 + 'static) -> Self {
        self.easing = Rc::new(easing);
        self
    }

    fn default_goal_updated_at(&self) -> Instant {
        Instant::now() - Duration::from_secs_f32(self.duration_secs)
    }

    /// Evaluates the value of the transition without using the cache.
    /// Returns if the transition is finished (bool) and the evaluated value (T).
    fn raw_evaluate(&self, cx: &mut App) -> (bool, T) {
        let mut state_entity = self.state.as_mut(cx);
        let state: &mut TransitionState<T> = state_entity.borrow_mut();

        let elapsed_secs = state
            .goal_last_updated_at
            .unwrap_or_else(|| self.default_goal_updated_at())
            .elapsed()
            .as_secs_f32();
        let delta = (self.easing)((elapsed_secs / self.duration_secs).min(1.));

        debug_assert!(
            (0.0..=1.0).contains(&delta),
            "delta should always be between 0 and 1"
        );

        state.last_delta = delta;

        let evaluated_value = state.start_goal.lerp(&state.end_goal, delta);

        (delta != 1., evaluated_value)
    }

    /// Evaluates the current value of the transition.
    pub fn evaluate(&self, window: &mut Window, cx: &mut App) -> Ref<'_, T> {
        if self.cached_value.borrow().is_none() {
            let (in_progress, evaluated_value) = self.raw_evaluate(cx);

            if in_progress {
                window.request_animation_frame();
            }

            *self.cached_value.borrow_mut() = Some(evaluated_value);
        }

        Ref::map(self.cached_value.borrow(), |opt| opt.as_ref().unwrap())
    }

    /// Reads the end goal of the transitions.
    pub fn read_goal<'b>(&'b self, cx: &'b mut App) -> &'b T {
        &self.state.read(cx).end_goal
    }

    /// Reads the current value of the cached transition, if it exists.
    pub fn read_cache(&self) -> Ref<'_, Option<T>> {
        self.cached_value.borrow()
    }

    /// Evaluates the current delta of the transition.
    pub fn evaluate_delta<'b>(&'b self, cx: &'b App) -> f32 {
        let goal_last_updated_at = self
            .state
            .read(cx)
            .goal_last_updated_at
            .unwrap_or_else(|| self.default_goal_updated_at());

        let elapsed_secs = goal_last_updated_at.elapsed().as_secs_f32();
        (self.easing)((elapsed_secs / self.duration_secs).min(1.))
    }

    /// Updates the goal for the transition without notifying gpui of any changes.
    pub fn update<R>(
        &self,
        cx: &mut App,
        update: impl FnOnce(&mut T, &mut crate::Context<TransitionState<T>>) -> R,
    ) -> bool {
        let mut was_updated = false;

        self.state.update(cx, |state, cx| {
            let last_end_goal = state.end_goal.clone();

            update(&mut state.end_goal, cx);

            if state.end_goal == last_end_goal {
                return;
            };

            state.goal_last_updated_at = Some(Instant::now());
            state.start_goal = state.start_goal.lerp(&last_end_goal, state.last_delta);

            was_updated = true;
        });

        was_updated
    }

    /// Get the entity ID associated with this entity
    pub fn entity_id(&self) -> EntityId {
        self.state.entity_id()
    }
}
