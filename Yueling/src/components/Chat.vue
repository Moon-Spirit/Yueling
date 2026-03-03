<template>
  <div id="chat-container" class="container" :class="{ active: isActive }">
    <div class="chat-app">
      <!-- 侧边栏 -->
      <Sidebar
        :currentUser="currentUser"
        :friends="friends"
        :groups="groups"
        :selectedContact="selectedContact"
        :isDarkMode="isDarkMode"
        :isMobile="isMobile"
        :showSidebar="showSidebar"
        @logout="logout"
        @show-add-friend="showAddFriend"
        @show-friend-requests="showFriendRequests"
        @show-profile="showProfile"
        @select-contact="selectContact"
        @toggle-theme="toggleTheme"
        @avatar-change="handleAvatarChange"
        @toggle-sidebar="toggleSidebar"
      />
      
      <!-- 聊天区域 -->
      <main class="chat-main" :class="{ 'full-width': !showSidebar && isMobile }">
        <ChatHeader
          :selectedContact="selectedContact"
          :isMobile="isMobile"
          @toggle-sidebar="toggleSidebar"
          @start-voice-call="startVoiceCall"
        />
        
        <ChatMessages
          :currentUser="currentUser"
          :messages="messages"
        />
        
        <ChatInput
          :currentUser="currentUser"
          :selectedContact="selectedContact"
          @send-message="sendMessage"
        />
      </main>
    </div>
    
    <!-- 语音通话组件 -->
    <VoiceCall
      :isActive="showVoiceCall"
      :currentUser="currentUser"
      :remoteUser="selectedContact"
      @close="closeVoiceCall"
      @call-ended="handleCallEnded"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, PropType } from 'vue'
import { User, Contact, Message } from '../types'
import Sidebar from './Sidebar.vue'
import ChatHeader from './ChatHeader.vue'
import ChatMessages from './ChatMessages.vue'
import ChatInput from './ChatInput.vue'
import VoiceCall from './VoiceCall.vue'
import { voiceCallService } from '../services/voice'
import { useMobile } from '../hooks'

export default defineComponent({
  name: 'Chat',
  components: {
    Sidebar,
    ChatHeader,
    ChatMessages,
    ChatInput,
    VoiceCall
  },
  props: {
    isActive: {
      type: Boolean,
      default: false
    },
    currentUser: {
      type: Object as PropType<User | null>,
      default: null
    },
    friends: {
      type: Array as PropType<Contact[]>,
      default: () => []
    },
    groups: {
      type: Array as PropType<Contact[]>,
      default: () => []
    },
    messages: {
      type: Array as PropType<Message[]>,
      default: () => []
    },
    isDarkMode: {
      type: Boolean,
      default: false
    }
  },
  emits: ['logout', 'show-add-friend', 'show-friend-requests', 'show-profile', 'select-contact', 'send-message', 'toggle-theme', 'avatar-change'],
  setup(props, { emit }) {
    const selectedContact = ref<Contact | null>(null)
    const { isMobile } = useMobile()
    const showSidebar = ref(true)
    const showVoiceCall = ref(false)

    const toggleSidebar = () => {
      showSidebar.value = !showSidebar.value
    }

    const selectContact = (contact: Contact) => {
      selectedContact.value = contact
      emit('select-contact', contact)
    }

    const sendMessage = (message: Message) => {
      emit('send-message', message)
    }

    const toggleTheme = () => {
      emit('toggle-theme')
    }

    const logout = () => {
      emit('logout')
    }

    const showAddFriend = () => {
      emit('show-add-friend')
    }

    const showFriendRequests = () => {
      emit('show-friend-requests')
    }

    const showProfile = () => {
      emit('show-profile')
    }

    const handleAvatarChange = (file: File) => {
      emit('avatar-change', file)
    }

    const startVoiceCall = async () => {
      if (!props.currentUser || !selectedContact.value) return

      try {
        await voiceCallService.initiateCall(props.currentUser.id, selectedContact.value.id)
        showVoiceCall.value = true
      } catch (error) {
        console.error('Failed to start voice call:', error)
      }
    }

    const closeVoiceCall = () => {
      showVoiceCall.value = false
    }

    const handleCallEnded = () => {
      showVoiceCall.value = false
    }

    const handleVoiceCallStateChange = (state: any) => {
      // 当收到语音通话邀请时，自动显示语音通话组件
      if (state.status === 'connecting' && state.remoteUserId) {
        // 查找对应的联系人
        const contact = [...props.friends, ...props.groups].find(c => c.id === state.remoteUserId)
        if (contact) {
          selectedContact.value = contact
          showVoiceCall.value = true
        }
      }
    }

    // 监听语音通话服务的状态变化
    voiceCallService.onStateChange(handleVoiceCallStateChange)

    return {
      selectedContact,
      isMobile,
      showSidebar,
      showVoiceCall,
      toggleSidebar,
      selectContact,
      sendMessage,
      toggleTheme,
      logout,
      showAddFriend,
      showFriendRequests,
      showProfile,
      handleAvatarChange,
      startVoiceCall,
      closeVoiceCall,
      handleCallEnded
    }
  }
})
</script>