use std::time::Instant;

use crate::Lerp;

/// State for a transition.
#[derive(Clone)]
pub struct TransitionState<T: Lerp + Clone + PartialEq + 'static> {
    pub(crate) goal_last_updated_at: Option<Instant>,
    pub(crate) initial_goal: T,
    pub(crate) start_goal: T,
    pub(crate) end_goal: T,
    pub(crate) last_delta: f32,
}

impl<T: Lerp + Clone + PartialEq + 'static> TransitionState<T> {
    pub fn new(initial_goal: T) -> Self {
        Self {
            goal_last_updated_at: None,
            initial_goal: initial_goal.clone(),
            start_goal: initial_goal.clone(),
            end_goal: initial_goal,
            last_delta: 1.,
        }
    }
}
