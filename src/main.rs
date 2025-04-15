//
use std::io;
use std::io::Write;
use std::io::BufRead;


struct TodoItem {
    id: u64,
    title: String,
    completed: bool,
}


struct TodoList {
    items: Vec<TodoItem>,
}

impl TodoList {
    fn new() -> Self {
        TodoList { items: Vec::new() }
    }

    fn add_item(&mut self, title: String) {
        let id = self.items.len() as u64 + 1;
        let new_item = TodoItem {
            id,
            title,
            completed: false,
        };
        self.items.push(new_item);
    }

    fn remove_item(&mut self, id: u64) {
        self.items.retain(|item| item.id != id);
    }
    fn mark_completed(&mut self, id: u64) {
        if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
            item.completed = true;
        }
    }

    fn list_items(&self) {
        for item in &self.items {
            let status = if item.completed { "✓" } else { "✗" };
            println!("{}: {} [{}]", item.id, item.title.trim(), status);
        }
    }
    fn save_to_file(&self, filename: &str) {
        let mut file = std::fs::File::create(filename).expect("Unable to create file");
        for item in &self.items {
            writeln!(file, "{}: {}: {}", item.id, item.title, item.completed).expect("Unable to write to file");
        }
    }
    fn load_from_file(&mut self, filename: &str) {
        let file = std::fs::File::open(filename).expect("Unable to open file");
        let reader = std::io::BufReader::new(file);
        for line in reader.lines() {
            let line = line.expect("Unable to read line");
            let parts: Vec<&str> = line.split(": ").collect();

            let id: u64 = parts[0].parse().expect("Unable to parse ID");
            let title = parts[1].to_string();
            let completed: bool = parts[2].parse().expect("Unable to parse completed status");
            self.items.push(TodoItem { id, title, completed });
        }
    }

    fn load_default_todo_list_from_file(&mut self) {
        let file = std::fs::File::open("DTL.txt").expect("Unable to open file");
        let reader = std::io::BufReader::new(file);
        for line in reader.lines() {
            let line = line.expect("Unable to read line");
            let parts: Vec<&str> = line.split(": ").collect();

            let id: u64 = parts[0].parse().expect("Unable to parse ID");
            let title = parts[1].to_string();
            let completed: bool = parts[2].parse().expect("Unable to parse completed status");
            self.items.push(TodoItem { id, title, completed });
        }
    }
    fn create_default_todo_list_empty_file(&self) {
        let mut file = std::fs::File::create("DTL.txt").expect("Unable to create file");
        writeln!(file, "0: : false").expect("Unable to write to file");
    }

}

fn main() {
    let mut todo_list = TodoList::new();
    if std::path::Path::new("DTL.txt").exists() {
        todo_list.load_default_todo_list_from_file();
    } else {
        todo_list.create_default_todo_list_empty_file();
    }
    println!("Welcome to the Todo List App!");
    loop {
        println!("1. Add item");
        println!("2. Remove item");
        println!("3. Mark completed");
        println!("4. List items");
        println!("5. Save to file");
        println!("6. Load from file");
        println!("7. Exit");
        println!("Please select an option:");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read line");
        let choice: u32 = choice.trim().parse().expect("Please enter a number");

        match choice {
            1 => {
                let mut title = String::new();
                println!("Enter item title:");
                io::stdin().read_line(&mut title).expect("Failed to read line");
                todo_list.add_item(title.trim().to_string());
            }
            2 => {
                let mut id = String::new();
                println!("Enter item ID to remove:");
                io::stdin().read_line(&mut id).expect("Failed to read line");
                let id: u64 = id.trim().parse().expect("Please enter a number");
                todo_list.remove_item(id);
            }
            3 => {
                let mut id = String::new();
                println!("Enter item ID to mark as completed:");
                io::stdin().read_line(&mut id).expect("Failed to read line");
                let id: u64 = id.trim().parse().expect("Please enter a number");
                todo_list.mark_completed(id);
            }
            4 => {
                todo_list.list_items();
            }
            5 => {
                let mut filename = String::new();
                println!("Enter filename to save:");
                io::stdin().read_line(&mut filename).expect("Failed to read line");
                todo_list.save_to_file(filename.trim());
            }
            6 => {
                let mut filename = String::new();
                println!("Enter filename to load:");
                io::stdin().read_line(&mut filename).expect("Failed to read line");
                todo_list.load_from_file(filename.trim());
                println!("");
                println!("Loaded items:");
                todo_list.list_items();
            }
            7 => break,
            _ => println!("Invalid choice, please try again."),
        }
    }
}