import { defineStore } from 'pinia'
import { Friend, FriendRequest } from '../types'
import { friendService } from '../services/friend'

export const useFriendStore = defineStore('friend', {
  state: () => ({
    friends: [] as Friend[],
    friendRequests: [] as FriendRequest[],
    isLoading: false,
    error: null as string | null
  }),

  getters: {
    friendCount: (state) => state.friends.length,
    pendingRequestCount: (state) => state.friendRequests.length
  },

  actions: {
    async loadFriends(userId: string) {
      this.isLoading = true
      this.error = null
      try {
        const friends = await friendService.loadFriends(userId)
        this.friends = friends
        return friends
      } catch (error: any) {
        this.error = error.message || '加载好友列表失败'
        return []
      } finally {
        this.isLoading = false
      }
    },

    async addFriend(fromUserId: string, toUsername: string, displayName?: string, note?: string) {
      this.isLoading = true
      this.error = null
      try {
        await friendService.addFriend(fromUserId, toUsername, displayName, note)
      } catch (error: any) {
        this.error = error.message || '添加好友失败'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    async loadFriendRequests(userId: string) {
      this.isLoading = true
      this.error = null
      try {
        const requests = await friendService.loadFriendRequests(userId)
        this.friendRequests = requests
        return requests
      } catch (error: any) {
        this.error = error.message || '加载好友请求失败'
        return []
      } finally {
        this.isLoading = false
      }
    },

    async respondToFriendRequest(requestId: string, userId: string, response: 'accepted' | 'rejected') {
      this.isLoading = true
      this.error = null
      try {
        await friendService.respondToFriendRequest(requestId, userId, response)
        // 从本地列表中移除
        this.friendRequests = this.friendRequests.filter(req => req.id !== requestId)
      } catch (error: any) {
        this.error = error.message || '处理好友请求失败'
        throw error
      } finally {
        this.isLoading = false
      }
    },

    clearError() {
      this.error = null
    }
  }
})