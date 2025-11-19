// app.rs
use crate::todo::Todo;
use crate::tui::parse_due_date;
use chrono::Local;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub fn get_data_file_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "KushalMeghani", "RustyTodos")
        .expect("Failed to get project directories");
    let dir = proj_dirs.config_dir();
    std::fs::create_dir_all(dir).unwrap();
    dir.join("todos.json")
}

#[derive(PartialEq, Deserialize, Serialize)]
pub enum InputMode {
    Normal,
    EditingDescription,
    EditingDueDate,
    Searching, // Added for search mode
}

#[derive(Serialize, Deserialize)]
pub struct App {
    pub todos: Vec<Todo>,

    #[serde(skip)]
    pub input_mode: InputMode,
    #[serde(skip)]
    pub input_description: String,
    #[serde(skip)]
    pub input_due_date: String,
    #[serde(skip)]
    pub selected: usize,
    #[serde(skip)]
    pub error_message: Option<String>,
    #[serde(skip)]
    pub search_query: String, // Added for search
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal
    }
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            todos: Vec::new(),
            input_mode: InputMode::Normal,
            input_description: String::new(),
            input_due_date: String::new(),
            selected: 0,
            error_message: None,
            search_query: String::new(), // Initialize search_query
        }
    }

    pub fn add_todo(&mut self) -> Result<(), String> {
        if self.input_description.trim().is_empty() {
            return Err("Description cannot be empty.".to_string());
        }

        let due_date_str = if self.input_due_date.trim().is_empty() {
            None
        } else {
            Some(parse_due_date(&self.input_due_date)?)
        };

        let mut new_todo = Todo {
            description: self.input_description.clone(),
            done: false,
            due_date: due_date_str.clone(),
            created_date: Local::now().format("%Y-%m-%d").to_string(),
            priority: 0.0,
        };

        // Calculate priority based on due date for proper sorting
        if let Some(ref due_date) = due_date_str {
            // Find insertion point based on due date
            let mut insertion_priority: f64 = 0.0;
            for todo in &self.todos {
                if let Some(ref existing_due) = todo.due_date {
                    if due_date >= existing_due {
                        insertion_priority = insertion_priority.max(todo.priority);
                    }
                }
            }
            new_todo.priority = insertion_priority;
        } else {
            // Tasks without due dates go to the end
            new_todo.priority = self.todos.iter().map(|t| t.priority).fold(0.0_f64, f64::max);
        }

        self.todos.push(new_todo);

        // clear inputs after adding
        self.input_description.clear();
        self.input_due_date.clear();
        self.error_message = None;

        Ok(())
    }

    pub fn delete_todo(&mut self) {
        if !self.todos.is_empty() {
            self.todos.remove(self.selected);
            if self.selected > 0 {
                self.selected -= 1;
            }
        }
    }

    pub fn mark_done(&mut self) {
        if let Some(todo) = self.todos.get_mut(self.selected) {
            todo.done = !todo.done;
        }
    }

    pub fn move_todo_up(&mut self) {
        if self.selected == 0 || self.todos.is_empty() {
            return;
        }

        // Get actual indices in the todos vec
        let filtered_indices = self.get_filtered_indices();
        if self.selected >= filtered_indices.len() {
            return;
        }

        let current_idx = filtered_indices[self.selected];
        let above_idx = filtered_indices[self.selected - 1];

        // Swap priorities with a small offset to ensure stable ordering
        let above_priority = self.todos[above_idx].priority;
        self.todos[current_idx].priority = above_priority - 0.5;

        self.selected -= 1;
    }

    pub fn move_todo_down(&mut self) {
        if self.todos.is_empty() {
            return;
        }

        let filtered_indices = self.get_filtered_indices();
        if self.selected >= filtered_indices.len().saturating_sub(1) {
            return;
        }

        let current_idx = filtered_indices[self.selected];
        let below_idx = filtered_indices[self.selected + 1];

        // Swap priorities with a small offset to ensure stable ordering
        let below_priority = self.todos[below_idx].priority;
        self.todos[current_idx].priority = below_priority + 0.5;

        self.selected += 1;
    }

    fn get_filtered_indices(&self) -> Vec<usize> {
        let filtered: Vec<(usize, &Todo)> = if self.search_query.is_empty() {
            self.todos.iter().enumerate().collect()
        } else {
            let q = self.search_query.to_lowercase();
            self.todos
                .iter()
                .enumerate()
                .filter(|(_, t)| {
                    t.description.to_lowercase().contains(&q)
                        || t.due_date
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&q))
                            .unwrap_or(false)
                })
                .collect()
        };

        let mut sorted_filtered = filtered;
        sorted_filtered.sort_by(|(_, a), (_, b)| {
            match (&a.due_date, &b.due_date) {
                (Some(a_date), Some(b_date)) => {
                    if a_date == b_date {
                        a.priority.partial_cmp(&b.priority).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        a_date.cmp(b_date)
                    }
                }
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.priority.partial_cmp(&b.priority).unwrap_or(std::cmp::Ordering::Equal),
            }
        });

        sorted_filtered.iter().map(|(idx, _)| *idx).collect()
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;

        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)
            .map_err(|e| format!("Failed to write JSON!: {}", e))
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Self {
        let file = File::open(&path);
        if let Ok(file) = file {
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_else(|_| App::new())
        } else {
            App::new()
        }
    }
}
