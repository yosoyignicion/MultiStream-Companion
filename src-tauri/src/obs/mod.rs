use obws::Client;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;

use crate::config::{get_secret_internal, load_config_internal};

// ----------------- ESTADO GLOBAL DE OBS -----------------

#[derive(Default)]
pub struct ObsState {
    pub client: Arc<Mutex<Option<Client>>>,
}

#[derive(serde::Serialize)]
pub struct SceneInfo {
    pub name: String,
    pub is_active: bool,
}

// -------------------- COMANDOS IPC --------------------

#[tauri::command]
pub async fn obs_connect(
    app_handle: tauri::AppHandle,
    state: State<'_, ObsState>,
) -> Result<String, String> {
    let config = load_config_internal(&app_handle)?;

    let password = get_secret_internal("obs_password").unwrap_or_default();
    let auth_password = if password.is_empty() { None } else { Some(password.as_str()) };

    eprintln!(
        "[DEBUG OBS] Iniciando handshake con host: {}, puerto: {}",
        config.obs_host, config.obs_port
    );
    eprintln!("[DEBUG OBS] Contrasena provista?: {}", !password.is_empty());

    let client = Client::connect(config.obs_host.as_str(), config.obs_port, auth_password)
        .await
        .map_err(|e| {
            let err_msg = format!("Error de conexion o handshake con OBS: {:?}", e);
            eprintln!("[DEBUG OBS ERROR] {}", err_msg);
            err_msg
        })?;

    let mut guard = state.client.lock().await;
    *guard = Some(client);

    Ok("Conectado a OBS".to_string())
}

#[tauri::command]
pub async fn obs_disconnect(state: State<'_, ObsState>) -> Result<String, String> {
    let mut guard = state.client.lock().await;
    if guard.is_some() {
        *guard = None;
        Ok("Desconectado de OBS".to_string())
    } else {
        Err("No hay conexion activa con OBS".to_string())
    }
}

#[tauri::command]
pub async fn obs_get_scenes(
    state: State<'_, ObsState>,
) -> Result<Vec<SceneInfo>, String> {
    let guard = state.client.lock().await;
    let client = guard
        .as_ref()
        .ok_or_else(|| "OBS no esta conectado".to_string())?;

    let scene_list = client
        .scenes()
        .list()
        .await
        .map_err(|e| format!("Error al obtener escenas: {}", e))?;

    let program = scene_list.current_program_scene.map(|id| id.name);
    let preview = scene_list.current_preview_scene.map(|id| id.name);

    let scenes = scene_list
        .scenes
        .into_iter()
        .map(|s| SceneInfo {
            is_active: Some(s.id.name.clone()) == program
                || Some(s.id.name.clone()) == preview,
            name: s.id.name,
        })
        .collect();

    Ok(scenes)
}

#[tauri::command]
pub async fn obs_set_scene(
    scene_name: String,
    state: State<'_, ObsState>,
) -> Result<(), String> {
    let guard = state.client.lock().await;
    let client = guard
        .as_ref()
        .ok_or_else(|| "OBS no esta conectado".to_string())?;

    client
        .scenes()
        .set_current_program_scene(scene_name.as_str())
        .await
        .map_err(|e| format!("Error al cambiar escena: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn obs_toggle_streaming(state: State<'_, ObsState>) -> Result<String, String> {
    let guard = state.client.lock().await;
    let client = guard
        .as_ref()
        .ok_or_else(|| "OBS no esta conectado".to_string())?;

    client
        .streaming()
        .toggle()
        .await
        .map_err(|e| format!("Error al alternar transmision de OBS: {}", e))?;

    Ok("Comando de transmision alternado con exito".to_string())
}
