use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::json;

// ----------------- CONTROL DE METADATA (TITULOS) -----------------

pub async fn update_twitch_title(
    username: &str,
    oauth_token: &str,
    client_id: &str,
    new_title: &str,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", oauth_token)).unwrap(),
    );
    headers.insert(
        "Client-Id",
        HeaderValue::from_str(client_id).unwrap(),
    );

    let user_url = format!(
        "https://api.twitch.tv/helix/users?login={}",
        username.to_lowercase()
    );
    let res = client
        .get(&user_url)
        .headers(headers.clone())
        .send()
        .await
        .map_err(|e| format!("Fallo al consultar ID de Twitch: {}", e))?;

    let user_data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

    if let Some(msg) = user_data["message"].as_str() {
        return Err(format!("Twitch API Error: {}", msg));
    }

    let broadcaster_id = user_data["data"][0]["id"]
        .as_str()
        .ok_or_else(|| "No se encontro el ID de usuario de Twitch. Verifica tu Username y Client-ID.".to_string())?;

    let update_url = format!(
        "https://api.twitch.tv/helix/channels?broadcaster_id={}",
        broadcaster_id
    );
    let body = json!({ "title": new_title });

    let patch_res = client
        .patch(&update_url)
        .headers(headers)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Fallo al enviar PATCH a Twitch: {}", e))?;

    if patch_res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Twitch respondio con error: {}", patch_res.status()))
    }
}

pub async fn update_kick_title(_username: &str, _new_title: &str) -> Result<(), String> {
    eprintln!("Simulando cambio de titulo en Kick a: {}", _new_title);
    Ok(())
}

// ----------------- ACCIONES DE MODERACION -----------------

pub async fn send_twitch_chat_message(
    broadcaster_username: &str,
    client_id: &str,
    oauth_token: &str,
    message_text: &str,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", oauth_token)).unwrap(),
    );
    headers.insert("Client-Id", HeaderValue::from_str(client_id).unwrap());

    let user_url = format!(
        "https://api.twitch.tv/helix/users?login={}",
        broadcaster_username.to_lowercase()
    );
    let res = client
        .get(&user_url)
        .headers(headers.clone())
        .send()
        .await
        .map_err(|e| format!("Error al buscar ID: {}", e))?;

    let user_data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let broadcaster_id = user_data["data"][0]["id"]
        .as_str()
        .ok_or_else(|| "No se encontro el ID".to_string())?;

    let chat_url = "https://api.twitch.tv/helix/chat/messages";
    let body = json!({
        "broadcaster_id": broadcaster_id,
        "sender_id": broadcaster_id,
        "message": message_text
    });

    let chat_res = client
        .post(chat_url)
        .headers(headers)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Error de red al enviar chat: {}", e))?;

    if chat_res.status().is_success() {
        Ok(())
    } else {
        let err_json: serde_json::Value = chat_res.json().await.unwrap_or_default();
        Err(format!("Twitch rechazo mensaje: {}", err_json["message"]))
    }
}

pub async fn execute_twitch_moderation(
    _broadcaster_username: &str,
    target_username: &str,
    action: &str,
    _oauth_token: &str,
) -> Result<(), String> {
    eprintln!(
        "Ejecutando accion de moderacion en Twitch: [{}] al usuario [{}]",
        action, target_username
    );
    Ok(())
}
