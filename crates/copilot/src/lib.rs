mod station;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Day1 Copilot.", name)
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running Day1 Copilot");
}
