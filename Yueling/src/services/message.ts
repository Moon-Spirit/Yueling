// 消息服务
// 提供消息发送、接收和历史记录管理
import { websocketService } from './websocket';

export interface Message {
    id?: string;
    content: string;
    senderId: string;
    receiverId: string;
    timestamp: number;
    type?: 'message' | 'system';
}

export class MessageService {
    private messageHistory: Record<string, Message[]> = {};

    /**
     * 发送消息
     * @param content 消息内容
     * @param senderId 发送者ID
     * @param receiverId 接收者ID
     */
    sendMessage(content: string, senderId: string, receiverId: string): void {
        const message: Message = {
            content,
            senderId,
            receiverId,
            timestamp: Date.now(),
            type: 'message'
        };
        websocketService.send({
            type: 'message',
            ...message
        });
        this.saveMessage(message);
    }

    /**
     * 保存消息
     * @param message 消息对象
     */
    saveMessage(message: Message): void {
        const key = `${message.senderId}-${message.receiverId}`;
        if (!this.messageHistory[key]) {
            this.messageHistory[key] = [];
        }
        this.messageHistory[key].push(message);
        // 限制历史记录数量
        if (this.messageHistory[key].length > 100) {
            this.messageHistory[key] = this.messageHistory[key].slice(-100);
        }
        localStorage.setItem('messageHistory', JSON.stringify(this.messageHistory));
    }

    /**
     * 加载消息历史
     * @param senderId 发送者ID
     * @param receiverId 接收者ID
     * @returns 消息列表
     */
    loadMessages(senderId: string, receiverId: string): Message[] {
        const key = `${senderId}-${receiverId}`;
        return this.messageHistory[key] || [];
    }

    /**
     * 从本地存储加载消息历史
     */
    loadFromStorage(): void {
        const stored = localStorage.getItem('messageHistory');
        if (stored) {
            try {
                this.messageHistory = JSON.parse(stored);
            } catch (e) {
                console.error('解析消息历史失败:', e);
            }
        }
    }

    /**
     * 清除消息历史
     */
    clearHistory(): void {
        this.messageHistory = {};
        localStorage.removeItem('messageHistory');
    }
}

export const messageService = new MessageService();