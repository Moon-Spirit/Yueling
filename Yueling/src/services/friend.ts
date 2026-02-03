// 好友服务
// 提供好友管理、好友请求处理等功能
import { apiClient } from './apiClient';

export interface Friend {
    id: string;
    name: string;
    status: 'online' | 'offline';
    avatarUrl?: string;
}

export interface FriendRequest {
    id: string;
    fromUserId: string;
    fromUsername?: string;
    createdAt: number;
}

export class FriendService {
    private friends: Friend[] = [];
    private friendRequests: FriendRequest[] = [];

    /**
     * 加载好友列表
     * @param userId 用户ID
     * @returns 好友列表
     */
    async loadFriends(userId: string): Promise<Friend[]> {
        try {
            const friends = await apiClient.getFriends(userId);
            this.friends = friends;
            this.saveFriendsToStorage();
            return this.friends;
        } catch (error) {
            console.error('加载好友列表失败:', error);
        }
        // 回退到本地存储
        const stored = this.loadFriendsFromStorage();
        this.friends = stored;
        return stored;
    }

    /**
     * 添加好友
     * @param fromUserId 发起用户ID
     * @param toUsername 目标用户名
     * @param displayName 显示名称
     * @param note 备注
     */
    async addFriend(fromUserId: string, toUsername: string, displayName?: string, note?: string): Promise<void> {
        await apiClient.addFriend({
            fromUserId,
            toUsername,
            displayName,
            note
        });
    }

    /**
     * 加载好友请求
     * @param userId 用户ID
     * @returns 好友请求列表
     */
    async loadFriendRequests(userId: string): Promise<FriendRequest[]> {
        try {
            const requests = await apiClient.getFriendRequests(userId);
            this.friendRequests = requests;
            this.saveFriendRequestsToStorage();
            return this.friendRequests;
        } catch (error) {
            console.error('加载好友请求失败:', error);
        }
        // 回退到本地存储
        const stored = this.loadFriendRequestsFromStorage();
        this.friendRequests = stored;
        return stored;
    }

    /**
     * 响应好友请求
     * @param requestId 请求ID
     * @param userId 用户ID
     * @param response 响应类型
     */
    async respondToFriendRequest(requestId: string, userId: string, response: 'accepted' | 'rejected'): Promise<void> {
        await apiClient.respondToFriendRequest(requestId, userId, response);
        // 从本地列表中移除
        this.friendRequests = this.friendRequests.filter(req => req.id !== requestId);
        this.saveFriendRequestsToStorage();
    }

    /**
     * 保存好友列表到本地存储
     */
    private saveFriendsToStorage(): void {
        localStorage.setItem('friendsList', JSON.stringify(this.friends));
    }

    /**
     * 从本地存储加载好友列表
     * @returns 好友列表
     */
    private loadFriendsFromStorage(): Friend[] {
        const stored = localStorage.getItem('friendsList');
        if (stored) {
            try {
                return JSON.parse(stored);
            } catch (e) {
                console.error('解析好友列表失败:', e);
            }
        }
        return [];
    }

    /**
     * 保存好友请求到本地存储
     */
    private saveFriendRequestsToStorage(): void {
        localStorage.setItem('receivedFriendRequests', JSON.stringify(this.friendRequests));
    }

    /**
     * 从本地存储加载好友请求
     * @returns 好友请求列表
     */
    private loadFriendRequestsFromStorage(): FriendRequest[] {
        const stored = localStorage.getItem('receivedFriendRequests');
        if (stored) {
            try {
                return JSON.parse(stored);
            } catch (e) {
                console.error('解析好友请求失败:', e);
            }
        }
        return [];
    }
}

export const friendService = new FriendService();