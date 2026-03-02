<template>
  <div id="app">
    <div v-if="!serverConnected" class="server-warning">
      <i class="fas fa-exclamation-triangle"></i>
      无法连接到服务器，请检查后端是否运行在 http://localhost:2025
    </div>
    
    <!-- 登录界面 -->
    <Login 
      :isActive="currentView === 'login'" 
      @login="handleLogin"
      @switch-to-register="switchToRegister"
    />
    
    <!-- 注册界面 -->
    <Register 
      :isActive="currentView === 'register'" 
      @register="handleRegister"
      @switch-to-login="switchToLogin"
    />
    
    <!-- 聊天主界面 -->
    <Chat 
      :isActive="currentView === 'chat'"
      :currentUser="currentUser"
      :friends="friends"
      :groups="groups"
      :messages="messages"
      :isDarkMode="isDarkMode"
      @logout="logout"
      @show-add-friend="showAddFriend"
      @show-friend-requests="showFriendRequests"
      @show-profile="showProfile"
      @select-contact="selectContact"
      @send-message="sendMessage"
      @toggle-theme="toggleTheme"
      @avatar-change="handleAvatarChange"
    />
    
    <!-- 通知提示 -->
    <div id="toast" class="toast" :class="toastClass" v-if="toastMessage">{{ toastMessage }}</div>
    
    <!-- 添加好友页面 -->
    <AddFriend 
      :isActive="currentView === 'add-friend'"
      @add-friend="handleAddFriendSubmit"
      @cancel="showChat"
    />

    <!-- 好友请求页面 -->
    <FriendRequests 
      :isActive="currentView === 'friend-requests'"
      :friendRequests="friendRequests"
      @respond-request="respondToFriendRequest"
      @cancel="showChat"
    />

    <!-- 个人主页 -->
    <Profile 
      :isActive="currentView === 'profile'"
      :currentUser="currentUser"
      @save-profile="saveProfile"
      @avatar-change="handleAvatarChange"
      @cancel="showChat"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed, onMounted } from 'vue'
import { websocketService } from './services/websocket'
import Login from './components/Login.vue'
import Register from './components/Register.vue'
import Chat from './components/Chat.vue'
import AddFriend from './components/AddFriend.vue'
import FriendRequests from './components/FriendRequests.vue'
import Profile from './components/Profile.vue'
import { LoginData, RegisterData, AddFriendData, ProfileData, FriendRequestResponse, Contact, Message } from './types'
import { useUserStore } from './stores/user'
import { useFriendStore } from './stores/friend'
import { useMessageStore } from './stores/message'
import { useServerStore } from './stores/server'
import { useTheme } from './hooks'

