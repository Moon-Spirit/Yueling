// 全局变量
let currentUser = null;
let wsConnection = null;
let isConnected = false;
let currentChatContact = null;

// API配置
const API_CONFIG = {
    tcp: {
        baseUrl: 'http://localhost:2025',
        wsUrl: 'ws://localhost:2025/ws'
    },
    currentProtocol: 'tcp' // 统一使用TCP协议
};

// DOM元素
let loginContainer;
let registerContainer;
let chatContainer;
let loginForm;
let registerForm;
let switchToRegisterBtn;
let switchToLoginBtn;
let togglePasswordBtns;
let toast;
let addFriendContainer;
let addFriendForm;
let addFriendOpenBtn;
let addFriendCancelBtn;

// 聊天相关元素
let chatMessages;
let messageInput;
let sendBtn;
let currentUsernameEl;
let logoutBtn;
let tabBtns;
let contactList;
let chatContactName;

window.addEventListener("DOMContentLoaded", async () => {
    // 初始化DOM元素
    initDOM();

    // 绑定事件
    bindEvents();

    // 检查本地存储中的用户信息（先向后端确认用户存在）
    await checkLocalStorage();

    // 应用启动时尝试连接后端（若已有连接则会被跳过）
    connectWebSocket();
});

// 初始化DOM元素
function initDOM() {
    // 容器
    loginContainer = document.getElementById('login-container');
    registerContainer = document.getElementById('register-container');
    chatContainer = document.getElementById('chat-container');
    
    // 表单
    loginForm = document.getElementById('login-form');
    registerForm = document.getElementById('register-form');
    
    // 切换按钮
    switchToRegisterBtn = document.getElementById('switch-to-register');
    switchToLoginBtn = document.getElementById('switch-to-login');
    
    // 密码切换按钮
    togglePasswordBtns = document.querySelectorAll('.toggle-password');
    
    // 通知提示
    toast = document.getElementById('toast');
    
    // 聊天相关元素
    chatMessages = document.getElementById('chat-messages');
    messageInput = document.getElementById('message-input');
    sendBtn = document.getElementById('send-btn');
    currentUsernameEl = document.getElementById('current-username');
    logoutBtn = document.getElementById('logout-btn');
    tabBtns = document.querySelectorAll('.tab-btn');
    contactList = document.querySelector('.contact-list');
    chatContactName = document.getElementById('chat-contact-name');
    addFriendContainer = document.getElementById('add-friend-container');
    addFriendForm = document.getElementById('add-friend-form');
    addFriendOpenBtn = document.getElementById('add-friend-open');
    addFriendCancelBtn = document.getElementById('add-friend-cancel');
}

// 绑定事件
function bindEvents() {
    // 表单提交事件
    loginForm.addEventListener('submit', handleLogin);
    registerForm.addEventListener('submit', handleRegister);
    
    // 切换登录/注册界面
    switchToRegisterBtn.addEventListener('click', () => {
        showContainer('register');
    });
    
    switchToLoginBtn.addEventListener('click', () => {
        showContainer('login');
    });
    
    // 密码显示/隐藏
    togglePasswordBtns.forEach(btn => {
        btn.addEventListener('click', togglePassword);
    });

    // 添加好友相关
    if (addFriendOpenBtn) {
        addFriendOpenBtn.addEventListener('click', () => {
            showContainer('add-friend');
        });
    }

    if (addFriendForm) {
        addFriendForm.addEventListener('submit', handleAddFriendSubmit);
    }

    if (addFriendCancelBtn) {
        addFriendCancelBtn.addEventListener('click', () => showContainer('chat'));
    }
    
    // 聊天相关事件
    sendBtn.addEventListener('click', sendMessage);
    messageInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            sendMessage();
        }
    });
    
    // 自动调整输入框高度
    messageInput.addEventListener('input', autoResizeTextarea);
    
    // 登出事件
    logoutBtn.addEventListener('click', logout);
    
    // 标签切换
    tabBtns.forEach(btn => {
        btn.addEventListener('click', switchTab);
    });
}

// 显示指定容器
function showContainer(containerName) {
    // 隐藏所有容器
    loginContainer.classList.remove('active');
    registerContainer.classList.remove('active');
    chatContainer.classList.remove('active');
    if (addFriendContainer) addFriendContainer.classList.remove('active');
    
    // 显示指定容器
    switch(containerName) {
        case 'login':
            loginContainer.classList.add('active');
            break;
        case 'register':
            registerContainer.classList.add('active');
            break;
        case 'chat':
            chatContainer.classList.add('active');
            break;
        case 'add-friend':
            if (addFriendContainer) addFriendContainer.classList.add('active');
            break;
    }
}

