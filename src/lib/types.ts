export interface UnifiedChatMessage {
  id: string;
  platform: 'twitch' | 'kick' | 'youtube';
  user: string;
  text: string;
  color: string;
}

export interface SceneInfo {
  name: string;
  is_active: boolean;
}

export interface AppConfig {
  obs_host: string;
  obs_port: number;
  stream_title_preset: string;
  twitch_username: string;
  kick_username: string;
  twitch_client_id: string;
  kick_chatroom_id: string;
}
