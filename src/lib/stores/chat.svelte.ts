import { listen } from '@tauri-apps/api/event';
import type { UnifiedChatMessage } from '../types';

class ChatStore {
  messages = $state<UnifiedChatMessage[]>([]);
  maxMessages = 150;

  addMessage(msg: UnifiedChatMessage) {
    this.messages.push(msg);
    if (this.messages.length > this.maxMessages) {
      this.messages.shift();
    }
  }

  clear() {
    this.messages = [];
  }

  initListeners() {
    listen<UnifiedChatMessage>('chat-message', (event) => {
      this.addMessage(event.payload);
    });
  }
}

export const chatStore = new ChatStore();
