// 用户相关类型
export interface User {
  id: string
  username: string
  avatar_url?: string
}

// 联系人相关类型
export interface Contact {
  id: string
  name: string
  status: string
  memberCount?: number
  avatar_url?: string
}

// 好友相关类型
export interface Friend {
  id: string
  name: string
  status: 'online' | 'offline'
  avatar_url?: string
}

// 消息相关类型
export interface Message {
  id?: string
  content: string
  sender_id: string
  receiver_id: string
  timestamp: number
  status?: 'sent' | 'delivered' | 'read'
  message_type?: string
  is_read?: boolean
}

// 好友请求相关类型
export interface FriendRequest {
  id: string
  from_user_id: string
  from_username?: string
  created_at: number
}

// 登录数据类型
export interface LoginData {
  username: string
  password: string
  rememberMe: boolean
}

// 注册数据类型
export interface RegisterData {
  username: string
  password: string
  confirmPassword: string
}

// 添加好友数据类型
export interface AddFriendData {
  username: string
  displayName: string
  note: string
}

// 个人资料数据类型
export interface ProfileData {
  username: string
  bio: string
  email: string
}

// 好友请求响应数据类型
export interface FriendRequestResponse {
  requestId: string
  response: 'accepted' | 'rejected'
}

// API响应类型
export interface ApiResponse<T = any> {
  success: boolean
  message?: string
  [key: string]: any
  data?: T
}