// 切换密码显示/隐藏
function togglePassword(e) {
    const btn = e.target;
    const input = btn.parentElement.querySelector('input[type="password"]');
    
    if (input.type === 'password') {
        input.type = 'text';
        btn.classList.remove('fa-eye-slash');
        btn.classList.add('fa-eye');
    } else {
        input.type = 'password';
        btn.classList.remove('fa-eye');
        btn.classList.add('fa-eye-slash');
    }
}

// 自动调整输入框高度
function autoResizeTextarea() {
    this.style.height = 'auto';
    this.style.height = Math.min(this.scrollHeight, 120) + 'px';
}

// 显示通知
function showToast(message, type = 'info') {
    toast.textContent = message;
    toast.className = `toast ${type} show`;
    
    setTimeout(() => {
        toast.classList.remove('show');
    }, 3000);
}

// 检查本地存储
function checkLocalStorage() {
    return (async () => {
        const savedUser = localStorage.getItem('currentUser');
        if (!savedUser) return;

        const parsed = JSON.parse(savedUser);

        try {
            const response = await fetch(`${API_CONFIG.tcp.baseUrl}/user/exists`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ user_id: parsed.id })
            });

            if (!response.ok) {
                // 后端返回错误（例如 4xx/5xx），不自动登录
                const txt = await response.text();
                console.error('User exists check failed:', response.status, txt);
                showToast('无法验证用户，请重新登录', 'error');
                removeUserFromStorage();
                showContainer('login');
                return;
            }

            const result = await response.json();
            if (result.success && result.exists) {
                currentUser = parsed;
                showChatInterface();
                connectWebSocket();
            } else {
                removeUserFromStorage();
                showToast('本地用户未在服务器找到，请重新登录', 'error');
                showContainer('login');
            }
        } catch (err) {
            console.error('检查用户存在性时出错：', err);
            showToast('无法连接到服务器，请检查网络', 'error');
            removeUserFromStorage();
            showContainer('login');
        }
    })();
}

// 保存用户到本地存储
function saveUserToStorage(user) {
    localStorage.setItem('currentUser', JSON.stringify(user));
}

// 从本地存储移除用户
function removeUserFromStorage() {
    localStorage.removeItem('currentUser');
}

// 处理登录
async function handleLogin(e) {
    e.preventDefault();
    
    const username = document.getElementById('login-username').value;
    const password = document.getElementById('login-password').value;
    
    try {
        // 统一使用TCP协议
        const response = await fetch(`${API_CONFIG.tcp.baseUrl}/login`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ username, password }),
        });
        if (!response.ok) {
            const text = await response.text();
            console.error('Login failed:', response.status, text);
            showToast(`登录失败：${response.status} ${text}`, 'error');
            return;
        }

        let result;
        try {
            result = await response.json();
        } catch (err) {
            const text = await response.text();
            console.error('Failed to parse login response as JSON:', err, text);
            showToast('登录失败：服务器返回了无法解析的响应', 'error');
            return;
        }

        if (result.success) {
            currentUser = {
                id: result.user_id,
                username: result.username
            };
            
            // 保存到本地存储
            saveUserToStorage(currentUser);
            
            showToast('登录成功', 'success');
            showChatInterface();
            connectWebSocket();
        } else {
            console.error('Login returned error payload:', result);
            showToast(result.message || '登录失败', 'error');
        }
    } catch (error) {
        showToast('登录失败，请检查网络连接', 'error');
        console.error('Login error:', error);
    }
}

// 处理注册
async function handleRegister(e) {
    e.preventDefault();
    
    const username = document.getElementById('register-username').value;
    const password = document.getElementById('register-password').value;
    const confirmPassword = document.getElementById('register-confirm-password').value;
    
    // 验证密码
    if (password !== confirmPassword) {
        showToast('两次输入的密码不一致', 'error');
        return;
    }
    
    try {
        // 统一使用TCP协议
        const response = await fetch(`${API_CONFIG.tcp.baseUrl}/register`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ username, password }),
        });
        // 如果返回非 2xx，先读取原始文本以便日志和提示
        if (!response.ok) {
            const text = await response.text();
            console.error('Register failed:', response.status, text);
            showToast(`注册失败：${response.status} ${text}`, 'error');
            return;
        }

        let result;
        try {
            result = await response.json();
        } catch (err) {
            const text = await response.text();
            console.error('Failed to parse register response as JSON:', err, text);
            showToast('注册失败：服务器返回了无法解析的响应', 'error');
            return;
        }

        if (result.success) {
            showToast('注册成功，请登录', 'success');
            showContainer('login');
            // 清空注册表单
            registerForm.reset();
        } else {
            console.error('Register returned error payload:', result);
            showToast(result.message || '注册失败', 'error');
        }
    } catch (error) {
        showToast('注册失败，请检查网络连接', 'error');
        console.error('Register error (network):', error);
    }
}

