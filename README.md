# 月灵聊天应用

一个现代化的即时通讯应用，支持实时消息、好友管理、头像上传等功能。

## 技术栈

### 前端
- Vue 3
- TypeScript
- Vite
- CSS3

### 后端
- Rust
- Actix Web
- SQLite

## 项目结构

```
Yueling/
├── Yueling/            # 前端项目
│   ├── src/            # 前端源代码
│   │   ├── config/     # 配置文件
│   │   ├── services/   # 服务层
│   │   ├── App.vue     # 主组件
│   │   ├── main.ts     # 入口文件
│   │   └── styles.css  # 全局样式
│   ├── src-tauri/      # Tauri 配置（桌面应用）
│   ├── index.html      # HTML 入口
│   ├── package.json    # 前端依赖
│   └── vite.config.ts  # Vite 配置
├── server/             # 后端项目
│   ├── src/            # 后端源代码
│   │   ├── api/        # API 路由
│   │   ├── config/      # 配置文件
│   │   ├── core/        # 核心逻辑
│   │   ├── storage/     # 存储层
│   │   ├── utils/       # 工具函数
│   │   ├── lib.rs       # 库入口
│   │   └── main.rs      # 主入口
│   ├── Cargo.toml       # Rust 依赖
│   └── server.db        # SQLite 数据库
└── README.md            # 项目说明文档
```

## 安装和运行

### 前端

1. 进入前端目录
   ```bash
   cd Yueling/Yueling
   ```

2. 安装依赖
   ```bash
   npm install
   ```

3. 启动开发服务器
   ```bash
   npm run dev
   ```

4. 构建生产版本
   ```bash
   npm run build
   ```

### 后端

1. 进入后端目录
   ```bash
   cd Yueling/server
   ```

2. 安装依赖并构建
   ```bash
   cargo build
   ```

3. 启动服务器
   ```bash
   cargo run
   ```

## 功能特性

- ✅ 用户登录和注册
- ✅ 实时消息通讯
- ✅ 好友管理（添加好友、好友请求）
- ✅ 头像上传和显示
- ✅ 在线状态显示
- ✅ 深色/浅色主题切换
- ✅ 响应式设计，支持移动端

## 核心功能

### 1. 用户系统
- 注册新用户
- 用户登录（支持记住密码）
- 个人资料管理
- 头像上传

### 2. 消息系统
- 实时文本消息
- 消息状态显示（已发送、已送达、已读）
- 消息历史记录

### 3. 好友系统
- 添加好友
- 好友请求管理
- 好友列表
- 好友在线状态

### 4. UI 特性
- 现代化卡片式设计
- 流畅的动画效果
- 响应式布局
- 深色/浅色主题

## 注意事项

1. 后端服务器默认运行在 `http://localhost:2025`
2. 前端开发服务器默认运行在 `http://localhost:3000`
3. 首次运行时，后端会自动创建 SQLite 数据库文件
4. 确保后端服务器在前端之前启动，否则前端会无法连接

## 开发指南

### 前端开发
- 代码风格：使用 TypeScript 严格模式
- 组件命名：PascalCase
- 变量命名：camelCase
- 样式：使用 CSS 变量和现代 CSS 特性

### 后端开发
- 代码风格：遵循 Rust 官方风格指南
- API 设计：RESTful API
- 数据库：使用 SQLite 进行本地开发

## 许可证

MIT 许可证

## 贡献

欢迎提交 Issue 和 Pull Request！

---

© 2026 月灵聊天应用