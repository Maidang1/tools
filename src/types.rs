use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TodoItem {
    pub id: u32,
    pub text: String,
    pub completed: bool,
    pub created_at: u64,
    pub name: String,
}

impl TodoItem {
    pub fn new(id: u32, text: String, name: String) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        TodoItem {
            id,
            text,
            completed: false,
            created_at,
            name,
        }
    }
}

impl fmt::Display for TodoItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.completed { "✓" } else { "○" };
        write!(f, "[{}] {}: {}", status, self.id, self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_item_creation() {
        let item = TodoItem::new(1, "Test task".to_string(), "Test name".to_string());
        assert_eq!(item.id, 1);
        assert_eq!(item.text, "Test task");
        assert_eq!(item.name, "Test name");
        assert_eq!(item.completed, false);
        assert!(item.created_at > 0);
    }

    #[test]
    fn test_display_format() {
        let item = TodoItem::new(1, "Test task".to_string(), "Test name".to_string());
        let display_str = format!("{}", item);
        assert_eq!(display_str, "[○] 1: Test task");

        let mut item_with_completed = item;
        item_with_completed.completed = true;
        let display_str_completed = format!("{}", item_with_completed);
        assert_eq!(display_str_completed, "[✓] 1: Test task");
    }
}
