import { api } from './api'

export class ServerService {
    private isConnected = false
    private connectionCheckInterval: number | null = null
    private checkInterval = 5000 // 5秒检查一次

    async checkConnection(): Promise<boolean> {
        try {
            const response = await api.get('/health')
            this.isConnected = response.success
            return this.isConnected
        } catch (error) {
            console.error('Server connection check failed:', error)
            this.isConnected = false
            return false
        }
    }

    startConnectionCheck() {
        if (this.connectionCheckInterval) {
            clearInterval(this.connectionCheckInterval)
        }

        this.connectionCheckInterval = window.setInterval(async () => {
            await this.checkConnection()
        }, this.checkInterval)

        // 立即检查一次
        this.checkConnection()
    }

    stopConnectionCheck() {
        if (this.connectionCheckInterval) {
            clearInterval(this.connectionCheckInterval)
            this.connectionCheckInterval = null
        }
    }

    getConnectionStatus(): boolean {
        return this.isConnected
    }
}

export const serverService = new ServerService()