#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod chat;
mod config;
mod obs;
mod services;

use chat::{start_chat_ingestion, ChatState};
use config::{
    delete_secret_internal, get_secret_internal, load_config_internal, save_config_internal,
    save_secret_internal, AppConfig,
};
use obs::{obs_connect, obs_disconnect, obs_get_scenes, obs_set_scene, obs_toggle_streaming, ObsState};
use services::stream_info::{
    execute_twitch_moderation, send_twitch_chat_message, update_kick_title, update_twitch_title,
};

use tauri::Emitter;

// -------------------- COMANDOS IPC --------------------

#[tauri::command]
async fn load_config(app_handle: tauri::AppHandle) -> Result<AppConfig, String> {
    load_config_internal(&app_handle)
}

#[tauri::command]
async fn save_config(app_handle: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    save_config_internal(&app_handle, config)
}

#[tauri::command]
async fn get_secure_credential(key_name: String) -> Result<String, String> {
    get_secret_internal(&key_name)
}

#[tauri::command]
async fn save_secure_credential(key_name: String, secret: String) -> Result<(), String> {
    save_secret_internal(&key_name, &secret)
}

#[tauri::command]
async fn delete_secure_credential(key_name: String) -> Result<(), String> {
    delete_secret_internal(&key_name)
}

#[tauri::command]
async fn update_global_title(
    app_handle: tauri::AppHandle,
    new_title: String,
) -> Result<String, String> {
    let config = load_config_internal(&app_handle)?;

    let mut status_reports = Vec::new();

    if !config.twitch_username.is_empty() {
        let oauth_token = config::get_secret_internal("twitch_oauth_token")?;
        if !oauth_token.is_empty() {
            match update_twitch_title(&config.twitch_username, &oauth_token, &config.twitch_client_id, &new_title).await {
                Ok(_) => status_reports.push("Twitch: Actualizado".to_string()),
                Err(e) => status_reports.push(format!("Twitch Fallo: {}", e)),
            }
        }
    }

    if !config.kick_username.is_empty() {
        match update_kick_title(&config.kick_username, &new_title).await {
            Ok(_) => status_reports.push("Kick: Sincronizado (Mock)".to_string()),
            Err(e) => status_reports.push(format!("Kick Fallo: {}", e)),
        }
    }

    Ok(status_reports.join(" | "))
}

#[tauri::command]
async fn moderate_user_command(
    app_handle: tauri::AppHandle,
    platform: String,
    target_username: String,
    action: String,
) -> Result<String, String> {
    let config = load_config_internal(&app_handle)?;

    if platform == "twitch" {
        let oauth_token = config::get_secret_internal("twitch_oauth_token")?;
        execute_twitch_moderation(
            &config.twitch_username,
            &target_username,
            &action,
            &oauth_token,
        )
        .await?;
        return Ok(format!("Moderacion enviada a Twitch para {}", target_username));
    }

    Ok(format!("Accion simulada en {} para {}", platform, target_username))
}

#[tauri::command]
async fn simulate_spam_messages(app_handle: tauri::AppHandle) -> Result<(), String> {
    let platforms = vec!["twitch", "kick"];
    let users = vec!["GamerLinux", "MintUser", "OBS_Master", "PusherFan", "RustDev"];
    let texts = vec![
        "Este multichat es increiblemente rapido!",
        "Svelte 5 con runas es una locura de velocidad",
        "Se nota delay en el stream?",
        "Baneen al spamer del canal de Kick!",
        "Unificando chats como un pro.",
        "Linux Mint dominando la escena de streaming.",
    ];

    tokio::spawn(async move {
        let mut idx = 0usize;
        for _ in 0..50 {
            let platform = platforms[idx % platforms.len()];
            let user = users[idx % users.len()];
            let text = texts[idx % texts.len()];
            let color = if platform == "twitch" { "#9146FF" } else { "#53FC18" };

            let msg = chat::kick::UnifiedChatMessage {
                id: uuid::Uuid::new_v4().to_string(),
                platform: platform.to_string(),
                user: user.to_string(),
                text: text.to_string(),
                color: color.to_string(),
            };

            let _ = app_handle.emit("chat-message", msg);
            idx += 1;
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        }
    });

    Ok(())
}

#[tauri::command]
async fn send_unified_chat_message(
    app_handle: tauri::AppHandle,
    message: String,
) -> Result<String, String> {
    let config = load_config_internal(&app_handle)?;
    let mut reports = Vec::new();

    if !config.twitch_username.is_empty() && !config.twitch_client_id.is_empty() {
        let token = config::get_secret_internal("twitch_oauth_token")?;
        if !token.is_empty() {
            match send_twitch_chat_message(
                &config.twitch_username,
                &config.twitch_client_id,
                &token,
                &message,
            )
            .await
            {
                Ok(_) => reports.push("Twitch: Enviado".to_string()),
                Err(e) => reports.push(format!("Twitch Fallo: {}", e)),
            }
        }
    }

    if !config.kick_username.is_empty() {
        println!("[DEBUG KICK WRITE] Enviando mensaje como Admin a Kick: {}", message);
        reports.push("Kick: Sincronizado (Mock)".to_string());
    }

    Ok(reports.join(" | "))
}

fn main() {
    tauri::Builder::default()
        .manage(ObsState::default())
        .manage(ChatState::default())
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            get_secure_credential,
            save_secure_credential,
            delete_secure_credential,
            obs_connect,
            obs_disconnect,
            obs_get_scenes,
            obs_set_scene,
            obs_toggle_streaming,
            start_chat_ingestion,
            update_global_title,
            moderate_user_command,
            simulate_spam_messages,
            send_unified_chat_message
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
