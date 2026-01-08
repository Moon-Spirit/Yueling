import { websocketService } from './websocket'

export interface Message {
    id?: string
    content: string
    sender_id: string
    receiver_id: string
    timestamp: number
    type?: 'message' | 'system'
}

export class MessageService {
    private messageHistory: Record<string, Message[]> = {}

    sendMessage(content: string, senderId: string, receiverId: string) {
        const message: Message = {
            content,
            sender_id: senderId,
            receiver_id: receiverId,
            timestamp: Date.now(),
            type: 'message'
        }
        websocketService.send({
            type: 'message',
            ...message
        })
        this.saveMessage(message)
    }

    saveMessage(message: Message) {
        const key = `${message.sender_id}-${message.receiver_id}`
        if (!this.messageHistory[key]) {
            this.messageHistory[key] = []
        }
        this.messageHistory[key].push(message)
        // 限制历史记录数量
        if (this.messageHistory[key].length > 100) {
            this.messageHistory[key] = this.messageHistory[key].slice(-100)
        }
        localStorage.setItem('messageHistory', JSON.stringify(this.messageHistory))
    }

    loadMessages(senderId: string, receiverId: string): Message[] {
        const key = `${senderId}-${receiverId}`
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

    clearHistory() {
        this.messageHistory = {}
        localStorage.removeItem('messageHistory')
    }
}

export const messageService = new MessageService()