export default defineComponent({
  name: 'App',
  components: {
    Login,
    Register,
    Chat,
    AddFriend,
    FriendRequests,
    Profile
  },
  setup() {
    const currentView = ref<'login' | 'register' | 'chat' | 'add-friend' | 'friend-requests' | 'profile'>('login')
    const toastMessage = ref('')
    const toastType = ref<'info' | 'success' | 'error'>('info')
    const { isDarkMode, toggleTheme, checkThemePreference } = useTheme()

    // 初始化 stores
    const userStore = useUserStore()
    const friendStore = useFriendStore()
    const messageStore = useMessageStore()
    const serverStore = useServerStore()

    const toastClass = computed(() => `toast ${toastType.value} ${toastMessage.value ? 'show' : ''}`)

    const showToast = (message: string, type: 'info' | 'success' | 'error' = 'info') => {
      toastMessage.value = message
      toastType.value = type
      setTimeout(() => {
        toastMessage.value = ''
      }, 3000)
    }

    const switchToRegister = () => {
      currentView.value = 'register'
    }

    const switchToLogin = () => {
      currentView.value = 'login'
    }

    const showChat = () => {
      currentView.value = 'chat'
    }

    const showAddFriend = () => {
      currentView.value = 'add-friend'
    }

    const showFriendRequests = () => {
      currentView.value = 'friend-requests'
    }

    const showProfile = () => {
      currentView.value = 'profile'
    }

    const handleLogin = async (loginData: LoginData) => {
      if (!loginData.username || !loginData.password) {
        showToast('请输入用户名和密码', 'error')
        return
      }
      try {
        const user = await userStore.login(loginData.username, loginData.password)
        showToast('登录成功', 'success')
        currentView.value = 'chat'
        // 加载好友列表和好友请求
        loadFriends()
        loadFriendRequests()
        // 同步历史消息
        await messageStore.syncMessages(user.id)
        // 连接 WebSocket
        try {
          await websocketService.connect()
          // 发送身份标识
          websocketService.send({
            type: 'identify',
            user_id: user.id
          })
          // 监听好友添加事件
          websocketService.on('friend_added', () => {
            console.log('收到好友添加通知，重新加载好友列表')
            loadFriends()
          })
          // 监听好友请求事件
          websocketService.on('friend_request', () => {
            console.log('收到好友请求通知，重新加载好友请求列表')
            loadFriendRequests()
          })
        } catch (wsError) {
          console.warn('WebSocket 连接失败:', wsError)
        }
      } catch (error: any) {
        showToast(error.message || '登录失败', 'error')
      }
    }

    const handleRegister = async (registerData: RegisterData) => {
      if (registerData.password !== registerData.confirmPassword) {
        showToast('两次输入的密码不一致', 'error')
        return
      }
      try {
        await userStore.register(registerData.username, registerData.password, registerData.confirmPassword)
        showToast('注册成功，请登录', 'success')
        currentView.value = 'login'
      } catch (error: any) {
        showToast(error.message || '注册失败', 'error')
      }
    }

    const logout = () => {
      userStore.logout()
      friendStore.$reset()
      messageStore.$reset()
      websocketService.disconnect()
      currentView.value = 'login'
      showToast('已成功登出', 'success')
    }

    const selectContact = (contact: Contact) => {
      messageStore.setSelectedContact(contact.id)
      // 加载历史消息
      if (userStore.userId) {
        messageStore.loadMessages(userStore.userId, contact.id)
      }
    }

    const sendMessage = (message: Message) => {
      messageStore.addMessage(message)
      // 发送到 WebSocket
      websocketService.send({
        type: 'message',
        ...message
      })
    }

    const handleAddFriendSubmit = async (addFriendData: AddFriendData) => {
      if (!addFriendData.username.trim()) {
        showToast('请输入好友用户名', 'error')
        return
      }
      try {
        await friendStore.addFriend(
          userStore.userId || '',
          addFriendData.username,
          addFriendData.displayName,
          addFriendData.note
        )
        showToast('好友请求已发送，等待对方确认', 'success')
        currentView.value = 'chat'
      } catch (error: any) {
        showToast(error.message || '发送好友请求失败', 'error')
      }
    }

    const respondToFriendRequest = async (responseData: FriendRequestResponse) => {
      try {
        const userId = userStore.userId
        if (!userId) {
          showToast('用户未登录', 'error')
          return
        }
        await friendStore.respondToFriendRequest(responseData.requestId, userId, responseData.response)
        showToast(`好友请求已${responseData.response === 'accepted' ? '接受' : '拒绝'}`, 'success')
      } catch (error: any) {
        showToast(error.message || '处理好友请求失败', 'error')
      }
    }

    // 保存个人主页修改
    const saveProfile = async (profileData: ProfileData) => {
      if (!userStore.currentUser) return
      try {
        await userStore.updateUserInfo(profileData.username, profileData.email)
        showToast('个人信息保存成功', 'success')
        currentView.value = 'chat'
      } catch (error: any) {
        showToast(error.message || '保存失败', 'error')
      }
    }

    // 处理头像选择
    const handleAvatarChange = async (file: File) => {
      if (!userStore.currentUser) return
      try {
        await userStore.uploadAvatar(file)
        showToast('头像上传成功', 'success')
      } catch (error: any) {
        showToast(error.message || '头像上传失败', 'error')
      }
    }

    const loadFriends = async () => {
      if (!userStore.userId) return
      await friendStore.loadFriends(userStore.userId)
    }

    const loadFriendRequests = async () => {
      if (!userStore.userId) return
      await friendStore.loadFriendRequests(userStore.userId)
    }

    const checkServer = async () => {
      console.log('开始检查服务器连接...')
      try {
        const isConnected = await serverStore.checkConnection()
        console.log('服务器连接状态:', isConnected)
        if (isConnected) {
          // 服务器连接成功，验证当前用户是否存在
          const currentUser = userStore.currentUser
          if (currentUser) {
            import('./services/auth').then(({ authService }) => {
              authService.checkUserExists(currentUser.id)
                .then(exists => {
                  if (!exists) {
                    // 用户不存在于服务器，强制退出登录
                    showToast('用户信息已失效，请重新登录', 'error')
                    logout()
                  }
                })
                .catch(err => console.error('验证用户失败:', err))
            })
          }
        } else {
          showToast('无法连接到服务器，请检查后端是否运行', 'error')
        }
      } catch (error) {
        console.error('服务器连接检查失败:', error)
        showToast('无法连接到服务器，请检查后端是否运行', 'error')
      }
    }

    onMounted(() => {
      // 检查主题偏好
      checkThemePreference()
      // 从存储加载用户
      userStore.loadUserFromStorage()
      // 启动服务器连接检查
      serverStore.startConnectionCheck()
      if (userStore.currentUser) {
        currentView.value = 'chat'
        // 加载好友列表和好友请求
        loadFriends()
        loadFriendRequests()
        // 连接 WebSocket
        websocketService.connect().then(() => {
          // 发送身份标识
          websocketService.send({
            type: 'identify',
            user_id: userStore.currentUser!.id
          })
          // 监听好友添加事件
          websocketService.on('friend_added', () => {
            console.log('收到好友添加通知，重新加载好友列表')
            loadFriends()
          })
          // 监听好友请求事件
          websocketService.on('friend_request', () => {
            console.log('收到好友请求通知，重新加载好友请求列表')
            loadFriendRequests()
          })
        }).catch(err => console.warn('WebSocket 连接失败:', err))
        
        // 定期同步消息
        const syncInterval = setInterval(async () => {
          if (userStore.currentUser) {
            await messageStore.syncMessages(userStore.currentUser.id)
          }
        }, 5000) // 每5秒同步一次
        
        // 组件卸载时清除定时器
        return () => clearInterval(syncInterval)
      }
      // 检查服务器连接
      checkServer()
    })

    return {
      currentView,
      currentUser: computed(() => userStore.currentUser),
      friends: computed(() => friendStore.friends.map(f => ({
        id: f.id,
        name: f.name,
        status: f.status,
        avatar_url: f.avatar_url
      }))),
      groups: ref([]),
      selectedContact: ref(null),
      messages: computed(() => messageStore.messages),
      toastMessage,
      toastClass,
      friendRequests: computed(() => friendStore.friendRequests),
      serverConnected: computed(() => serverStore.isConnected),
      isDarkMode,
      showToast,
      switchToRegister,
      switchToLogin,
      showChat,
      showAddFriend,
      showFriendRequests,
      showProfile,
      toggleTheme,
      handleLogin,
      handleRegister,
      logout,
      selectContact,
      sendMessage,
      handleAddFriendSubmit,
      respondToFriendRequest,
      saveProfile,
      handleAvatarChange,
    }
  },
})
</script>

<style scoped>
/* 样式已存在于全局 styles.css 中 */
</style>