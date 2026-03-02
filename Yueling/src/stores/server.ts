import { defineStore } from 'pinia'
import { serverService } from '../services/server'

export const useServerStore = defineStore('server', {
  state: () => ({
    isConnected: false,
    lastCheckTime: 0
  }),

  getters: {
    connectionStatus: (state) => state.isConnected
  },

  actions: {
    async checkConnection() {
      const status = await serverService.checkConnection()
      this.isConnected = status
      this.lastCheckTime = Date.now()
      return status
    },

    startConnectionCheck() {
      serverService.startConnectionCheck()
      // 监听连接状态变化
      setInterval(() => {
        this.isConnected = serverService.getConnectionStatus()
      }, 1000)
    },

    stopConnectionCheck() {
      serverService.stopConnectionCheck()
    }
  }
})