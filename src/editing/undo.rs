//! Managing undo state

use std::collections::VecDeque;

/// Default size of the undo stack.
const DEFAULT_UNDO_STACK_SIZE: usize = 128;

/// A stack of states that can be undone and redone.
#[derive(Debug, Clone)]
pub struct UndoState<T: Clone> {
    /// Maximum number of undo states to store.
    max_undo_count: usize,
    /// The stack of states.
    stack: VecDeque<T>,
    /// The index in `stack` of the current state.
    live_index: usize,
}

impl<T: Clone> UndoState<T> {
    /// Create a new undo state with the default stack size.
    pub fn new(init_state: T) -> Self {
        Self::new_sized(DEFAULT_UNDO_STACK_SIZE, init_state)
    }

    /// Create a new undo state with a specific maximum stack size.
    fn new_sized(max_undo_count: usize, init_state: T) -> Self {
        let mut stack = VecDeque::new();
        stack.push_back(init_state);
        UndoState {
            max_undo_count,
            stack,
            live_index: 0,
        }
    }

    /// Undo the last action, returning the previous state.
    pub fn undo(&mut self) -> Option<&T> {
        if self.live_index == 0 {
            return None;
        }
        self.live_index -= 1;
        self.stack.get(self.live_index)
    }

    /// Redo a previously undone action, returning the state to revert to.
    pub fn redo(&mut self) -> Option<&T> {
        if self.live_index == self.stack.len() - 1 {
            return None;
        }
        self.live_index += 1;
        self.stack.get(self.live_index)
    }

    /// Add a new state to the undo stack.
    pub fn push(&mut self, item: T) {
        // If we have undone actions and then edit, we need to truncate the stack
        if self.live_index < self.stack.len() - 1 {
            self.stack.truncate(self.live_index + 1);
        }

        self.live_index += 1;
        self.stack.push_back(item);

        // If we exceed the max number of undo states, remove the oldest one
        if self.stack.len() > self.max_undo_count {
            self.stack.pop_front();
            self.live_index -= 1;
        }
    }

    /// Modify the state for the currently active undo group.
    ///
    /// This might be done if an edit occurs that combines with the previous undo,
    /// or if we want to save selection state.
    #[allow(dead_code)]
    pub fn update_current(&mut self, item: T) {
        if let Some(state) = self.stack.get_mut(self.live_index) {
            *state = item;
        }
    }

    /// Get the number of states in the undo stack.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Get the current index position in the undo stack.
    #[allow(dead_code)]
    pub fn current_index(&self) -> usize {
        self.live_index
    }

    /// Check if the undo stack is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Alias for push
    #[allow(dead_code)]
    pub fn add_undo_group(&mut self, item: T) {
        self.push(item)
    }

    /// Alias for update_current
    #[allow(dead_code)]
    pub fn update_current_undo(&mut self, mut f: impl FnMut(&mut T)) {
        if let Some(state) = self.stack.get_mut(self.live_index) {
            f(state);
        }
    }
}
