<template>
  <div id="app">
    <div v-if="!serverConnected" class="server-warning">
      <i class="fas fa-exclamation-triangle"></i>
      无法连接到服务器，请检查后端是否运行在 http://localhost:2025
    </div>
    <!-- 登录界面 -->
    <div id="login-container" class="container" :class="{ active: currentView === 'login' }">
      <div class="login-box">
        <div class="logo">
          <i class="fas fa-moon"></i>
          <h1>月灵聊天</h1>
        </div>
        
        <form id="login-form" class="form" @submit.prevent="handleLogin">
          <div class="form-group">
            <label for="login-username">用户名</label>
            <div class="input-wrapper">
              <i class="fas fa-user"></i>
              <input type="text" id="login-username" v-model="loginUsername" placeholder="请输入用户名" required>
            </div>
          </div>
          
          <div class="form-group">
            <label for="login-password">密码</label>
            <div class="input-wrapper">
              <i class="fas fa-lock"></i>
              <input type="password" id="login-password" v-model="loginPassword" placeholder="请输入密码" required>
              <i class="fas fa-eye-slash toggle-password" @click="togglePasswordVisibility('login')"></i>
            </div>
          </div>
          
          <div class="form-options">
            <label class="checkbox">
              <input type="checkbox" id="remember-me" v-model="rememberMe">
              <span>记住密码</span>
            </label>
            <a href="#" class="forgot-password">忘记密码？</a>
          </div>
          
          <button type="submit" class="btn btn-primary">登录</button>
        </form>
        
        <div class="form-footer">
          <p>没有账号？ <a href="#" id="switch-to-register" @click.prevent="switchToRegister">立即注册</a></p>
        </div>
      </div>
    </div>
    
    <!-- 注册界面 -->
    <div id="register-container" class="container" :class="{ active: currentView === 'register' }">
      <div class="register-box">
        <div class="logo">
          <i class="fas fa-moon"></i>
          <h1>注册月灵账号</h1>
        </div>
        
        <form id="register-form" class="form" @submit.prevent="handleRegister">
          <div class="form-group">
            <label for="register-username">用户名</label>
            <div class="input-wrapper">
              <i class="fas fa-user"></i>
              <input type="text" id="register-username" v-model="registerUsername" placeholder="请输入用户名" required>
            </div>
          </div>
          
          <div class="form-group">
            <label for="register-password">密码</label>
            <div class="input-wrapper">
              <i class="fas fa-lock"></i>
              <input type="password" id="register-password" v-model="registerPassword" placeholder="请输入密码" required>
              <i class="fas fa-eye-slash toggle-password" @click="togglePasswordVisibility('register')"></i>
            </div>
          </div>
          
          <div class="form-group">
            <label for="register-confirm-password">确认密码</label>
            <div class="input-wrapper">
              <i class="fas fa-lock"></i>
              <input type="password" id="register-confirm-password" v-model="registerConfirmPassword" placeholder="请再次输入密码" required>
              <i class="fas fa-eye-slash toggle-password" @click="togglePasswordVisibility('register-confirm')"></i>
            </div>
          </div>
          
          <button type="submit" class="btn btn-primary">注册</button>
        </form>
        
        <div class="form-footer">
          <p>已有账号？ <a href="#" id="switch-to-login" @click.prevent="switchToLogin">立即登录</a></p>
        </div>
      </div>
    </div>
    
    <!-- 聊天主界面 -->
    <div id="chat-container" class="container" :class="{ active: currentView === 'chat' }">
      <div class="chat-app">
        <!-- 侧边栏 -->
        <aside class="sidebar">
          <div class="sidebar-header">
            <div class="user-info">
              <div class="avatar-container">
                <div class="avatar" @click="showAvatarUpload">
                  <img v-if="currentUser?.avatar_url" :src="`${API_CONFIG.BASE_URL}${currentUser.avatar_url}`" :alt="currentUser.username" class="avatar-img">
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
                <div v-for="friend in friends" :key="friend.id" class="contact-item" :class="{ active: selectedContact?.id === friend.id }" @click="selectContact(friend)">
                  <div class="avatar">
                    <img v-if="friend.avatar_url" :src="`${API_CONFIG.BASE_URL}${friend.avatar_url}`" :alt="friend.name" class="avatar-img">
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
                <div v-for="group in groups" :key="group.id" class="contact-item" :class="{ active: selectedContact?.id === group.id }" @click="selectContact(group)">
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
        
        <!-- 聊天区域 -->
        <main class="chat-main">
          <div class="chat-header">
            <div class="contact-info">
              <div class="avatar">
                <img v-if="selectedContact?.avatar_url" :src="`${API_CONFIG.BASE_URL}${selectedContact.avatar_url}`" :alt="selectedContact.name" class="avatar-img">
                <i v-else class="fas fa-user-circle avatar-icon"></i>
              </div>
              <div class="contact-details">
                <h3 id="chat-contact-name">{{ selectedContact?.name || '请选择一个好友或群聊' }}</h3>
                <span id="chat-contact-status" class="status">{{ selectedContact?.status }}</span>
              </div>
            </div>
            <div class="chat-actions">
              <button class="btn-icon" title="更多">
                <i class="fas fa-ellipsis-v"></i>
              </button>
            </div>
          </div>
          
          <div class="chat-messages" id="chat-messages">
            <div v-for="message in messages" :key="message.id" :class="['message', message.sender_id === currentUser?.id ? 'user' : 'other']">
              <div class="message-content">{{ message.content }}</div>
              <div class="message-time">{{ formatTime(message.timestamp) }}</div>
            </div>
          </div>
          
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
        </main>
      </div>
    </div>
    
    <!-- 通知提示 -->
    <div id="toast" class="toast" :class="toastClass" v-if="toastMessage">{{ toastMessage }}</div>
    
    <!-- 添加好友页面 -->
    <div id="add-friend-container" class="container" :class="{ active: currentView === 'add-friend' }">
      <div class="register-box">
        <div class="logo">
          <i class="fas fa-user-plus"></i>
          <h1>添加好友</h1>
        </div>
        <form id="add-friend-form" class="form" @submit.prevent="handleAddFriendSubmit">
          <div class="form-group">
            <label for="add-friend-username">好友用户名</label>
            <div class="input-wrapper">
              <i class="fas fa-user"></i>
              <input type="text" id="add-friend-username" v-model="addFriendUsername" placeholder="输入好友用户名" required>
            </div>
          </div>

          <div class="form-group">
            <label for="add-friend-display">显示名称（可选）</label>
            <div class="input-wrapper">
              <i class="fas fa-id-badge"></i>
              <input type="text" id="add-friend-display" v-model="addFriendDisplay" placeholder="对好友显示的名称">
            </div>
          </div>

          <div class="form-group">
            <label for="add-friend-note">备注（可选）</label>
            <div class="input-wrapper">
              <i class="fas fa-sticky-note"></i>
              <input type="text" id="add-friend-note" v-model="addFriendNote" placeholder="给好友写点备注">
            </div>
          </div>

          <div class="form-buttons">
            <button type="submit" class="btn btn-primary">发送好友请求</button>
            <button type="button" id="add-friend-cancel" class="btn btn-secondary" @click="showChat">取消</button>
          </div>
        </form>
      </div>
    </div>

    <!-- 好友请求页面 -->
    <div id="friend-requests-container" class="container" :class="{ active: currentView === 'friend-requests' }">
      <div class="register-box">
        <div class="logo">
          <i class="fas fa-user-check"></i>
          <h1>好友请求</h1>
        </div>
        <div class="form">
          <div class="form-group">
            <label>收到的好友请求</label>
            <div class="input-wrapper">
              <div id="friend-requests-list" style="width:100%;">
                <div v-for="request in friendRequests" :key="request.id" class="friend-request-item">
                  <div style="display:flex;justify-content:space-between;align-items:center;">
                    <div>
                      <div><strong>{{ request.from_username || request.from_user_id || '未知用户' }}</strong></div>
                      <div style="font-size:12px;color:#666;margin-top:4px;">请求时间：{{ formatRequestTime(request.created_at) }}</div>
                    </div>
                    <div style="display:flex;gap:12px;">
                      <button class="btn btn-accept" @click="respondToFriendRequest(request.id, 'accepted')">接受</button>
                      <button class="btn btn-reject" @click="respondToFriendRequest(request.id, 'rejected')">拒绝</button>
                    </div>
                  </div>
                </div>
                <div v-if="friendRequests.length === 0" style="text-align:center;color:var(--text-secondary);padding:24px;background:var(--bg-light);border-radius:12px;margin-top:12px;">没有新的好友请求。</div>
              </div>
            </div>
          </div>

          <div class="form-buttons">
            <button type="button" id="friend-requests-cancel" class="btn btn-primary" @click="showChat">返回</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed, onMounted } from 'vue'
