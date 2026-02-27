use crate::cli::ConfigAction;
use anyhow::{Context, Result};
use colored::Colorize;

fn config_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~"))
        .join(".d1doctor")
        .join("config.toml")
}

pub async fn execute(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Get { key } => {
            let path = config_path();
            if !path.exists() {
                println!(
                    "{} Config file not found at {}",
                    "⚠".yellow(),
                    path.display()
                );
                return Ok(());
            }
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let table: toml::Value = content
                .parse()
                .with_context(|| "Failed to parse config.toml")?;

            match key {
                None => println!("{}", toml::to_string_pretty(&table)?),
                Some(k) => {
                    let val = k
                        .split('.')
                        .try_fold(&table, |t, segment| t.get(segment))
                        .ok_or_else(|| anyhow::anyhow!("Key '{}' not found in config", k))?;
                    println!("{}", val);
                }
            }
        }
        ConfigAction::Set { key, value } => {
            let path = config_path();
            let content = if path.exists() {
                std::fs::read_to_string(&path)?
            } else {
                let parent = path.parent().ok_or_else(|| anyhow::anyhow!("Config path has no parent directory"))?;
                std::fs::create_dir_all(parent)?;
                String::new()
            };
            let mut table: toml::value::Table = toml::from_str(&content).unwrap_or_default();

            // Set nested key e.g. "orchestrator.url"
            let parts: Vec<&str> = key.split('.').collect();
            set_toml_value(&mut table, &parts, &value)?;

            std::fs::write(
                &path,
                toml::to_string_pretty(&toml::Value::Table(table))?,
            )?;
            println!("{} Set {} = {}", "✓".green(), key.bold(), value);
        }
    }
    Ok(())
}

fn set_toml_value(
    table: &mut toml::value::Table,
    keys: &[&str],
    value: &str,
) -> Result<()> {
    if keys.is_empty() {
        return Ok(());
    }
    if keys.len() == 1 {
        table.insert(
            keys[0].to_string(),
            toml::Value::String(value.to_string()),
        );
        return Ok(());
    }
    let entry = table
        .entry(keys[0].to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    if let toml::Value::Table(sub) = entry {
        set_toml_value(sub, &keys[1..], value)
    } else {
        anyhow::bail!("Key '{}' is not a table", keys[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_toml_value_simple() {
        let mut table = toml::value::Table::new();
        set_toml_value(&mut table, &["key"], "value").unwrap();
        assert_eq!(table["key"].as_str(), Some("value"));
    }

    #[test]
    fn test_set_toml_value_nested() {
        let mut table = toml::value::Table::new();
        set_toml_value(&mut table, &["section", "key"], "val").unwrap();
        let section = table["section"].as_table().unwrap();
        assert_eq!(section["key"].as_str(), Some("val"));
    }
}
