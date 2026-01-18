# 月灵聊天应用

![月灵聊天应用](https://neeko-copilot.bytedance.net/api/text2image?prompt=modern%20chat%20application%20interface%20with%20dark%20theme%2C%20sidebar%20with%20user%20avatar%20and%20friends%20list%2C%20chat%20window%20with%20messages&size=landscape_16_9)

[![GitHub stars](https://img.shields.io/github/stars/Moon-Spirit/Yueling?style=social)](https://github.com/Moon-Spirit/Yueling/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/Moon-Spirit/Yueling?style=social)](https://github.com/Moon-Spirit/Yueling/network/members)
[![License](https://img.shields.io/github/license/Moon-Spirit/Yueling)](https://github.com/Moon-Spirit/Yueling/blob/main/LICENSE)

一个现代化的即时通讯应用，支持实时消息、好友管理、头像上传等功能。

## 项目地址

[GitHub 仓库](https://github.com/Moon-Spirit/Yueling)

## 技术栈

### 前端
| 技术 | 版本 | 用途 |
|------|------|------|
| Vue 3 | ^3.5.0 | 前端框架 |
| TypeScript | ~5.5.0 | 类型系统 |
| Vite | ^7.0.0 | 构建工具 |
| CSS3 | - | 样式设计 |

### 后端
| 技术 | 版本 | 用途 |
|------|------|------|
| Rust | 2024 | 后端语言 |
| Actix Web | - | Web 框架 |
| SQLite | 0.38.0 | 数据库 |

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

### 🎯 核心功能

| 功能 | 描述 | 状态 |
|------|------|------|
| 用户登录和注册 | 支持新用户注册和现有用户登录 | ✅ 已实现 |
| 实时消息通讯 | 实时发送和接收文本消息 | ✅ 已实现 |
| 好友管理 | 添加好友、处理好友请求 | ✅ 已实现 |
| 头像上传 | 支持自定义头像上传和显示 | ✅ 已实现 |
| 在线状态 | 显示用户在线/离线状态 | ❌ 未实现 |
| 主题切换 | 支持深色/浅色主题自动切换 | ✅ 已实现 |
| 响应式设计 | 适配桌面端和移动端 | ✅ 已实现 |
| 群聊功能 | 创建和管理群聊，支持多人聊天 | ❌ 未实现 |
| 语音消息 | 支持发送和接收语音消息 | ❌ 未实现 |
| 视频通话 | 支持一对一视频通话 | ❌ 未实现 |
| 文件传输 | 支持发送和接收文件 | ❌ 未实现 |
| 消息加密 | 端到端加密保护消息安全 | ❌ 未实现 |
| 多设备同步 | 消息和联系人多设备同步 | ❌ 未实现 |
| 消息撤回 | 支持撤回已发送的消息 | ❌ 未实现 |
| 消息转发 | 支持转发消息给其他联系人 | ❌ 未实现 |

### 🎨 UI 特性

- 现代化卡片式设计
- 流畅的动画效果
- 渐变色彩方案
- 响应式布局
- 深色/浅色主题

## 功能详解

### 📱 用户系统
- 注册新用户
- 用户登录（支持记住密码）
- 个人资料管理
- 头像上传和自定义

### 💬 消息系统
- 实时文本消息
- 消息状态显示（已发送、已送达、已读）
- 消息历史记录
- 消息气泡动画效果

### 👥 好友系统
- 添加好友（通过用户名）
- 好友请求管理（接受/拒绝）
- 好友列表展示
- 好友在线状态实时更新（开发中）

### 👥 群聊系统（开发中）
- 创建和管理群聊
- 邀请好友加入群聊
- 群聊消息管理
- 群成员管理

### 🎤 语音消息（开发中）
- 录制和发送语音消息
- 播放接收的语音消息
- 语音消息时长显示

### 📹 视频通话（开发中）
- 一对一视频通话
- 视频通话质量调整
- 视频通话记录

### 📁 文件传输（开发中）
- 发送和接收文件
- 文件传输进度显示
- 文件历史记录

### 🔒 消息加密（开发中）
- 端到端加密
- 消息安全保障

### 📱 多设备同步（开发中）
- 消息多设备同步
- 联系人多设备同步
- 配置多设备同步

### ⏰ 消息撤回（开发中）
- 撤回已发送的消息
- 撤回消息通知

### 🔄 消息转发（开发中）
- 转发消息给其他联系人
- 转发消息到群聊

### 🎨 界面设计
- 现代化卡片式布局
- 流畅的交互动画
- 渐变色彩方案
- 响应式设计（支持桌面端和移动端）
- 深色/浅色主题自动切换

## ⚠️ 注意事项

| 事项 | 说明 |
|------|------|
| 后端服务器 | 默认运行在 `http://localhost:2025` |
| 前端服务器 | 默认运行在 `http://localhost:3000` |
| 数据库 | 首次运行时自动创建 SQLite 数据库文件 |
| 启动顺序 | 确保先启动后端服务器，再启动前端 |

## 🛠️ 开发指南

### 前端开发规范
- **代码风格**：使用 TypeScript 严格模式
- **组件命名**：采用 PascalCase 命名法
- **变量命名**：采用 camelCase 命名法
- **样式设计**：使用 CSS 变量和现代 CSS 特性
- **文件结构**：按功能模块组织代码

### 后端开发规范
- **代码风格**：遵循 Rust 官方风格指南
- **API 设计**：采用 RESTful API 设计规范
- **数据库**：使用 SQLite 进行本地开发
- **错误处理**：统一的错误处理机制

## 📄 许可证

本项目采用 MIT 许可证

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

### 贡献流程
1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

## 📞 联系方式

如有问题或建议，欢迎通过以下方式联系：
- [GitHub Issues](https://github.com/Moon-Spirit/Yueling/issues)

---

### 🌟 感谢使用月灵聊天应用！

如果这个项目对您有帮助，欢迎给个 Star ⭐️

© 2026 月灵聊天应用 | 用心打造现代化通讯体验