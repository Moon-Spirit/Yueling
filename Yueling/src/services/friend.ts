import { api } from './api'

export interface Friend {
    id: string
    name: string
    status: 'online' | 'offline'
}

export interface FriendRequest {
    id: string
    from_user_id: string
    from_username?: string
    created_at: number
}

export class FriendService {
    private friends: Friend[] = []
    private friendRequests: FriendRequest[] = []

    async loadFriends(userId: string): Promise<Friend[]> {
        try {
            const result = await api.post('/get-friends', { user_id: userId })
            if (result.success && Array.isArray(result.friends)) {
                this.friends = result.friends.map((f: any) => ({
                    id: f.id,
                    name: f.username,
                    status: 'online'
                }))
                this.saveFriendsToStorage()
                return this.friends
            }
        } catch (error) {
            console.error('加载好友列表失败:', error)
        }
        // 回退到本地存储
        const stored = this.loadFriendsFromStorage()
        this.friends = stored
        return stored
    }

    async addFriend(fromUserId: string, toUsername: string, displayName?: string, note?: string): Promise<void> {
        const result = await api.post('/friends/add', {
            from_user_id: fromUserId,
            to_username: toUsername,
            display_name: displayName || '',
            note: note || ''
        })
        if (!result.success) {
            throw new Error(result.message || '添加好友失败')
        }
    }

    async loadFriendRequests(userId: string): Promise<FriendRequest[]> {
        try {
            const result = await api.post('/get-friend-requests', { user_id: userId })
            if (result.success && Array.isArray(result.requests)) {
                this.friendRequests = result.requests
                this.saveFriendRequestsToStorage()
                return this.friendRequests
            }
        } catch (error) {
            console.error('加载好友请求失败:', error)
        }
        // 回退到本地存储
        const stored = this.loadFriendRequestsFromStorage()
        this.friendRequests = stored
        return stored
    }

    async respondToFriendRequest(requestId: string, userId: string, response: 'accepted' | 'rejected'): Promise<void> {
        const result = await api.post('/respond-to-friend-request', {
            request_id: requestId,
            user_id: userId,
            response
        })
        if (!result.success) {
            throw new Error(result.message || '处理好友请求失败')
        }
        // 从本地列表中移除
        this.friendRequests = this.friendRequests.filter(req => req.id !== requestId)
        this.saveFriendRequestsToStorage()
    }

    private saveFriendsToStorage() {
        localStorage.setItem('friendsList', JSON.stringify(this.friends))
    }

    private loadFriendsFromStorage(): Friend[] {
        const stored = localStorage.getItem('friendsList')
        if (stored) {
            try {
                return JSON.parse(stored)
            } catch (e) {
                console.error('Failed to parse friends list', e)
            }
        }
        return []
    }

    private saveFriendRequestsToStorage() {
        localStorage.setItem('receivedFriendRequests', JSON.stringify(this.friendRequests))
    }

    private loadFriendRequestsFromStorage(): FriendRequest[] {
        const stored = localStorage.getItem('receivedFriendRequests')
        if (stored) {
            try {
                return JSON.parse(stored)
            } catch (e) {
                console.error('Failed to parse friend requests', e)
            }
        }
        return []
    }
}

export const friendService = new FriendService()