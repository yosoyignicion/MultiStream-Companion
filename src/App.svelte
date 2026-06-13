<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { chatStore } from './lib/stores/chat.svelte';
  import type { AppConfig, SceneInfo } from './lib/types';
  import logo from './assets/logo.png';

  let config = $state<AppConfig>({
    obs_host: '127.0.0.1',
    obs_port: 4455,
    stream_title_preset: '',
    twitch_username: '',
    kick_username: '',
    twitch_client_id: '',
    kick_chatroom_id: ''
  });

  let obsPassword = $state('');
  let obsConnected = $state(false);
  let scenes = $state<SceneInfo[]>([]);
  let twitchToken = $state('');

  let obsStatusMsg = $state('OBS Desconectado');
  let chatStatusMsg = $state('Chat inactivo');
  let chatContainer: HTMLElement | null = $state(null);

  let showSettingsModal = $state(false);

  let adminMessageText = $state('');
  let writeStatusMsg = $state('');

  $effect(() => {
    if (chatStore.messages.length && chatContainer) {
      chatContainer.scrollTop = chatContainer.scrollHeight;
    }
  });

  onMount(async () => {
    chatStore.initListeners();
    try {
      config = await invoke<AppConfig>('load_config');
      obsPassword = await invoke<string>('get_secure_credential', { keyName: 'obs_password' });
      twitchToken = await invoke<string>('get_secure_credential', { keyName: 'twitch_oauth_token' });
    } catch (e) {
      console.error(e);
    }
  });

  async function saveSettings() {
    try {
      await invoke('save_config', { config });
      if (obsPassword) await invoke('save_secure_credential', { keyName: 'obs_password', secret: obsPassword });
      if (twitchToken) await invoke('save_secure_credential', { keyName: 'twitch_oauth_token', secret: twitchToken });
      showSettingsModal = false;
      alert('Configuracion guardada.');
    } catch (e) {
      alert(`Error al guardar: ${e}`);
    }
  }

  let streamActiveState = $state(false);
  let showStreamConfirmModal = $state(false);
  let globalTitle = $state('');
  let metadataStatusMsg = $state('Titulo sin sincronizar');

  async function connectOBS() {
    try {
      obsStatusMsg = 'Conectando...';
      obsStatusMsg = await invoke<string>('obs_connect');
      obsConnected = true;
      await refreshScenes();
    } catch (e) {
      obsStatusMsg = `Fallo: ${e}`;
      obsConnected = false;
    }
  }

  async function refreshScenes() {
    try { scenes = await invoke<SceneInfo[]>('obs_get_scenes'); } catch (e) { console.error(e); }
  }

  async function changeScene(name: string) {
    try { await invoke('obs_set_scene', { sceneName: name }); await refreshScenes(); } catch (e) { alert(e); }
  }

  async function handleStreamToggleClick() {
    if (streamActiveState) showStreamConfirmModal = true;
    else await triggerObsStreamToggle();
  }

  async function triggerObsStreamToggle() {
    try {
      showStreamConfirmModal = false;
      const res = await invoke<string>('obs_toggle_streaming');
      streamActiveState = !streamActiveState;
      alert(res);
    } catch (e) {
      alert(e);
    }
  }

  async function syncTitle() {
    if (!globalTitle) return;
    try {
      metadataStatusMsg = 'Actualizando...';
      metadataStatusMsg = await invoke<string>('update_global_title', { newTitle: globalTitle });
    } catch (e) {
      metadataStatusMsg = `Error: ${e}`;
    }
  }

  async function startChats() {
    try {
      chatStatusMsg = 'Iniciando sockets...';
      if (config.twitch_username) await invoke('start_chat_ingestion', { platform: 'twitch', username: config.twitch_username });
      if (config.kick_username) await invoke('start_chat_ingestion', { platform: 'kick', username: config.kick_username });
      chatStatusMsg = 'Chats en directo.';
    } catch (e) {
      chatStatusMsg = `Fallo: ${e}`;
    }
  }

  async function sendAdminMessage() {
    if (!adminMessageText) return;
    try {
      writeStatusMsg = 'Enviando...';
      const report = await invoke<string>('send_unified_chat_message', { message: adminMessageText });
      writeStatusMsg = report;
      adminMessageText = '';
      setTimeout(() => { writeStatusMsg = ''; }, 3000);
    } catch (e) {
      writeStatusMsg = `Error: ${e}`;
    }
  }

  async function moderateUser(platform: string, user: string, action: string) {
    try {
      const res = await invoke<string>('moderate_user_command', { platform, targetUsername: user, action });
      alert(res);
    } catch (e) {
      alert(e);
    }
  }