// 处理WebSocket消息
function handleWebSocketMessage(message) {
    try {
        const data = JSON.parse(message);
        
        switch (data.type) {
            case 'message':
                addMessageToChat(data);
                break;
            case 'user_joined':
                showToast(`${data.username} 加入了聊天`, 'info');
                break;
            case 'user_left':
                showToast(`${data.username} 离开了聊天`, 'info');
                break;
            default:
                console.log('未知消息类型:', data);
        }
    } catch (error) {
        // 处理普通文本消息
        addMessageToChat({
            content: message,
            sender: 'other',
            sender_id: 'system',
            timestamp: Date.now()
        });
    }
}

// 显示聊天界面
function showChatInterface() {
    // 更新当前用户名
    currentUsernameEl.textContent = currentUser.username;
    
    // 显示聊天容器
    showContainer('chat');
    
    // 加载好友列表（模拟数据）
    loadFriendsList();
    
    // 加载离线消息
    loadOfflineMessages();
    
    // 连接WebSocket
    connectWebSocket();
}

// 连接WebSocket
function connectWebSocket() {
    // 避免重复创建连接：若已有连接处于连接中或已打开状态，则跳过
    if (wsConnection && (wsConnection.readyState === WebSocket.OPEN || wsConnection.readyState === WebSocket.CONNECTING)) {
        console.log('WebSocket 已在连接中或已连接，跳过新连接');
        return;
    }
    try {
        const wsUrl = API_CONFIG.tcp.wsUrl;
        wsConnection = new WebSocket(wsUrl);
        
        wsConnection.onopen = () => {
            console.log('WebSocket连接成功');
            isConnected = true;
            // 仅在已有登录用户时显示连接成功提示，避免启动或注册页面产生噪音
            if (currentUser) {
                showToast('WebSocket连接成功', 'success');
            }
        };
        
        wsConnection.onmessage = (event) => {
            handleWebSocketMessage(event.data);
        };
        
        wsConnection.onclose = () => {
            console.log('WebSocket连接关闭');
            isConnected = false;
            // 仅在用户已登录时显示连接关闭提示
            if (currentUser) {
                showToast('WebSocket连接已关闭', 'info');
            } else {
                console.log('WebSocket closed (no current user)');
            }
            // 尝试重连
            setTimeout(() => {
                connectWebSocket();
            }, 5000);
        };
        
        wsConnection.onerror = (error) => {
            console.error('WebSocket错误:', error);
            isConnected = false;
            // 仅在用户已登录时显示错误提示
            if (currentUser) {
                showToast('WebSocket连接错误', 'error');
            } else {
                console.log('WebSocket error (no current user)');
            }
        };
    } catch (error) {
        console.error('WebSocket连接失败:', error);
        showToast('WebSocket连接失败', 'error');
    }
}

// 发送消息
function sendMessage() {
    const content = messageInput.value.trim();
    if (!content || !currentChatContact) return;
    
    const message = {
        type: 'message',
        content: content,
        sender_id: currentUser.id,
        sender: currentUser.username,
        receiver_id: currentChatContact.id,
        receiver: currentChatContact.name,
        timestamp: Date.now()
    };
    
    // 添加到聊天界面
    addMessageToChat(message);
    
    // 发送到WebSocket
    if (isConnected && wsConnection && wsConnection.readyState === WebSocket.OPEN) {
        wsConnection.send(JSON.stringify(message));
    }
    
    // 清空输入框
    messageInput.value = '';
    messageInput.style.height = 'auto';
    
    // 保存到本地消息历史
    saveMessageToHistory(message);
}

// 添加消息到聊天界面
function addMessageToChat(message) {
    const messageDiv = document.createElement('div');
    const isCurrentUser = message.sender_id === currentUser.id;
    
    messageDiv.className = `message ${isCurrentUser ? 'user' : 'other'}`;
    
    const messageContent = document.createElement('div');
    messageContent.textContent = message.content;
    
    const messageTime = document.createElement('div');
    messageTime.className = 'message-time';
    messageTime.textContent = formatTime(message.timestamp);
    
    messageDiv.appendChild(messageContent);
    messageDiv.appendChild(messageTime);
    
    chatMessages.appendChild(messageDiv);
    
    // 滚动到底部
    chatMessages.scrollTop = chatMessages.scrollHeight;
}

