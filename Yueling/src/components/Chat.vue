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
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, PropType } from 'vue'
import { User, Contact, Message } from '../types'
import Sidebar from './Sidebar.vue'
import ChatHeader from './ChatHeader.vue'
import ChatMessages from './ChatMessages.vue'
import ChatInput from './ChatInput.vue'
import { useMobile } from '../hooks'

export default defineComponent({
  name: 'Chat',
  components: {
    Sidebar,
    ChatHeader,
    ChatMessages,
    ChatInput
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
  setup(_, { emit }) {
    const selectedContact = ref<Contact | null>(null)
    const { isMobile } = useMobile()
    const showSidebar = ref(true)

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

    return {
      selectedContact,
      isMobile,
      showSidebar,
      toggleSidebar,
      selectContact,
      sendMessage,
      toggleTheme,
      logout,
      showAddFriend,
      showFriendRequests,
      showProfile,
      handleAvatarChange
    }
  }
})
</script>