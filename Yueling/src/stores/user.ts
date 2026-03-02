import { defineStore } from 'pinia'
import { User } from '../types'
import { authService } from '../services/auth'

export const useUserStore = defineStore('user', {
  state: () => ({
    currentUser: null as User | null,
    isLoading: false,
    error: null as string | null
  }),

  getters: {
    isLoggedIn: (state) => !!state.currentUser,
    userId: (state) => state.currentUser?.id || null,
    username: (state) => state.currentUser?.username || ''
  },

  actions: {
    async login(username: string, password: string) {
      this.isLoading = true
      this.error = null
      try {
        const user = await authService.login(username, password)
        this.currentUser = user
        return user
      } catch (error: any) {
        this.error = error.message || '登录失败'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async register(username: string, password: string, confirmPassword: string) {
      this.isLoading = true
      this.error = null
      try {
        await authService.register(username, password, confirmPassword)
      } catch (error: any) {
        this.error = error.message || '注册失败'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    logout() {
      authService.logout()
      this.currentUser = null
    },

    loadUserFromStorage() {
      const user = authService.loadFromStorage()
      if (user) {
        this.currentUser = user
      }
    },

    async updateUserInfo(username: string, email: string) {
      if (!this.currentUser) return
      this.isLoading = true
      this.error = null
      try {
        await authService.updateUserInfo(this.currentUser.id, username, email)
        if (username) {
          this.currentUser.username = username
        }
      } catch (error: any) {
        this.error = error.message || '更新用户信息失败'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async uploadAvatar(file: File) {
      if (!this.currentUser) return
      this.isLoading = true
      this.error = null
      try {
        const avatarUrl = await authService.uploadAvatar(this.currentUser.id, file)
        if (avatarUrl) {
          this.currentUser.avatar_url = avatarUrl
        }
        return avatarUrl
      } catch (error: any) {
        this.error = error.message || '上传头像失败'
        throw error
      } finally {
        this.isLoading = false
      }
    }
  }
})