import { API_CONFIG } from './config/api'
import { authService } from './services/auth'
import { friendService } from './services/friend'
import { websocketService } from './services/websocket'

interface User {
  id: string
  username: string
  avatar_url?: string
}

interface Contact {
  id: string
  name: string
  status: string
  memberCount?: number
  avatar_url?: string
}

interface Message {
  id?: string
  content: string
  sender_id: string
  timestamp: number
}

interface FriendRequest {
  id: string
  from_user_id: string
  from_username?: string
  created_at: number
}

export default defineComponent({
  name: 'App',
  setup() {
    const currentView = ref<'login' | 'register' | 'chat' | 'add-friend' | 'friend-requests'>('login')
    const loginUsername = ref('')
    const loginPassword = ref('')
    const rememberMe = ref(false)
    const registerUsername = ref('')
    const registerPassword = ref('')
    const registerConfirmPassword = ref('')
    const currentUser = ref<User | null>(null)
    const activeTab = ref<'friends' | 'groups'>('friends')
    const friends = ref<Contact[]>([])
    const groups = ref<Contact[]>([])
    const selectedContact = ref<Contact | null>(null)
    const messages = ref<Message[]>([])
    const newMessage = ref('')
    const toastMessage = ref('')
    const toastType = ref<'info' | 'success' | 'error'>('info')
    const addFriendUsername = ref('')
    const addFriendDisplay = ref('')
    const addFriendNote = ref('')
    const friendRequests = ref<FriendRequest[]>([])
    const avatarInput = ref<HTMLInputElement | null>(null)

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

    const togglePasswordVisibility = (field: string) => {
      // 实现密码显示/隐藏逻辑
      const input = document.getElementById(`${field}-password`) as HTMLInputElement
      if (input) {
        input.type = input.type === 'password' ? 'text' : 'password'
      }
    }

    const handleLogin = async () => {
      if (!loginUsername.value || !loginPassword.value) {
        showToast('请输入用户名和密码', 'error')
        return
      }
      try {
        const user = await authService.login(loginUsername.value, loginPassword.value)
        currentUser.value = user
        showToast('登录成功', 'success')
        currentView.value = 'chat'
        // 加载好友列表和好友请求
        loadFriends()
        loadFriendRequests()
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

    const handleRegister = async () => {
      if (registerPassword.value !== registerConfirmPassword.value) {
        showToast('两次输入的密码不一致', 'error')
        return
      }
      try {
        await authService.register(registerUsername.value, registerPassword.value, registerConfirmPassword.value)
        showToast('注册成功，请登录', 'success')
        currentView.value = 'login'
      } catch (error: any) {
        showToast(error.message || '注册失败', 'error')
      }
    }

    const logout = () => {
      authService.logout()
      websocketService.disconnect()
      currentUser.value = null
      currentView.value = 'login'
      showToast('已成功登出', 'success')
    }

    const switchTab = (tab: 'friends' | 'groups') => {
      activeTab.value = tab
    }

    const selectContact = (contact: Contact) => {
      selectedContact.value = contact
      // 加载消息历史
    }

    const sendMessage = () => {
      if (!newMessage.value.trim() || !selectedContact.value || !currentUser.value) return
      const message: Message = {
        content: newMessage.value,
        sender_id: currentUser.value.id,
        timestamp: Date.now(),
      }
      messages.value.push(message)
      newMessage.value = ''
      // 发送到 WebSocket
    }

    const autoResizeTextarea = (event: Event) => {
      const textarea = event.target as HTMLTextAreaElement
      textarea.style.height = 'auto'
      textarea.style.height = Math.min(textarea.scrollHeight, 120) + 'px'
    }

    const handleAddFriendSubmit = async () => {
      if (!addFriendUsername.value.trim()) {
        showToast('请输入好友用户名', 'error')
        return
      }
      try {
        await friendService.addFriend(
          currentUser.value?.id || '',
          addFriendUsername.value,
          addFriendDisplay.value,
          addFriendNote.value
        )
        showToast('好友请求已发送，等待对方确认', 'success')
        currentView.value = 'chat'
      } catch (error: any) {
        showToast(error.message || '发送好友请求失败', 'error')
      }
    }

    const respondToFriendRequest = async (requestId: string, response: 'accepted' | 'rejected') => {
      try {
        const userId = currentUser.value?.id
        if (!userId) {
          showToast('用户未登录', 'error')
          return
        }
        await friendService.respondToFriendRequest(requestId, userId, response)
        friendRequests.value = friendRequests.value.filter(req => req.id !== requestId)
        showToast(`好友请求已${response === 'accepted' ? '接受' : '拒绝'}`, 'success')
      } catch (error: any) {
        showToast(error.message || '处理好友请求失败', 'error')
      }
    }

    const formatTime = (timestamp: number) => {
      return new Date(timestamp).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
    }

    const formatRequestTime = (timestamp: number) => {
      return new Date(timestamp * 1000).toLocaleString('zh-CN')
    }

    const serverConnected = ref(false)
    const isDarkMode = ref(false)

    // 切换主题
    const toggleTheme = () => {
      isDarkMode.value = !isDarkMode.value
      document.documentElement.setAttribute('data-theme', isDarkMode.value ? 'dark' : 'light')
      localStorage.setItem('theme', isDarkMode.value ? 'dark' : 'light')
    }

    // 检查主题偏好
    const checkThemePreference = () => {
      const savedTheme = localStorage.getItem('theme')
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      isDarkMode.value = savedTheme ? savedTheme === 'dark' : prefersDark
      document.documentElement.setAttribute('data-theme', isDarkMode.value ? 'dark' : 'light')
    }

    // 显示头像上传对话框
    const showAvatarUpload = () => {
      if (avatarInput.value) {
        avatarInput.value.click()
      }
    }

    // 处理头像选择
    const handleAvatarChange = async (event: Event) => {
      const input = event.target as HTMLInputElement
      if (input.files && input.files.length > 0 && currentUser.value) {
        const file = input.files[0]
        try {
          await authService.uploadAvatar(currentUser.value.id, file)
          showToast('头像上传成功', 'success')
          // 重新加载用户信息
          const userInfo = await authService.getUserInfo(currentUser.value.id)
          if (userInfo.avatar_url) {
            currentUser.value.avatar_url = userInfo.avatar_url
          }
        } catch (error: any) {
          showToast(error.message || '头像上传失败', 'error')
        }
        // 清空文件输入，以便可以重新选择相同的文件
        input.value = ''
      }
    }

    const loadFriends = async () => {
      if (!currentUser.value) return
      try {
        const loaded = await friendService.loadFriends(currentUser.value.id)
        friends.value = loaded.map(f => ({
          id: f.id,
          name: f.name,
          status: f.status,
          avatar_url: f.avatar_url
        }))
      } catch (error) {
        console.error('加载好友列表失败:', error)
      }
    }

    const loadFriendRequests = async () => {
      if (!currentUser.value) return
      try {
        const loaded = await friendService.loadFriendRequests(currentUser.value.id)
        friendRequests.value = loaded
      } catch (error) {
        console.error('加载好友请求失败:', error)
      }
    }

    const checkServer = () => {
      console.log('开始检查服务器连接...')
      // 尝试 HTTP 连接
      fetch(`${API_CONFIG.BASE_URL}/login`, { method: 'GET' })
        .then(response => {
          console.log('HTTP 响应状态:', response.status)
          if (response.ok || response.status === 405) {
            // 405 表示方法不允许，但至少服务器有响应
            serverConnected.value = true
            console.log('服务器 HTTP 连接正常')
            // 服务器连接成功，验证当前用户是否存在
            if (currentUser.value) {
              authService.checkUserExists(currentUser.value.id)
                .then(exists => {
                  if (!exists) {
                    // 用户不存在于服务器，强制退出登录
                    showToast('用户信息已失效，请重新登录', 'error')
                    authService.logout()
                    currentUser.value = null
                    currentView.value = 'login'
                    friends.value = []
                    friendRequests.value = []
                  }
                })
                .catch(err => console.error('验证用户失败:', err))
            }
            return
          }
          throw new Error(`HTTP ${response.status}`)
        })
        .catch((err) => {
          console.warn('HTTP 检测失败:', err)
          // HTTP 失败，尝试 WebSocket
          console.log('尝试 WebSocket 连接...')
          const ws = new WebSocket(API_CONFIG.WS_URL)
          const timeout = setTimeout(() => {
            console.log('WebSocket 连接超时')
            ws.close()
            serverConnected.value = false
            showToast('无法连接到服务器，请检查后端是否运行', 'error')
          }, 3000)
          ws.onopen = () => {
            console.log('WebSocket 连接成功')
            clearTimeout(timeout)
            serverConnected.value = true
            // 服务器连接成功，验证当前用户是否存在
            if (currentUser.value) {
              authService.checkUserExists(currentUser.value.id)
                .then(exists => {
                  if (!exists) {
                    showToast('用户信息已失效，请重新登录', 'error')
                    authService.logout()
                    currentUser.value = null
                    currentView.value = 'login'
                    friends.value = []
                    friendRequests.value = []
                  }
                })
                .catch(err => console.error('验证用户失败:', err))
            }
            ws.close()
          }
          ws.onerror = (e) => {
            console.error('WebSocket 错误:', e)
            clearTimeout(timeout)
            serverConnected.value = false
            showToast('无法连接到服务器，请检查后端是否运行', 'error')
          }
        })
    }

    onMounted(() => {
      // 检查主题偏好
      checkThemePreference()
      // 从存储加载用户
      const savedUser = authService.loadFromStorage()
      if (savedUser) {
        currentUser.value = savedUser
        currentView.value = 'chat'
        // 加载好友列表和好友请求
        loadFriends()
        loadFriendRequests()
        // 连接 WebSocket
        websocketService.connect().then(() => {
          // 发送身份标识
          websocketService.send({
            type: 'identify',
            user_id: savedUser.id
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
      }
      // 检查服务器连接
      checkServer()
    })

    return {
      currentView,
      loginUsername,
      loginPassword,
      rememberMe,
      registerUsername,
      registerPassword,
      registerConfirmPassword,
      currentUser,
      activeTab,
      friends,
      groups,
      selectedContact,
      messages,
      newMessage,
      toastMessage,
      toastClass,
      addFriendUsername,
      addFriendDisplay,
      addFriendNote,
      friendRequests,
      serverConnected,
      isDarkMode,
      avatarInput,
      API_CONFIG,
      showToast,
      switchToRegister,
      switchToLogin,
      showChat,
      showAddFriend,
      showFriendRequests,
      togglePasswordVisibility,
      toggleTheme,
      handleLogin,
      handleRegister,
      logout,
      switchTab,
      selectContact,
      sendMessage,
      autoResizeTextarea,
      handleAddFriendSubmit,
      respondToFriendRequest,
      formatTime,
      formatRequestTime,
      showAvatarUpload,
      handleAvatarChange,
    }
  },
})
</script>

<style scoped>
/* 样式已存在于全局 styles.css 中 */
</style>