// WebSocket服务
// 提供实时通信功能
import { apiConfig } from '../config/api';

export interface WebSocketMessage {
    type: string;
    [key: string]: any;
}

export type WebSocketCallback = (data: any) => void;

export class WebSocketService {
    private connection: WebSocket | null = null;
    private isConnected = false;
    private reconnectAttempts = 0;
    private maxReconnectAttempts = 5;
    private messageCallbacks: Map<string, WebSocketCallback[]> = new Map();

    /**
     * 连接WebSocket
     * @returns Promise
     */
    connect(): Promise<void> {
        return new Promise((resolve, reject) => {
            if (this.connection && (this.connection.readyState === WebSocket.OPEN || this.connection.readyState === WebSocket.CONNECTING)) {
                console.log('WebSocket already connected or connecting');
                resolve();
                return;
            }

            try {
                this.connection = new WebSocket(apiConfig.wsUrl);
                this.connection.onopen = () => {
                    console.log('WebSocket connected');
                    this.isConnected = true;
                    this.reconnectAttempts = 0;
                    resolve();
                };
                this.connection.onmessage = (event) => {
                    try {
                        const data: WebSocketMessage = JSON.parse(event.data);
                        this.handleMessage(data);
                    } catch (e) {
                        console.error('Failed to parse WebSocket message:', e);
                    }
                };
                this.connection.onclose = () => {
                    console.log('WebSocket closed');
                    this.isConnected = false;
                    this.attemptReconnect();
                };
                this.connection.onerror = (error) => {
                    console.error('WebSocket error:', error);
                    reject(error);
                };
            } catch (error) {
                reject(error);
            }
        });
    }

    /**
     * 断开WebSocket连接
     */
    disconnect(): void {
        if (this.connection) {
            this.connection.close();
            this.connection = null;
        }
    }

    /**
     * 发送WebSocket消息
     * @param message 消息内容
     */
    send(message: WebSocketMessage): void {
        if (this.isConnected && this.connection) {
            this.connection.send(JSON.stringify(message));
        } else {
            console.error('WebSocket not connected');
        }
    }

    /**
     * 注册消息回调
     * @param messageType 消息类型
     * @param callback 回调函数
     */
    on(messageType: string, callback: WebSocketCallback): void {
        if (!this.messageCallbacks.has(messageType)) {
            this.messageCallbacks.set(messageType, []);
        }
        this.messageCallbacks.get(messageType)!.push(callback);
    }

    /**
     * 移除消息回调
     * @param messageType 消息类型
     * @param callback 回调函数
     */
    off(messageType: string, callback: WebSocketCallback): void {
        const callbacks = this.messageCallbacks.get(messageType);
        if (callbacks) {
            const index = callbacks.indexOf(callback);
            if (index > -1) callbacks.splice(index, 1);
        }
    }

    /**
     * 处理接收到的消息
     * @param data 消息数据
     */
    private handleMessage(data: WebSocketMessage): void {
        const callbacks = this.messageCallbacks.get(data.type);
        if (callbacks) {
            callbacks.forEach(cb => cb(data));
        }
    }

    /**
     * 尝试重连
     */
    private attemptReconnect(): void {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error('Max reconnection attempts reached');
            return;
        }
        this.reconnectAttempts++;
        setTimeout(() => {
            console.log(`Reconnecting attempt ${this.reconnectAttempts}`);
            this.connect().catch(err => console.error('Reconnection failed:', err));
        }, 3000);
    }
}

export const websocketService = new WebSocketService();