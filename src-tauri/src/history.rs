use std::collections::VecDeque;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ActionEntry {
    pub action_type: String,
    pub sound_id: String,
    pub prompt: String,
    pub timestamp: String,
}

#[derive(Clone, Debug)]
pub struct ActionHistory {
    pub undo_stack: VecDeque<ActionEntry>,
    pub redo_stack: VecDeque<ActionEntry>,
    pub max_entries: usize,
}

impl ActionHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(max_entries),
            redo_stack: VecDeque::with_capacity(max_entries),
            max_entries,
        }
    }

    pub fn push_action(&mut self, entry: ActionEntry) {
        if self.undo_stack.len() >= self.max_entries {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(entry);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<ActionEntry> {
        let entry = self.undo_stack.pop_back()?;
        if self.redo_stack.len() >= self.max_entries {
            self.redo_stack.pop_front();
        }
        self.redo_stack.push_back(entry.clone());
        Some(entry)
    }

    pub fn redo(&mut self) -> Option<ActionEntry> {
        let entry = self.redo_stack.pop_back()?;
        if self.undo_stack.len() >= self.max_entries {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(entry.clone());
        Some(entry)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