</script>

<main class="v2-layout">

  {#if showSettingsModal}
    <div class="modal-overlay">
      <div class="modal-box">
        <div class="modal-header">
          <h2>Ajustes del Sistema</h2>
          <button onclick={() => showSettingsModal = false} class="btn-close">✕</button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label>OBS Host / Port</label>
            <div class="row">
              <input type="text" bind:value={config.obs_host} placeholder="127.0.0.1" />
              <input type="number" bind:value={config.obs_port} style="width: 80px;" />
            </div>
          </div>
          <div class="form-group">
            <label>OBS Password</label>
            <input type="password" bind:value={obsPassword} placeholder="********" />
          </div>
          <hr />
          <div class="form-group">
            <label>Twitch Username</label>
            <input type="text" bind:value={config.twitch_username} placeholder="Canal de Twitch" />
          </div>
          <div class="form-group">
            <label>Twitch Client-ID</label>
            <input type="text" bind:value={config.twitch_client_id} placeholder="Client ID de Helix" />
          </div>
          <div class="form-group">
            <label>Twitch OAuth Token</label>
            <input type="password" bind:value={twitchToken} placeholder="oauth:********" />
          </div>
          <hr />
          <div class="form-group">
            <label>Kick Username</label>
            <input type="text" bind:value={config.kick_username} placeholder="Canal de Kick" />
          </div>
          <div class="form-group">
            <label>Kick Chatroom ID (Bypass)</label>
            <input type="text" bind:value={config.kick_chatroom_id} placeholder="ID numerico" />
          </div>
          <button onclick={saveSettings} class="btn-primary" style="width: 100%; margin-top: 15px;">Guardar y Aplicar</button>
        </div>
      </div>
    </div>
  {/if}

  <section class="control-panel-v2">
    <div class="panel-header-compact">
      <div class="logo-title-group">
        <img src={logo} alt="MSC Logo" class="app-logo-v2" onclick={() => showSettingsModal = true} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && (showSettingsModal = true)} />
        <h2>MultiStream Companion</h2>
      </div>
      <div class="status-indicator {obsConnected ? 'active' : ''}">OBS: {obsStatusMsg}</div>
    </div>

    {#if !obsConnected}
      <button onclick={connectOBS} class="btn-obs-large">🔌 Conectar con OBS Studio</button>
    {:else}
      <div class="horizontal-controls">
        <div class="control-card">
          <h3>Emision</h3>
          <button onclick={handleStreamToggleClick} class="btn-stream-v2 {streamActiveState ? 'streaming' : ''}">
            {streamActiveState ? '🔴 CORTAR TRANSMISION' : '🟩 INICIAR DIRECTO'}
          </button>
        </div>

        <div class="control-card">
          <h3>Titulo Unificado</h3>
          <div class="title-row">
            <input type="text" bind:value={globalTitle} placeholder="Escribe el titulo unificado..." />
            <button onclick={syncTitle} class="btn-sync">Sincronizar</button>
          </div>
          <span class="status-micro">{metadataStatusMsg}</span>
        </div>

        <div class="control-card scrollable-card">
          <h3>Escenas</h3>
          <div class="scenes-grid">
            {#each scenes as scene}
              <button onclick={() => changeScene(scene.name)} class="scene-pill {scene.is_active ? 'active' : ''}">
                {scene.name}
              </button>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    {#if showStreamConfirmModal}
      <div class="confirm-overlay">
        <div class="confirm-box">
          <h3>⚠️ Cortar transmision en vivo?</h3>
          <p>Esta accion detendra de inmediato tu senal en todas las plataformas simultaneamente.</p>
          <div class="confirm-actions">
            <button onclick={triggerObsStreamToggle} class="btn-confirm-danger">Si, Cortar</button>
            <button onclick={() => showStreamConfirmModal = false} class="btn-confirm-cancel">Cancelar</button>
          </div>
        </div>
      </div>
    {/if}
  </section>

  <section class="chat-panel-v2">
    <div class="chat-header-v2">
      <h2>Multichat de Moderacion</h2>
      <button onclick={startChats} class="btn-chat-connect">Conectar</button>
    </div>
    <span class="chat-status-micro">{chatStatusMsg}</span>

    <div bind:this={chatContainer} class="chat-container-v2">
      {#each chatStore.messages as msg (msg.id)}
        <div class="chat-message-v2 border-{msg.platform}">
          <span class="badge badge-{msg.platform}">{msg.platform.charAt(0).toUpperCase()}</span>
          <strong style="color: {msg.color}">{msg.user}:</strong>
          <span class="message-text">{msg.text}</span>
          <div class="mod-actions">
            <button onclick={() => moderateUser(msg.platform, msg.user, 'timeout')} title="Timeout" class="btn-mod">🔇</button>
            <button onclick={() => moderateUser(msg.platform, msg.user, 'ban')} title="Ban" class="btn-mod">🚫</button>
          </div>
        </div>
      {:else}
        <p class="empty-chat">Sincronizacion de chat lista. Esperando transmisiones...</p>
      {/each}
    </div>

    <div class="admin-chat-sender">
      <div class="sender-row">
        <input
          type="text"
          bind:value={adminMessageText}
          onkeydown={(e) => e.key === 'Enter' && sendAdminMessage()}
          placeholder="Mandar mensaje global como Broadcaster... [Enter]"
        />
        <button onclick={sendAdminMessage} class="btn-send-msg">Enviar</button>
      </div>
      {#if writeStatusMsg}
        <span class="status-micro-write">{writeStatusMsg}</span>
      {/if}
    </div>
  </section>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: system-ui, -apple-system, sans-serif;
    background-color: #050505;
    color: #ffffff;
    overflow: hidden;
  }

  .v2-layout {
    display: grid;
    grid-template-columns: 1fr 340px;
    height: 100vh;
    padding: 10px;
    gap: 10px;
    box-sizing: border-box;
    position: relative;
    background-color: #050505;
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background-color: rgba(0, 0, 0, 0.9);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 100;
  }
  .modal-box {
    background-color: #121212;
    border: 1px solid #ff003c;
    border-radius: 8px;
    width: 360px;
    padding: 20px;
    box-shadow: 0 10px 30px rgba(255, 0, 60, 0.15);
  }
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 15px;
    border-bottom: 1px solid #262626;
    padding-bottom: 10px;
  }
  .modal-header h2 { margin: 0; font-size: 1.1rem; color: #ff003c; }
  .btn-close { background: transparent; border: none; color: #ffffff; font-size: 1.1rem; cursor: pointer; }
  .btn-close:hover { color: #ff003c; }

  .control-panel-v2 {
    background-color: #121212;
    border-radius: 8px;
    border: 1px solid #262626;
    padding: 15px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }
  .panel-header-compact {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #262626;
    padding-bottom: 10px;
  }
  .panel-header-compact h2 { margin: 0; font-size: 1.1rem; color: #ff003c; }
  .status-indicator { font-size: 0.8rem; color: #9c9c9c; }
  .status-indicator.active { color: #ff003c; font-weight: bold; }

  .horizontal-controls {
    display: grid;
    grid-template-rows: repeat(3, auto);
    gap: 15px;
    margin-top: 15px;
    flex: 1;
    justify-content: center;
  }
  .control-card {
    background-color: #050505;
    border: 1px solid #262626;
    border-radius: 6px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .control-card h3 { margin: 0; font-size: 0.85rem; color: #9c9c9c; text-transform: uppercase; }

  .btn-stream-v2 {
    background-color: #262626;
    border: 1px solid #ff003c;
    color: #ff003c;
    padding: 10px;
    font-weight: bold;
    border-radius: 4px;
    width: 100%;
    cursor: pointer;
    transition: background-color 0.2s, color 0.2s;
  }
  .btn-stream-v2:hover {
    background-color: #ff003c;
    color: #ffffff;
  }
  .btn-stream-v2.streaming {
    background-color: #ff003c;
    color: #ffffff;
    border-color: #ffffff;
    animation: pulse 1.5s infinite;
  }

  .title-row { display: flex; gap: 5px; }
  .title-row input {
    flex: 1;
    background-color: #121212;
    border: 1px solid #262626;
    color: #ffffff;
    padding: 6px;
    border-radius: 4px;
  }
  .title-row input:focus {
    border-color: #ff003c;
    outline: none;
  }
  .btn-sync {
    background-color: #ff003c;
    color: #ffffff;
    padding: 0 12px;
    font-size: 0.85rem;
  }
  .btn-sync:hover { background-color: #cc0030; }

  .scenes-grid { display: flex; flex-wrap: wrap; gap: 5px; max-height: 110px; overflow-y: auto; }
  .scene-pill {
    background-color: #121212;
    color: #9c9c9c;
    border: 1px solid #262626;
    font-size: 0.8rem;
    padding: 6px 10px;
    border-radius: 20px;
    text-align: center;
    cursor: pointer;
    transition: border-color 0.2s;
  }
  .scene-pill:hover { border-color: #ff003c; color: #ffffff; }
  .scene-pill.active {
    background-color: #ff003c;
    color: #ffffff;
    border-color: #ff003c;
    font-weight: bold;
  }

  .chat-panel-v2 {
    background-color: #121212;
    border-radius: 8px;
    border: 1px solid #262626;
    padding: 12px;
    display: flex;
    flex-direction: column;
    height: 100%;
    box-sizing: border-box;
  }
  .chat-header-v2 { display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #262626; padding-bottom: 8px; }
  .chat-header-v2 h2 { margin: 0; font-size: 1rem; color: #ff003c; }
  .btn-chat-connect {
    background-color: #262626;
    border: 1px solid #ff003c;
    color: #ff003c;
    font-size: 0.8rem;
    padding: 4px 8px;
    transition: background-color 0.2s, color 0.2s;
  }
  .btn-chat-connect:hover {
    background-color: #ff003c;
    color: #ffffff;
  }
  .chat-status-micro { font-size: 0.75rem; color: #9c9c9c; margin-top: 4px; }

  .chat-container-v2 {
    flex: 1;
    background-color: #050505;
    border: 1px solid #262626;
    border-radius: 4px;
    padding: 8px;
    overflow-y: auto;
    margin: 8px 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .chat-message-v2 {
    padding: 4px 6px;
    border-radius: 4px;
    background-color: #121212;
    font-size: 0.85rem;
    display: flex;
    align-items: flex-start;
    gap: 6px;
    border-left: 3px solid #ffffff;
  }
  .border-twitch { border-left-color: #9146FF; }
  .border-kick { border-left-color: #53FC18; }

  .badge { font-size: 0.65rem; padding: 1px 3px; border-radius: 2px; font-weight: bold; }
  .badge-twitch { background-color: #9146FF; color: white; }
  .badge-kick { background-color: #53FC18; color: #121214; }

  .message-text { color: #ffffff; word-break: break-word; flex: 1; }

  .mod-actions { display: flex; gap: 2px; margin-left: 4px; }
  .btn-mod { background: transparent; border: none; font-size: 0.75rem; padding: 2px; cursor: pointer; border-radius: 2px; }
  .btn-mod:hover { background-color: #262626; }

  .admin-chat-sender {
    background-color: #050505;
    border: 1px solid #262626;
    border-radius: 4px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .sender-row { display: flex; gap: 5px; }
  .sender-row input {
    flex: 1;
    background-color: #121212;
    border: 1px solid #262626;
    color: #ffffff;
    padding: 6px;
    border-radius: 4px;
    font-size: 0.8rem;
  }
  .sender-row input:focus { border-color: #ff003c; outline: none; }
  .btn-send-msg {
    background-color: #ff003c;
    color: #ffffff;
    font-size: 0.8rem;
    padding: 0 10px;
    font-weight: bold;
  }
  .btn-send-msg:hover { background-color: #cc0030; }

  .form-group { display: flex; flex-direction: column; gap: 4px; margin-bottom: 10px; }
  .form-group label { font-size: 0.75rem; color: #9c9c9c; }
  .form-group input {
    background-color: #050505;
    border: 1px solid #262626;
    padding: 6px;
    border-radius: 4px;
    color: #ffffff;
    font-size: 0.85rem;
  }
  .form-group input:focus { border-color: #ff003c; outline: none; }
  .row { display: flex; gap: 5px; }

  .btn-obs-large {
    background-color: #121212;
    border: 1px solid #ff003c;
    color: #ff003c;
    font-weight: bold;
    font-size: 0.9rem;
    padding: 12px;
    width: 100%;
    margin-top: 15px;
    cursor: pointer;
    transition: background-color 0.2s, color 0.2s;
  }
  .btn-obs-large:hover {
    background-color: #ff003c;
    color: #ffffff;
  }

  .btn-primary {
    background-color: #ff003c;
    color: #ffffff;
    padding: 10px;
    margin-top: auto;
    cursor: pointer;
  }
  .btn-primary:hover { background-color: #cc0030; }

  .status-micro { font-size: 0.75rem; color: #9c9c9c; }
  .status-micro-write { font-size: 0.7rem; color: #ff003c; }
  .scrollable-card { max-height: 150px; overflow-y: auto; }

  .confirm-overlay { position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.9); display: flex; justify-content: center; align-items: center; z-index: 1000; }
  .confirm-box { background: #121212; border: 1px solid #ff003c; border-radius: 6px; padding: 15px; text-align: center; width: 300px; }
  .confirm-box h3 { color: #ff003c; margin: 0 0 10px 0; font-size: 1rem; }
  .confirm-box p { font-size: 0.8rem; color: #9c9c9c; margin-bottom: 15px; }
  .confirm-actions { display: flex; gap: 8px; justify-content: center; }
  .btn-confirm-danger { background: #ff003c; color: #ffffff; padding: 6px 12px; font-size: 0.8rem; cursor: pointer; }
  .btn-confirm-danger:hover { background-color: #cc0030; }
  .btn-confirm-cancel { background: #262626; color: #ffffff; padding: 6px 12px; font-size: 0.8rem; cursor: pointer; }
  .btn-confirm-cancel:hover { background-color: #3e3e4a; }

  @keyframes pulse { 0% { opacity: 1; } 50% { opacity: 0.7; } 100% { opacity: 1; } }

  .logo-title-group {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .app-logo-v2 {
    height: 28px;
    width: auto;
    object-fit: contain;
    filter: drop-shadow(0 0 4px rgba(255, 0, 60, 0.3));
    cursor: pointer;
    transition: filter 0.2s, transform 0.2s;
  }
  .app-logo-v2:hover {
    filter: drop-shadow(0 0 12px rgba(255, 0, 60, 0.7));
    transform: scale(1.05);
  }
</style>
