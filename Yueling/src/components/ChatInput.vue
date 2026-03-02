<template>
  <div class="chat-input-area">
    <div class="input-tools">
      <button class="btn-icon" title="表情">
        <i class="far fa-smile"></i>
      </button>
      <button class="btn-icon" title="图片">
        <i class="far fa-image"></i>
      </button>
      <button class="btn-icon" title="文件">
        <i class="fas fa-paperclip"></i>
      </button>
      <button class="btn-icon" title="语音">
        <i class="fas fa-microphone"></i>
      </button>
      <button class="btn-icon" title="位置">
        <i class="fas fa-map-marker-alt"></i>
      </button>
      <button class="btn-icon" title="视频">
        <i class="fas fa-video"></i>
      </button>
      <button class="btn-icon" title="更多">
        <i class="fas fa-ellipsis-h"></i>
      </button>
    </div>
    <div class="input-wrapper">
      <textarea id="message-input" v-model="newMessage" placeholder="输入消息..." rows="1" @keydown.enter.prevent="sendMessage" @input="autoResizeTextarea"></textarea>
    </div>
    <button id="send-btn" class="btn" @click="sendMessage">
      <i class="fas fa-paper-plane"></i>
    </button>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, PropType } from 'vue'
import { User, Message } from '../types'

export default defineComponent({
  name: 'ChatInput',
  props: {
    currentUser: {
      type: Object as PropType<User | null>,
      default: null
    },
    selectedContact: {
      type: Object as PropType<any | null>,
      default: null
    }
  },
  emits: ['send-message'],
  setup(props, { emit }) {
    const newMessage = ref('')

    const sendMessage = () => {
      if (!newMessage.value.trim() || !props.selectedContact || !props.currentUser) return
      const message: Message = {
        content: newMessage.value,
        sender_id: props.currentUser.id,
        receiver_id: props.selectedContact.id,
        timestamp: Date.now(),
      }
      emit('send-message', message)
      newMessage.value = ''
    }

    const autoResizeTextarea = (event: Event) => {
      const textarea = event.target as HTMLTextAreaElement
      textarea.style.height = 'auto'
      textarea.style.height = Math.min(textarea.scrollHeight, 120) + 'px'
    }

    return {
      newMessage,
      sendMessage,
      autoResizeTextarea
    }
  }
})
</script>