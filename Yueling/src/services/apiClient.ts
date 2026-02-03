// API客户端
// 封装所有API调用，实现傻瓜式调用
import { api } from './api';
import { User } from './auth';
import { Friend, FriendRequest } from './friend';

// 统一响应格式
export interface ApiResponse<T = any> {
    success: boolean;
    message?: string;
    data?: T;
}

// 登录请求参数
export interface LoginParams {
    username: string;
    password: string;
}

// 注册请求参数
export interface RegisterParams {
    username: string;
    password: string;
    confirmPassword: string;
}

// 用户信息更新参数
export interface UpdateUserParams {
    username: string;
    email: string;
}

// 添加好友参数
export interface AddFriendParams {
    fromUserId: string;
    toUsername: string;
    displayName?: string;
    note?: string;
}

// API客户端类
export class ApiClient {
    /**
     * 用户登录
     * @param params 登录参数
     * @returns 用户信息
     */
    async login(params: LoginParams): Promise<User> {
        const { username, password } = params;
        const result = await api.post('/login', { username, password });
        if (result.success) {
            return {
                id: result.user_id,
                username: result.username,
                avatarUrl: result.avatar_url || ''
            };
        } else {
            throw new Error(result.message || '登录失败');
        }
    }

    /**
     * 用户注册
     * @param params 注册参数
     */
    async register(params: RegisterParams): Promise<void> {
        const { username, password, confirmPassword } = params;
        if (password !== confirmPassword) {
            throw new Error('两次输入的密码不一致');
        }
        const result = await api.post('/register', { username, password });
        if (!result.success) {
            throw new Error(result.message || '注册失败');
        }
    }

    /**
     * 获取用户信息
     * @param userId 用户ID
     * @returns 用户信息
     */
    async getUserInfo(userId: string): Promise<{ avatarUrl?: string }> {
        const result = await api.get(`/user/${userId}`);
        if (result.success && result.user) {
            return {
                avatarUrl: result.user.avatar_url || result.user.avatarUrl
            };
        }
        return {};
    }

    /**
     * 检查用户是否存在
     * @param userId 用户ID
     * @returns 是否存在
     */
    async checkUserExists(userId: string): Promise<boolean> {
        const result = await api.post('/user/exists', { user_id: userId });
        return result.success && result.exists;
    }

    /**
     * 上传头像
     * @param userId 用户ID
     * @param file 头像文件
     * @returns 头像URL
     */
    async uploadAvatar(userId: string, file: File): Promise<string> {
        const formData = new FormData();
        formData.append('avatar', file);
        
        const result = await api.upload(`/user/${userId}/avatar`, formData);
        if (result.success && (result.avatar_url || result.avatarUrl)) {
            return result.avatar_url || result.avatarUrl;
        }
        throw new Error(result.message || '上传头像失败');
    }

    /**
     * 更新用户信息
     * @param userId 用户ID
     * @param params 更新参数
     */
    async updateUserInfo(userId: string, params: UpdateUserParams): Promise<void> {
        const { username, email } = params;
        const result = await api.put(`/user/${userId}`, { username, email });
        if (!result.success) {
            throw new Error(result.message || '更新用户信息失败');
        }
    }

    /**
     * 获取好友列表
     * @param userId 用户ID
     * @returns 好友列表
     */
    async getFriends(userId: string): Promise<Friend[]> {
        const result = await api.post('/get-friends', { user_id: userId });
        if (result.success && Array.isArray(result.friends)) {
            return result.friends.map((friend: any) => ({
                id: friend.id,
                name: friend.username,
                status: 'online' as const,
                avatarUrl: friend.avatar_url || friend.avatarUrl
            }));
        }
        return [];
    }

    /**
     * 添加好友
     * @param params 添加好友参数
     */
    async addFriend(params: AddFriendParams): Promise<void> {
        const { fromUserId, toUsername, displayName, note } = params;
        const result = await api.post('/friends/add', {
            from_user_id: fromUserId,
            to_username: toUsername,
            display_name: displayName || '',
            note: note || ''
        });
        if (!result.success) {
            throw new Error(result.message || '添加好友失败');
        }
    }

    /**
     * 获取好友请求
     * @param userId 用户ID
     * @returns 好友请求列表
     */
    async getFriendRequests(userId: string): Promise<FriendRequest[]> {
        const result = await api.post('/get-friend-requests', { user_id: userId });
        if (result.success && Array.isArray(result.requests)) {
            return result.requests.map((request: any) => ({
                id: request.id,
                fromUserId: request.from_user_id,
                fromUsername: request.from_username,
                createdAt: request.created_at
            }));
        }
        return [];
    }

    /**
     * 响应好友请求
     * @param requestId 请求ID
     * @param userId 用户ID
     * @param response 响应类型
     */
    async respondToFriendRequest(requestId: string, userId: string, response: 'accepted' | 'rejected'): Promise<void> {
        const result = await api.post('/respond-to-friend-request', {
            request_id: requestId,
            user_id: userId,
            response
        });
        if (!result.success) {
            throw new Error(result.message || '处理好友请求失败');
        }
    }
}

// 导出API客户端实例
export const apiClient = new ApiClient();
