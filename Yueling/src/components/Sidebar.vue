<template>
  <aside class="sidebar" :class="{ 'active': showSidebar || !isMobile }">
    <div class="sidebar-header">
      <div class="user-info">
        <div class="avatar-container">
          <div class="avatar" @click="handleAvatarClick">
            <img v-if="currentUser?.avatar_url" :src="`${API_CONFIG.BASE_URL}${currentUser.avatar_url}?t=${Date.now()}`" :alt="currentUser.username" class="avatar-img">
            <i v-else class="fas fa-user-circle avatar-icon"></i>
            <input type="file" ref="avatarInput" style="display: none" accept="image/*" @change="handleAvatarChange">
          </div>
          <span id="user-status" class="status online">在线</span>
        </div>
        <div class="user-details">
          <h3 id="current-username">{{ currentUser?.username || '用户名' }}</h3>
        </div>
      </div>
    </div>
    
    <div class="sidebar-search">
      <div class="input-wrapper with-button">
        <i class="fas fa-search"></i>
        <input type="text" placeholder="搜索好友或群聊">
        <button id="add-friend-open" class="btn-icon search-btn" title="添加好友" @click="showAddFriend">
          <i class="fas fa-user-plus"></i>
        </button>
      </div>
    </div>
    
    <div class="sidebar-tabs">
      <button class="tab-btn" :class="{ active: activeTab === 'friends' }" data-tab="friends" @click="switchTab('friends')">
        <i class="fas fa-user-friends"></i>
        <span>好友</span>
      </button>
      <button class="tab-btn" :class="{ active: activeTab === 'groups' }" data-tab="groups" @click="switchTab('groups')">
        <i class="fas fa-users"></i>
        <span>群聊</span>
      </button>
    </div>
    
    <div class="sidebar-content">
      <!-- 好友列表 -->
      <div id="friends-tab" class="tab-content" :class="{ active: activeTab === 'friends' }">
        <div class="contact-list">
          <div v-for="friend in friends" :key="friend.id" class="contact-item" :class="{ active: selectedContact?.id === friend.id }" @click="selectContact(friend); if (isMobile) $emit('toggle-sidebar')">
            <div class="avatar">
              <img v-if="friend.avatar_url" :src="`${API_CONFIG.BASE_URL}${friend.avatar_url}?t=${Date.now()}`" :alt="friend.name" class="avatar-img">
              <i v-else class="fas fa-user-circle avatar-icon"></i>
            </div>
            <div class="contact-info">
              <div class="contact-name">
                <span>{{ friend.name }}</span>
                <span class="status" :class="friend.status">{{ friend.status }}</span>
              </div>
              <div class="contact-last-message">点击开始聊天</div>
            </div>
          </div>
        </div>
      </div>
      
      <!-- 群聊列表 -->
      <div id="groups-tab" class="tab-content" :class="{ active: activeTab === 'groups' }">
        <div class="contact-list">
          <div v-for="group in groups" :key="group.id" class="contact-item" :class="{ active: selectedContact?.id === group.id }" @click="selectContact(group); if (isMobile) $emit('toggle-sidebar')">
            <div class="avatar">
              <i class="fas fa-users"></i>
            </div>
            <div class="contact-info">
              <div class="contact-name">
                <span>{{ group.name }}</span>
                <span class="status group">群聊</span>
              </div>
              <div class="contact-last-message">{{ group.memberCount }} 名成员</div>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <div class="sidebar-footer">
      <div class="sidebar-actions">
        <button class="btn-icon vertical-theme-toggle" title="切换主题" @click="toggleTheme">
          <i v-if="!isDarkMode" class="fas fa-moon"></i>
          <i v-else class="fas fa-sun"></i>
        </button>
        <button id="friend-requests-open" class="btn-icon" title="好友请求" @click="showFriendRequests">
          <i class="fas fa-user-check"></i>
        </button>
        <button id="logout-btn" class="btn btn-secondary" @click="logout">
          <i class="fas fa-sign-out-alt"></i>
        </button>
      </div>
    </div>
  </aside>
</template>

<script lang="ts">
import { defineComponent, ref, PropType } from 'vue'
import { API_CONFIG } from '../config/api'
import { User, Contact } from '../types'

export default defineComponent({
  name: 'Sidebar',
  props: {
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
    selectedContact: {
      type: Object as PropType<Contact | null>,
      default: null
    },
    isDarkMode: {
      type: Boolean,
      default: false
    },
    isMobile: {
      type: Boolean,
      default: false
    },
    showSidebar: {
      type: Boolean,
      default: true
    }
  },
  emits: ['logout', 'show-add-friend', 'show-friend-requests', 'show-profile', 'select-contact', 'toggle-theme', 'avatar-change', 'toggle-sidebar'],
  setup(props, { emit }) {
    const activeTab = ref<'friends' | 'groups'>('friends')
    const avatarInput = ref<HTMLInputElement | null>(null)
    let clickTimer: number | null = null

    const switchTab = (tab: 'friends' | 'groups') => {
      activeTab.value = tab
    }

    const selectContact = (contact: Contact) => {
      emit('select-contact', contact)
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

    const handleAvatarClick = () => {
      if (clickTimer) {
        clearTimeout(clickTimer)
        clickTimer = null
        if (props.currentUser) {
          emit('show-profile')
        }
      } else {
        clickTimer = window.setTimeout(() => {
          if (avatarInput.value) {
            avatarInput.value.click()
          }
          clickTimer = null
        }, 300)
      }
    }

    const handleAvatarChange = (event: Event) => {
      const input = event.target as HTMLInputElement
      if (input.files && input.files.length > 0 && props.currentUser) {
        const file = input.files[0]
        emit('avatar-change', file)
        input.value = ''
      }
    }

    return {
      activeTab,
      avatarInput,
      API_CONFIG,
      switchTab,
      selectContact,
      toggleTheme,
      logout,
      showAddFriend,
      showFriendRequests,
      handleAvatarClick,
      handleAvatarChange
    }
  }
})
</script>