import { websocketService } from './websocket'

export interface Message {
    id?: string
    content: string
    sender_id: string
    receiver_id: string
    timestamp: number
    status?: 'sent' | 'delivered' | 'read'
    type?: 'message' | 'system'
}

export class MessageService {
    private messageHistory: Record<string, Message[]> = {}
    private lastSyncTime: number = 0

    constructor() {
        this.loadFromStorage()
        this.loadLastSyncTime()
    }

    sendMessage(content: string, senderId: string, receiverId: string) {
        // 生成临时ID，以便在同步时能够识别
        const tempId = `temp_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
        const message: Message = {
            id: tempId,
            content,
            sender_id: senderId,
            receiver_id: receiverId,
            timestamp: Date.now(),
            status: 'sent',
            type: 'message'
        }
        websocketService.send({
            type: 'message',
            ...message
        })
        this.saveMessage(message)
    }

    // 生成一致的消息存储键，确保顺序一致
    private generateKey(userId1: string, userId2: string): string {
        // 按字典序排序，确保无论发送方和接收方顺序如何，键都相同
        return userId1 < userId2 ? `${userId1}-${userId2}` : `${userId2}-${userId1}`
    }

    saveMessage(message: Message) {
        const key = this.generateKey(message.sender_id, message.receiver_id)
        if (!this.messageHistory[key]) {
            this.messageHistory[key] = []
        }
        
        // 检查是否已存在相同ID的消息
        let existingIndex = this.messageHistory[key].findIndex(m => m.id === message.id)
        
        // 如果没有找到相同ID，检查是否存在内容和时间戳相近的消息（处理临时ID的情况）
        if (existingIndex === -1 && message.id && !message.id.startsWith('temp_')) {
            existingIndex = this.messageHistory[key].findIndex(m => 
                m.content === message.content && 
                ((m.sender_id === message.sender_id && m.receiver_id === message.receiver_id) || 
                 (m.sender_id === message.receiver_id && m.receiver_id === message.sender_id)) && 
                Math.abs(m.timestamp - message.timestamp) < 10000 // 10秒内的相同消息
            )
        }
        
        if (existingIndex !== -1) {
            // 更新现有消息
            this.messageHistory[key][existingIndex] = message
        } else {
            // 添加新消息
            this.messageHistory[key].push(message)
        }
        
        // 限制历史记录数量
        if (this.messageHistory[key].length > 100) {
            this.messageHistory[key] = this.messageHistory[key].slice(-100)
        }
        
        // 优化：只有在消息实际变化时才更新本地存储
        localStorage.setItem('messageHistory', JSON.stringify(this.messageHistory))
    }

    loadMessages(senderId: string, receiverId: string): Message[] {
        const key = this.generateKey(senderId, receiverId)
        return this.messageHistory[key] || []
    }

    loadFromStorage() {
        const stored = localStorage.getItem('messageHistory')
        if (stored) {
            try {
                this.messageHistory = JSON.parse(stored)
            } catch (e) {
                console.error('Failed to parse message history', e)
            }
        }
    }

    loadLastSyncTime() {
        const stored = localStorage.getItem('lastSyncTime')
        if (stored) {
            try {
                this.lastSyncTime = parseInt(stored, 10)
            } catch (e) {
                console.error('Failed to parse last sync time', e)
            }
        }
    }

    saveLastSyncTime(time: number) {
        this.lastSyncTime = time
        localStorage.setItem('lastSyncTime', time.toString())
    }

    async syncMessages(userId: string) {
        try {
            const response = await fetch('http://localhost:2025/messages/sync', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    user_id: userId,
                    last_sync_time: this.lastSyncTime,
                    limit: 100
                })
            })

            if (response.ok) {
                const data = await response.json()
                if (data.success) {
                    // 处理同步到的消息
                    data.messages.forEach((message: any) => {
                        // 转换后端字段名到前端字段名
                        const frontEndMessage: Message = {
                            id: message.id,
                            content: message.content,
                            sender_id: message.sender_id,
                            receiver_id: message.receiver_id,
                            timestamp: message.created_at * 1000, // 后端是秒，前端是毫秒
                            status: message.status as 'sent' | 'delivered' | 'read',
                            type: 'message'
                        }
                        
                        // 对于发送方的消息，检查是否存在对应的临时ID消息
                        if (frontEndMessage.sender_id === userId) {
                            // 查找内容和时间戳相近的临时消息
                            let tempMessageFound = false
                            const key = this.generateKey(frontEndMessage.sender_id, frontEndMessage.receiver_id)
                            if (this.messageHistory[key]) {
                                for (let i = 0; i < this.messageHistory[key].length; i++) {
                                    const existingMsg = this.messageHistory[key][i]
                                    if (existingMsg.id?.startsWith('temp_') && 
                                        existingMsg.content === frontEndMessage.content && 
                                        existingMsg.receiver_id === frontEndMessage.receiver_id && 
                                        Math.abs(existingMsg.timestamp - frontEndMessage.timestamp) < 10000) {
                                        // 替换临时ID为正式ID
                                        existingMsg.id = frontEndMessage.id
                                        existingMsg.status = frontEndMessage.status
                                        tempMessageFound = true
                                        break
                                    }
                                }
                            }
                            // 如果没有找到临时消息，才保存为新消息
                            if (!tempMessageFound) {
                                this.saveMessage(frontEndMessage)
                            } else {
                                // 更新本地存储
                                localStorage.setItem('messageHistory', JSON.stringify(this.messageHistory))
                            }
                        } else {
                            // 对于接收方的消息，直接保存
                            this.saveMessage(frontEndMessage)
                        }
                    })
                    // 更新最后同步时间
                    this.saveLastSyncTime(data.last_sync_time)
                    return data.messages
                }
            }
        } catch (error) {
            console.error('Failed to sync messages', error)
        }
        return []
    }

    async markMessagesAsDelivered(messageIds: string[]) {
        try {
            const response = await fetch('http://localhost:2025/messages/delivered', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    message_ids: messageIds
                })
            })

            if (response.ok) {
                const data = await response.json()
                if (data.success) {
                    // 优化：只遍历需要更新的消息
                    let updated = false
                    Object.values(this.messageHistory).forEach(messages => {
                        messages.forEach(message => {
                            if (messageIds.includes(message.id || '')) {
                                message.status = 'delivered'
                                updated = true
                            }
                        })
                    })
                    // 只有在实际更新时才写入本地存储
                    if (updated) {
                        localStorage.setItem('messageHistory', JSON.stringify(this.messageHistory))
                    }
                }
            }
        } catch (error) {
            console.error('Failed to mark messages as delivered', error)
        }
    }

    async markMessagesAsRead(messageIds: string[]) {
        try {
            const response = await fetch('http://localhost:2025/messages/read', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    message_ids: messageIds
                })
            })

            if (response.ok) {
                const data = await response.json()
                if (data.success) {
                    // 优化：只遍历需要更新的消息
                    let updated = false
                    Object.values(this.messageHistory).forEach(messages => {
                        messages.forEach(message => {
                            if (messageIds.includes(message.id || '')) {
                                message.status = 'read'
                                updated = true
                            }
                        })
                    })
                    // 只有在实际更新时才写入本地存储
                    if (updated) {
                        localStorage.setItem('messageHistory', JSON.stringify(this.messageHistory))
                    }
                }
            }
        } catch (error) {
            console.error('Failed to mark messages as read', error)
        }
    }

    clearHistory() {
        this.messageHistory = {}
        localStorage.removeItem('messageHistory')
    }
}

export const messageService = new MessageService()