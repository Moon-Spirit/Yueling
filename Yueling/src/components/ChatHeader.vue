<template>
  <div class="chat-header">
    <div class="chat-header-left">
      <button v-if="isMobile" class="btn-icon menu-toggle" @click="$emit('toggle-sidebar')">
        <i class="fas fa-bars"></i>
      </button>
      <div class="contact-info">
        <div class="avatar">
          <img v-if="selectedContact?.avatar_url" :src="`${API_CONFIG.BASE_URL}${selectedContact.avatar_url}?t=${Date.now()}`" :alt="selectedContact.name" class="avatar-img">
          <i v-else class="fas fa-user-circle avatar-icon"></i>
        </div>
        <div class="contact-details">
          <h3 id="chat-contact-name">{{ selectedContact?.name || '请选择一个好友或群聊' }}</h3>
          <span id="chat-contact-status" class="status">{{ selectedContact?.status }}</span>
        </div>
      </div>
    </div>
    <div class="chat-actions">
      <button v-if="selectedContact" class="btn-icon" title="语音通话" @click="$emit('start-voice-call')">
        <i class="fas fa-phone-alt"></i>
      </button>
      <button class="btn-icon" title="更多">
        <i class="fas fa-ellipsis-v"></i>
      </button>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'
import { API_CONFIG } from '../config/api'
import { Contact } from '../types'

export default defineComponent({
  name: 'ChatHeader',
  props: {
    selectedContact: {
      type: Object as PropType<Contact | null>,
      default: null
    },
    isMobile: {
      type: Boolean,
      default: false
    }
  },
  emits: ['toggle-sidebar', 'start-voice-call'],
  setup() {
    return {
      API_CONFIG
    }
  }
})
</script>