// 格式化时间
function formatTime(timestamp) {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('zh-CN', { 
        hour: '2-digit', 
        minute: '2-digit' 
    });
}

// 保存消息到历史记录
function saveMessageToHistory(message) {
    let history = JSON.parse(localStorage.getItem('messageHistory') || '{}');
    
    const chatKey = `${message.sender_id}-${message.receiver_id}`;
    if (!history[chatKey]) {
        history[chatKey] = [];
    }
    
    history[chatKey].push(message);
    
    // 限制历史消息数量
    if (history[chatKey].length > 100) {
        history[chatKey] = history[chatKey].slice(-100);
    }
    
    localStorage.setItem('messageHistory', JSON.stringify(history));
}

// 加载聊天历史
function loadChatHistory(contact) {
    chatMessages.innerHTML = '';
    
    const chatKey = `${currentUser.id}-${contact.id}`;
    const history = JSON.parse(localStorage.getItem('messageHistory') || '{}');
    
    if (history[chatKey]) {
        history[chatKey].forEach(message => {
            addMessageToChat(message);
        });
    }
}

// 加载好友列表（模拟数据）
function loadFriendsList() {
    // 清空联系人列表
    contactList.innerHTML = '';
    // 尝试从本地存储读取好友列表，优先使用后端API填充
    const stored = JSON.parse(localStorage.getItem('friendsList') || '[]');

    if (stored && stored.length > 0) {
        stored.forEach(friend => {
            const contactItem = createContactItem(friend);
            contactList.appendChild(contactItem);
        });
        return;
    }

    // 如果本地没有，则保留空列表（可在此处请求后端API）
    // fetch(`${API_CONFIG.tcp.baseUrl}/friends`).then(r => r.json()).then(data => { /* 渲染 data */ });
}

// 创建联系人项
function createContactItem(contact) {
    const div = document.createElement('div');
    div.className = 'contact-item';
    div.dataset.id = contact.id;
    
    div.innerHTML = `
        <div class="avatar">
            <i class="fas fa-user-circle"></i>
        </div>
        <div class="contact-info">
            <div class="contact-name">
                <span>${contact.name}</span>
                <span class="status ${contact.status}">${contact.status}</span>
            </div>
            <div class="contact-last-message">点击开始聊天</div>
        </div>
    `;
    
    div.addEventListener('click', () => {
        // 移除其他选中状态
        document.querySelectorAll('.contact-item').forEach(item => {
            item.classList.remove('active');
        });
        
        // 添加当前选中状态
        div.classList.add('active');
        
        // 更新当前聊天联系人
        currentChatContact = contact;
        chatContactName.textContent = contact.name;
        
        // 加载聊天历史
        loadChatHistory(contact);
    });
    
    return div;
}

// 加载离线消息
async function loadOfflineMessages() {
    try {
        const response = await fetch(`${API_CONFIG.tcp.baseUrl}/messages/unread`, {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
            },
        });
        
        const result = await response.json();
        
        if (result.success && result.messages && result.messages.length > 0) {
            showToast(`您有 ${result.messages.length} 条未读消息`, 'info');
            
            // 处理离线消息
            result.messages.forEach(message => {
                // 添加到消息历史
                saveMessageToHistory({
                    ...message,
                    sender: message.sender_id,
                    receiver: currentUser.username
                });
            });
            
            // 标记为已读
            const messageIds = result.messages.map(msg => msg.id);
            await fetch(`${API_CONFIG.tcp.baseUrl}/messages/read`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ message_ids: messageIds }),
            });
        }
    } catch (error) {
        console.error('加载离线消息失败:', error);
    }
}

// 切换标签
function switchTab(e) {
    const tab = e.target.dataset.tab;
    
    // 移除其他标签的活动状态
    tabBtns.forEach(btn => {
        btn.classList.remove('active');
    });
    
    // 添加当前标签的活动状态
    e.target.classList.add('active');
    
    // 切换内容
    document.querySelectorAll('.tab-content').forEach(content => {
        content.classList.remove('active');
    });
    
    document.getElementById(`${tab}-tab`).classList.add('active');
    
    // 加载对应内容
    if (tab === 'friends') {
        loadFriendsList();
    } else {
        loadGroupsList();
    }
}

// 加载群聊列表（模拟数据）
function loadGroupsList() {
    // 清空联系人列表
    contactList.innerHTML = '';
    
    // 模拟群聊数据
    const groups = [
        { id: 'g1', name: '群聊1', memberCount: 5 },
        { id: 'g2', name: '群聊2', memberCount: 12 },
        { id: 'g3', name: '群聊3', memberCount: 8 }
    ];
    
    groups.forEach(group => {
        const contactItem = createContactItem({
            ...group,
            status: 'group'
        });
        contactList.appendChild(contactItem);
    });
}

