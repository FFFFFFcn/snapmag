# SnapMag

SnapMag 是一款现代化的桌面图片管理应用，专为高效管理和浏览剪贴板图片而设计。它能自动监听剪贴板变化，保存你复制的图片，并提供直观的界面进行浏览、查看和管理。

## ✨ 功能特性

### 核心功能
- **剪贴板自动保存**：自动监听剪贴板变化，保存所有复制的图片
- **图片浏览**：网格布局展示所有保存的图片
- **灯箱查看**：支持全屏灯箱模式查看图片细节
- **上下文菜单**：右键点击图片可进行复制、删除等操作
- **系统托盘集成**：最小化到系统托盘，随时访问应用
- **单实例运行**：确保系统中只有一个应用实例运行

### 交互体验
- **拖拽支持**：支持图片拖拽操作
- **响应式设计**：适配不同屏幕尺寸
- **深色/浅色主题**：自动适应系统主题设置
- **快捷键支持**：提高操作效率

## 🚀 快速开始

### 系统要求
- Windows 10/11 (x64)
- Node.js 18+ (推荐使用 Node.js 20)
- Rust 1.77+ (用于 Tauri 开发)

### 安装方式

#### 从安装包安装
1. 下载最新版本的安装包：
   - MSI 安装包：`SnapMag_0.1.0_x64_en-US.msi`
   - NSIS 安装包：`SnapMag_0.1.0_x64-setup.exe`

2. 双击安装包，按照向导完成安装

3. 安装完成后，从开始菜单或桌面图标启动 SnapMag

#### 从源码构建

```bash
# 克隆仓库
git clone <repository-url>
cd Photokirby

# 安装依赖
npm install

# 构建应用
npm run tauri:build

# 构建完成后，安装包将生成在：
# src-tauri/target/release/bundle/msi/
# src-tauri/target/release/bundle/nsis/
```

## 📖 使用指南

### 基本操作
1. **启动应用**：点击桌面图标或开始菜单中的 SnapMag
2. **自动保存图片**：复制任何图片，应用将自动保存到本地
3. **浏览图片**：在主界面上浏览所有保存的图片缩略图
4. **查看大图**：点击任何图片进入灯箱模式查看细节
5. **隐藏窗口**：点击窗口关闭按钮，应用将最小化到系统托盘
6. **显示窗口**：点击系统托盘图标或从托盘菜单选择"显示窗口"

### 上下文菜单
- **复制图片**：右键点击图片选择"复制"，将图片复制到剪贴板
- **删除图片**：右键点击图片选择"删除"，从应用中移除图片
- **清空所有**：右键点击空白区域选择"清空所有"，删除所有保存的图片

### 系统托盘操作
- **左键点击**：显示/隐藏应用窗口
- **右键点击**：打开托盘菜单
  - **显示窗口**：显示隐藏的应用窗口
  - **退出**：完全退出应用

## 🛠️ 开发指南

### 环境设置

1. **安装 Node.js**：从 [Node.js 官网](https://nodejs.org/) 下载并安装

2. **安装 Rust**：
   ```bash
   # 使用 Rustup 安装 Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **安装 Tauri CLI**：
   ```bash
   npm install -g @tauri-apps/cli
   ```

4. **克隆仓库并安装依赖**：
   ```bash
   git clone <repository-url>
   cd Photokirby
   npm install
   ```

### 开发命令

```bash
# 启动前端开发服务器
npm run dev

# 启动 Tauri 开发环境（推荐）
npm run tauri:dev

# 构建前端应用
npm run build

# 构建 Tauri 桌面应用（生产版）
npm run tauri:build

# 生成应用图标
npm run generate-icons

# 代码检查
npm run lint
```

### 项目结构

```
Photokirby/
├── src/                    # 前端代码
│   ├── components/         # React 组件
│   ├── services/           # API 服务
│   ├── assets/             # 静态资源
│   ├── App.tsx             # 应用主组件
│   └── main.tsx            # 应用入口
├── src-tauri/              # Tauri 后端代码
│   ├── src/                # Rust 源代码
│   ├── icons/              # 应用图标
│   ├── target/             # 构建输出
│   └── Cargo.toml          # Rust 项目配置
├── public/                 # 公共静态资源
├── dist/                   # 前端构建输出
├── package.json            # Node.js 项目配置
└── tsconfig.json           # TypeScript 配置
```

## 🧰 技术栈

### 前端
- **React 19**：UI 框架
- **TypeScript**：类型安全
- **Vite**：构建工具
- **TailwindCSS**：样式框架
- **React DnD**：拖拽功能
- **Yet Another React Lightbox**：图片灯箱
- **Lucide React**：图标库

### 后端
- **Rust**：系统编程语言
- **Tauri**：桌面应用框架
- **Tray Icon**：系统托盘支持
- **Windows API**：系统集成

### 图像处理
- **image** (Rust)：图像处理库

## 📝 注意事项

1. **单实例运行**：应用设计为单实例运行，避免系统资源浪费

2. **剪贴板监听**：应用会持续监听系统剪贴板变化，自动保存复制的图片

3. **图片存储**：所有图片保存在本地文件系统中，不会上传到任何服务器

4. **窗口管理**：
   - 点击关闭按钮会将应用最小化到系统托盘
   - 从系统托盘可以重新显示窗口
   - 从桌面图标再次启动会显示已运行的实例窗口

5. **性能优化**：
   - 图片会自动进行清理（可配置）
   - 应用占用资源较少，适合长时间运行

## 🐛 故障排除

### 问题：应用无法启动
- 确保已安装所有依赖
- 检查系统是否满足最低要求
- 尝试重新安装应用

### 问题：剪贴板图片未自动保存
- 检查应用是否有剪贴板访问权限
- 确保剪贴板中的内容是图片格式
- 重启应用后重试

### 问题：托盘图标无法显示窗口
- 确保应用已正确安装
- 尝试从任务管理器重启应用
- 检查系统托盘设置，确保应用图标可见

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

### 贡献流程
1. Fork 仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

### 开发规范
- 遵循现有的代码风格
- 添加适当的注释
- 确保所有测试通过
- 更新文档以反映更改

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情

## 📞 支持

如果您在使用过程中遇到问题，请通过以下方式联系：
- 提交 Issue：[GitHub Issues](<repository-url>/issues)
- 发送邮件：[support@snapmag.app]

---

**SnapMag** - 让图片管理更简单！ 📸
