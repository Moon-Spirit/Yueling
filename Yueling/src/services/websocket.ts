import { API_CONFIG } from '../config/api'

type WebSocketMessage = {
    type: string;
    [key: string]: any;
}

type WebSocketCallback = (data: any) => void

export class WebSocketService {
    private connection: WebSocket | null = null
    private isConnected = false
    private reconnectAttempts = 0
    private maxReconnectAttempts = 5
    private messageCallbacks: Map<string, WebSocketCallback[]> = new Map()

    connect(): Promise<void> {
        return new Promise((resolve, reject) => {
            if (this.connection && (this.connection.readyState === WebSocket.OPEN || this.connection.readyState === WebSocket.CONNECTING)) {
                console.log('WebSocket already connected or connecting')
                resolve()
                return
            }

            try {
                this.connection = new WebSocket(API_CONFIG.WS_URL)
                this.connection.onopen = () => {
                    console.log('WebSocket connected')
                    this.isConnected = true
                    this.reconnectAttempts = 0
                    resolve()
                }
                this.connection.onmessage = (event) => {
                    try {
                        const data: WebSocketMessage = JSON.parse(event.data)
                        this.handleMessage(data)
                    } catch (e) {
                        console.error('Failed to parse WebSocket message:', e)
                    }
                }
                this.connection.onclose = () => {
                    console.log('WebSocket closed')
                    this.isConnected = false
                    this.attemptReconnect()
                }
                this.connection.onerror = (error) => {
                    console.error('WebSocket error:', error)
                    reject(error)
                }
            } catch (error) {
                reject(error)
            }
        })
    }

    disconnect() {
        if (this.connection) {
            this.connection.close()
            this.connection = null
        }
    }

    send(message: WebSocketMessage) {
        if (this.isConnected && this.connection) {
            this.connection.send(JSON.stringify(message))
        } else {
            console.error('WebSocket not connected')
        }
    }

    on(messageType: string, callback: WebSocketCallback) {
        if (!this.messageCallbacks.has(messageType)) {
            this.messageCallbacks.set(messageType, [])
        }
        this.messageCallbacks.get(messageType)!.push(callback)
    }

    off(messageType: string, callback: WebSocketCallback) {
        const callbacks = this.messageCallbacks.get(messageType)
        if (callbacks) {
            const index = callbacks.indexOf(callback)
            if (index > -1) callbacks.splice(index, 1)
        }
    }

    private handleMessage(data: WebSocketMessage) {
        const callbacks = this.messageCallbacks.get(data.type)
        if (callbacks) {
            callbacks.forEach(cb => cb(data))
        }
    }

    private attemptReconnect() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error('Max reconnection attempts reached')
            return
        }
        this.reconnectAttempts++
        setTimeout(() => {
            console.log(`Reconnecting attempt ${this.reconnectAttempts}`)
            this.connect().catch(err => console.error('Reconnection failed:', err))
        }, 3000)
    }
}

export const websocketService = new WebSocketService()