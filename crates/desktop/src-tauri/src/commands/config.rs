use std::fs;
use std::path::PathBuf;

fn config_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Cannot find home directory".to_string())?;
    let config_dir = home.join(".d1doctor");
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    Ok(config_dir.join("config.toml"))
}

fn read_config_table() -> Result<toml::value::Table, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(toml::value::Table::new());
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let parsed: toml::Value = content.parse().map_err(|e: toml::de::Error| e.to_string())?;
    parsed
        .as_table()
        .cloned()
        .ok_or_else(|| "Config is not a TOML table".to_string())
}

#[tauri::command]
pub async fn get_config(key: String) -> Result<String, String> {
    let table = read_config_table()?;
    Ok(table
        .get(&key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string())
}

#[tauri::command]
pub async fn set_config(key: String, value: String) -> Result<(), String> {
    let path = config_path()?;
    let mut table = read_config_table()?;
    table.insert(key, toml::Value::String(value));
    let content =
        toml::to_string(&toml::Value::Table(table)).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}