// 登出
function logout() {
    // 关闭WebSocket连接
    if (wsConnection) {
        wsConnection.close();
    }
    
    // 清除当前用户
    currentUser = null;
    currentChatContact = null;
    
    // 从本地存储移除
    removeUserFromStorage();
    
    // 显示登录界面
    showContainer('login');
    
    // 清空聊天记录
    chatMessages.innerHTML = '';
    chatContactName.textContent = '请选择一个好友或群聊';
    
    showToast('已成功登出', 'success');
}

// 格式化时间
function formatTime(timestamp) {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('zh-CN', { 
        hour: '2-digit', 
        minute: '2-digit' 
    });
}

// 保存消息到本地历史
function saveMessageToHistory(message) {
    let history = JSON.parse(localStorage.getItem('messageHistory') || '{}');
    
    const chatKey = `${message.sender_id}-${message.receiver_id}`;
    if (!history[chatKey]) {
        history[chatKey] = [];
    }
    
    history[chatKey].push(message);
    
    // 限制历史消息数量
    if (history[chatKey].length > 100) {
        history[chatKey] = history[chatKey].slice(-100);
    }
    
    localStorage.setItem('messageHistory', JSON.stringify(history));
}

// 处理添加好友表单提交
async function handleAddFriendSubmit(e) {
    e.preventDefault();

    const username = document.getElementById('add-friend-username').value.trim();
    const display = document.getElementById('add-friend-display').value.trim();
    const note = document.getElementById('add-friend-note').value.trim();

    if (!username) {
        showToast('请输入好友用户名', 'error');
        return;
    }

    if (!currentUser) {
        showToast('请先登录后再添加好友', 'error');
        showContainer('login');
        return;
    }

    const payload = {
        from_user_id: currentUser.id,
        to_username: username,
        display_name: display || '',
        note: note || ''
    };

    try {
        const response = await fetch(`${API_CONFIG.tcp.baseUrl}/friends/add`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(payload)
        });

        if (!response.ok) {
            const txt = await response.text();
            console.error('Add friend failed:', response.status, txt);
            // 如果后端没有实现该接口（404），回退为本地保存待发送请求
            if (response.status === 404) {
                const pending = JSON.parse(localStorage.getItem('pendingFriendRequests') || '[]');
                pending.push({ username, display, note, from_user_id: currentUser.id, created_at: Date.now() });
                localStorage.setItem('pendingFriendRequests', JSON.stringify(pending));

                // 也在本地 friendsList 中创建一个待验证的条目，便于 UI 展示
                const stored = JSON.parse(localStorage.getItem('friendsList') || '[]');
                const tempId = `pending:${username}`;
                if (!stored.find(f => f.id === tempId)) {
                    const friendData = { id: tempId, name: display || username, note: note || '', status: 'pending' };
                    stored.push(friendData);
                    localStorage.setItem('friendsList', JSON.stringify(stored));
                    const item = createContactItem(friendData);
                    contactList.appendChild(item);
                }

                showToast('后端接口不存在 (404)，已在本地保存为待发送请求', 'info');
                addFriendForm.reset();
                showContainer('chat');
                return;
            }

            showToast(`添加好友失败：${response.status} ${txt}`, 'error');
            return;
        }

        let result;
        try {
            result = await response.json();
        } catch (err) {
            const txt = await response.text();
            console.error('Failed to parse add-friend response:', err, txt);
            showToast('添加好友失败：服务器返回无法解析的响应', 'error');
            return;
        }

        if (result.success) {
            // 后端返回的好友信息优先使用
            const friendData = result.friend || {
                id: result.user_id || username,
                name: result.username || display || username,
                note: note || '',
                status: result.status || 'offline'
            };

            // 保存到本地存储
            const stored = JSON.parse(localStorage.getItem('friendsList') || '[]');
            if (!stored.find(f => f.id === friendData.id)) {
                stored.push(friendData);
                localStorage.setItem('friendsList', JSON.stringify(stored));
                // 在 UI 中添加
                const item = createContactItem(friendData);
                contactList.appendChild(item);
            }

            showToast(result.message || '好友请求发送成功', 'success');
            addFriendForm.reset();
            showContainer('chat');
        } else {
            console.error('Add friend error payload:', result);
            showToast(result.message || '添加好友失败', 'error');
        }
    } catch (error) {
        console.error('Add friend network error:', error);
        showToast('添加好友失败，请检查网络连接', 'error');
    }
}