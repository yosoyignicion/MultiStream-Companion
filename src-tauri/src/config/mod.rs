use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;

// ----------------- CONFIGURACION NO SENSIBLE (JSON) -----------------

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct AppConfig {
    pub obs_host: String,
    pub obs_port: u16,
    pub stream_title_preset: String,
    pub twitch_username: String,
    pub kick_username: String,
    pub twitch_client_id: String,
    pub kick_chatroom_id: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            obs_host: "127.0.0.1".to_string(),
            obs_port: 4455,
            stream_title_preset: "Stream con MultiStream Companion!".to_string(),
            twitch_username: String::new(),
            kick_username: String::new(),
            twitch_client_id: String::new(),
            kick_chatroom_id: String::new(),
        }
    }
}

fn get_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
    let mut path = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| format!("No se pudo resolver el directorio de configuracion: {}", e))?;

    if !path.exists() {
        create_dir_all(&path).map_err(|e| format!("No se pudo crear el directorio: {}", e))?;
    }

    path.push("config.json");
    Ok(path)
}

pub fn load_config_internal(app_handle: &tauri::AppHandle) -> Result<AppConfig, String> {
    let path = get_config_path(app_handle)?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| e.to_string())?;

    let config: AppConfig =
        serde_json::from_str(&contents).unwrap_or_else(|_| AppConfig::default());
    Ok(config)
}

pub fn save_config_internal(
    app_handle: &tauri::AppHandle,
    config: AppConfig,
) -> Result<(), String> {
    let path = get_config_path(app_handle)?;
    let serialized =
        serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;

    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(serialized.as_bytes())
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ----------------- CREDENCIALES SEGURAS (KEYRING) -----------------

const SERVICE_NAME: &str = "com.msc.app";

pub fn save_secret_internal(key_name: &str, secret: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, key_name)
        .map_err(|e| format!("Error de inicializacion del llavero: {}", e))?;

    entry
        .set_password(secret)
        .map_err(|e| format!("Error al escribir en el llavero seguro: {}", e))?;
    Ok(())
}

pub fn get_secret_internal(key_name: &str) -> Result<String, String> {
    let entry = Entry::new(SERVICE_NAME, key_name)
        .map_err(|e| format!("Error de inicializacion del llavero: {}", e))?;

    match entry.get_password() {
        Ok(password) => Ok(password),
        Err(keyring::Error::NoEntry) => Ok(String::new()),
        Err(e) => Err(format!("Error al leer del llavero seguro: {}", e)),
    }
}

pub fn delete_secret_internal(key_name: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, key_name)
        .map_err(|e| format!("Error de inicializacion del llavero: {}", e))?;

    entry
        .delete_credential()
        .map_err(|e| format!("Error al eliminar del llavero seguro: {}", e))?;
    Ok(())
}
