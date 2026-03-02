import { defineStore } from 'pinia'
import { Message } from '../types'
import { messageService } from '../services/message'

export const useMessageStore = defineStore('message', {
  state: () => ({
    messages: [] as Message[],
    selectedContactId: null as string | null,
    isLoading: false,
    error: null as string | null,
    lastSyncTime: 0
  }),

  getters: {
    selectedContactMessages: (state) => {
      if (!state.selectedContactId) return []
      return state.messages.filter(msg => 
        msg.sender_id === state.selectedContactId || msg.receiver_id === state.selectedContactId
      )
    }
  },

  actions: {
    setSelectedContact(contactId: string) {
      this.selectedContactId = contactId
    },

    addMessage(message: Message) {
      // 检查是否已存在相同ID的消息
      let existingIndex = this.messages.findIndex(m => m.id === message.id)
      
      // 如果没有找到相同ID，检查是否存在内容和时间戳相近的消息（处理临时ID的情况）
      if (existingIndex === -1 && message.id && !message.id.startsWith('temp_')) {
        existingIndex = this.messages.findIndex(m => 
          m.content === message.content && 
          m.sender_id === message.sender_id && 
          m.receiver_id === message.receiver_id && 
          Math.abs(m.timestamp - message.timestamp) < 10000 // 10秒内的相同消息
        )
      }
      
      if (existingIndex !== -1) {
        // 更新现有消息
        this.messages[existingIndex] = message
      } else {
        // 添加新消息
        this.messages.push(message)
      }
      
      // 保存到本地存储（messageService已经有去重逻辑）
      messageService.saveMessage(message)
    },

    loadMessages(userId: string, contactId: string) {
      this.isLoading = true
      this.error = null
      try {
        const messages = messageService.loadMessages(userId, contactId)
        this.messages = messages
        return messages
      } catch (error: any) {
        this.error = error.message || '加载消息失败'
        return []
      } finally {
        this.isLoading = false
      }
    },

    async syncMessages(userId: string) {
      this.isLoading = true
      this.error = null
      try {
        const messages = await messageService.syncMessages(userId)
        messages.forEach((message: Message) => {
          this.addMessage(message)
        })
        return messages
      } catch (error: any) {
        this.error = error.message || '同步消息失败'
        return []
      } finally {
        this.isLoading = false
      }
    },

    async markMessagesAsDelivered(messageIds: string[]) {
      try {
        await messageService.markMessagesAsDelivered(messageIds)
        // 更新本地消息状态
        this.messages.forEach(message => {
          if (messageIds.includes(message.id || '')) {
            message.status = 'delivered'
          }
        })
      } catch (error: any) {
        this.error = error.message || '标记消息为已送达失败'
      }
    },

    async markMessagesAsRead(messageIds: string[]) {
      try {
        await messageService.markMessagesAsRead(messageIds)
        // 更新本地消息状态
        this.messages.forEach(message => {
          if (messageIds.includes(message.id || '')) {
            message.status = 'read'
          }
        })
      } catch (error: any) {
        this.error = error.message || '标记消息为已读失败'
      }
    },

    clearMessages() {
      this.messages = []
    },

    clearError() {
      this.error = null
    }
  }
})