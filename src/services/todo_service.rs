
use crate::types::TodoItem;
use serde_json;
use std::fs;
use std::path::PathBuf;

pub struct TodoService {
    todo_data_path: PathBuf,
    pub todos: Vec<TodoItem>,
}

impl TodoService {
    pub fn new() -> Self {
        let root_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"));
        let todo_data_path = root_dir.join(".milk-cap").join("todo.json");

        // Create directory if it doesn't exist
        if let Some(parent) = todo_data_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create todo data directory");
        }

        // Read existing data or initialize with an empty array
        let todos: Vec<TodoItem> = if todo_data_path.exists() {
            let content = fs::read_to_string(&todo_data_path).unwrap_or_else(|_| "[]".to_string());
            if content.trim().is_empty() {
                vec![]
            } else {
                serde_json::from_str(&content).unwrap_or_else(|_| vec![])
            }
        } else {
            // Create the file with an empty array if it doesn't exist
            fs::write(&todo_data_path, "[]").expect("Failed to write initial todo data");
            vec![]
        };

        TodoService {
            todo_data_path,
            todos,
        }
    }
}

impl TodoService {
    pub fn save_to_file(&self) {
        let json_data = serde_json::to_string_pretty(&self.todos).unwrap_or_else(|_| "[]".to_string());
        fs::write(&self.todo_data_path, json_data).expect("Failed to write todos to file");
    }

    pub fn add_todo(&mut self, text: &str) {
        let next_id = self.todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        let new_todo = TodoItem::new(next_id, text.to_string(), format!("Todo {}", next_id));
        self.todos.push(new_todo);
        self.save_to_file();
        println!("Added todo: {}", text);
    }

    pub fn delete_todo(&mut self, id: u32) {
        self.todos.retain(|todo| todo.id != id);
        self.save_to_file();
        println!("Deleted todo with id: {}", id);
    }

    pub fn list_todos(&self) {
        if self.todos.is_empty() {
            println!("No todos found");
        } else {
            for todo in &self.todos {
                println!("{}", todo);
            }
        }
    }
}
