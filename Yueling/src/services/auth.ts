// 认证服务
// 提供用户登录、注册、信息管理等功能
import { api } from './api';

export interface User {
    id: string;
    username: string;
    avatarUrl?: string;
}

export class AuthService {
    private currentUser: User | null = null;

    /**
     * 用户登录
     * @param username 用户名
     * @param password 密码
     * @returns 用户信息
     */
    async login(username: string, password: string): Promise<User> {
        const result = await api.post('/login', { username, password });
        if (result.success) {
            // 先创建基本用户对象
            const user = { id: result.user_id, username: result.username, avatarUrl: '' };
            this.setCurrentUser(user);
            // 获取用户详细信息（包括头像）
            try {
                const userInfo = await this.getUserInfo(result.user_id);
                user.avatarUrl = userInfo.avatarUrl || '';
                this.setCurrentUser(user);
            } catch (error) {
                console.error('获取用户头像失败:', error);
            }
            return user;
        } else {
            throw new Error(result.message || '登录失败');
        }
    }

    /**
     * 用户注册
     * @param username 用户名
     * @param password 密码
     * @param confirmPassword 确认密码
     */
    async register(username: string, password: string, confirmPassword: string): Promise<void> {
        if (password !== confirmPassword) {
            throw new Error('两次输入的密码不一致');
        }
        const result = await api.post('/register', { username, password });
        if (!result.success) {
            throw new Error(result.message || '注册失败');
        }
    }

    /**
     * 用户登出
     */
    logout(): void {
        this.currentUser = null;
        localStorage.removeItem('currentUser');
    }

    /**
     * 获取当前用户
     * @returns 当前用户信息或null
     */
    getCurrentUser(): User | null {
        return this.currentUser;
    }

    /**
     * 设置当前用户
     * @param user 用户信息
     */
    setCurrentUser(user: User): void {
        this.currentUser = user;
        localStorage.setItem('currentUser', JSON.stringify(user));
    }

    /**
     * 从本地存储加载用户信息
     * @returns 用户信息或null
     */
    loadFromStorage(): User | null {
        const saved = localStorage.getItem('currentUser');
        if (saved) {
            try {
                const user = JSON.parse(saved);
                this.currentUser = user;
                return user;
            } catch (e) {
                console.error('解析存储的用户信息失败:', e);
            }
        }
        return null;
    }

    /**
     * 检查用户是否存在
     * @param userId 用户ID
     * @returns 是否存在
     */
    async checkUserExists(userId: string): Promise<boolean> {
        try {
            const result = await api.post('/user/exists', { user_id: userId });
            return result.success && result.exists;
        } catch (error) {
            console.error('检查用户存在性失败:', error);
            return false;
        }
    }

    /**
     * 获取用户信息
     * @param userId 用户ID
     * @returns 用户信息
     */
    async getUserInfo(userId: string): Promise<{ avatarUrl?: string }> {
        try {
            const result = await api.get(`/user/${userId}`);
            if (result.success && result.user) {
                return {
                    avatarUrl: result.user.avatar_url || result.user.avatarUrl
                };
            }
            return {};
        } catch (error) {
            console.error('获取用户信息失败:', error);
            return {};
        }
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
        
        try {
            const result = await api.upload(`/user/${userId}/avatar`, formData);
            if (result.success && (result.avatar_url || result.avatarUrl)) {
                const avatarUrl = result.avatar_url || result.avatarUrl;
                // 更新当前用户的头像URL
                const currentUser = this.getCurrentUser();
                if (currentUser) {
                    currentUser.avatarUrl = avatarUrl;
                    this.setCurrentUser(currentUser);
                }
                return avatarUrl;
            }
            throw new Error(result.message || '上传头像失败');
        } catch (error) {
            console.error('上传头像失败:', error);
            throw error;
        }
    }

    /**
     * 更新用户信息
     * @param userId 用户ID
     * @param username 用户名
     * @param email 邮箱
     */
    async updateUserInfo(userId: string, username: string, email: string): Promise<void> {
        try {
            const result = await api.put(`/user/${userId}`, { username, email });
            if (!result.success) {
                throw new Error(result.message || '更新用户信息失败');
            }
            // 更新当前用户的用户名
            const currentUser = this.getCurrentUser();
            if (currentUser) {
                currentUser.username = username;
                this.setCurrentUser(currentUser);
            }
        } catch (error) {
            console.error('更新用户信息失败:', error);
            throw error;
        }
    }
}

export const authService = new AuthService();