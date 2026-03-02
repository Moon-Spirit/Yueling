<template>
  <div class="chat-messages" id="chat-messages">
    <div v-for="message in messages" :key="message.id" :class="['message', message.sender_id === currentUser?.id ? 'user' : 'other']">
      <div class="message-content">{{ message.content }}</div>
      <div class="message-time">{{ formatTime(message.timestamp) }}</div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'
import { User, Message } from '../types'
import { formatTime } from '../utils/time'

export default defineComponent({
  name: 'ChatMessages',
  props: {
    currentUser: {
      type: Object as PropType<User | null>,
      default: null
    },
    messages: {
      type: Array as PropType<Message[]>,
      default: () => []
    }
  },
  setup() {
    return {
      formatTime
    }
  }
})
</script>