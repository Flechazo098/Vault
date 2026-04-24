# Vault — API Key & Token Manager

**Vault** 是一款本地优先、加密存储的 API 密钥和令牌管理器。采用 Rust + Tauri v2 构建，所有数据在写入磁盘前均使用 AES-256-GCM 加密。

> A local-first, encrypted API key and token manager built with Rust + Tauri v2. All data is encrypted with AES-256-GCM before touching disk.

---

## 界面预览 / Preview

| 解锁界面 / Unlock View | 主界面 / Dashboard |
|---|---|
| 输入主密码解锁或创建新保险库 | 三栏布局：分类侧栏、条目列表、详情面板 |

---

## 功能特性 / Features

- **🔒 加密存储** — AES-256-GCM 加密，Argon2id 密钥派生 (64 MiB 内存，3 次迭代)
- **🔑 主密码保护** — 保险库钥匙加密机制，主密码不直接加密数据
- **🕐 自动锁定** — 5 分钟无操作自动锁定，零化内存中的密钥
- **📋 剪贴板管理** — 复制密钥 5 秒后混淆覆盖，15 秒后 `EmptyClipboard` 安全清除（不影响 Win+V 历史）
- **🔄 多语言支持** — 中文 / English 界面切换
- **📦 导入与导出** — 独立密码加密导出（AES-256-GCM + Argon2id），跨设备安全迁移，导入自动生成新 UUID
- **🔍 搜索与分类** — 按名称、描述、类型搜索，按分类筛选
- **🛡️ 内存安全** — 敏感数据在使用后主动零化

---

## 快速开始 / Quick Start

### 前置要求 / Prerequisites

| 工具 / Tool | 版本 / Version | 用途 / Purpose |
|---|---|---|
| [Rust](https://rustup.rs/) | 1.75+ | Rust 编译 |
| [Node.js](https://nodejs.org/) | 18+ | 前端构建 |
| [pnpm](https://pnpm.io/) 或 npm | — | 包管理 |

### 安装依赖 / Install Dependencies

```bash
# 前端依赖
npm install

# Tauri CLI
npm run tauri -v
```

### 开发模式运行 / Run in Development

```bash
npm run tauri dev
```

### 生产构建 / Build for Production

```bash
npm run tauri build
```

构建产物位于：`src-tauri/target/release/`

---

## 使用指南 / Usage Guide

### 第一次使用 / First Time

1. 启动应用，你会看到 **解锁 / New Vault** 界面
2. 点击 **New Vault** 标签，切换到创建模式
3. 输入**主密码**（至少 8 个字符）并确认
4. 点击 **Create Vault** 创建保险库
5. 系统自动解锁，进入主界面

> ⚠️ **警告**：主密码无法找回！请使用密码管理器保存，或确保你能记住它。

### 创建条目 / Create Entry

1. 点击顶部工具栏的 **New** 按钮
2. 填写表单：
   - **Name**（必填）：条目名称，如 "GitHub PAT"
   - **Type**：选择类型（API Key / Token / Secret / Password / Certificate / Other）
   - **Category**：选择分类（General / Development / Production 等），或点 "+ New Category" 自定义
   - **Description**：描述用途（可选）
   - **Secret**：粘贴或输入 API 密钥或令牌
3. 点击 **Create Entry** 保存

### 查看密钥 / View a Secret

1. 在左侧列表中点击一个条目
2. 在详情面板中点击 **Show** 按钮
3. 密钥会从后端解密并显示
4. 点击 **Hide** 或切换到其他条目时，密钥会被清除

### 复制到剪贴板 / Copy to Clipboard

1. 在详情面板中点击 **Copy** 按钮
2. 密钥被复制到剪贴板
3. **15 秒后自动清除**剪贴板内容

> ⚠️ **注意 / Note**：`EmptyClipboard()` 只能清空当前剪贴板内容（Ctrl+V），**无法清空 Windows 剪贴板历史记录（Win+V）**。复制的内容会短暂出现在 Win+V 历史中，5 秒后通过混淆覆盖为空字符串。历史记录中的 pinned 项不受任何影响。
>
> *`EmptyClipboard()` only clears the current clipboard content (Ctrl+V). It **cannot** clear Windows clipboard history (Win+V). The copied secret briefly appears in Win+V history, then is overwritten with empty text after 5 seconds. Pinned items are never affected.*

### 导出条目 / Export Entries

1. 点击顶部工具栏的 **Export** 按钮（向上箭头图标）
2. 在弹出的对话框中：
   - 创建 **导出密码**（至少 8 个字符，与主密码不同）
   - 确认导出密码
3. 选择保存位置，文件格式为 `.vault-encrypted`
4. 所有条目使用**独立的 Argon2id 派生密钥**逐条加密

> 🔒 **安全说明**：导出使用独立的密码和密钥，与保管库主密码无关。每个条目使用随机 nonce 独立加密，导出文件不会泄露保管库密钥结构。

### 导入条目 / Import Entries

1. 删除部分或全部现有条目（模拟迁移）
2. 点击顶部工具栏的 **Import** 按钮（向下箭头图标）
3. 选择之前导出的 `.vault-encrypted` 文件
4. 输入导出时设置的**导出密码**
5. 导入的条目自动获得**全新 UUID**，无跨保险库 ID 关联

> ⚠️ **注意**：导入操作是**追加**而非覆盖。导入时会自动生成新 UUID，避免 ID 冲突。若要完全迁移，请先清空再导入。

### 编辑条目 / Edit Entry

1. 在详情面板中点击编辑（铅笔）图标
2. 修改需要更改的字段
3. **Secret 字段留空则保持当前值不变**
4. 点击 **Save Changes** 保存

### 删除条目 / Delete Entry

1. 在详情面板中点击删除（垃圾桶）图标
2. 在弹出的确认对话框中点击 **Delete**
3. ⚠️ 此操作不可撤销

### 搜索与筛选 / Search & Filter

- **搜索**：在顶部搜索框输入关键字，按名称、描述、类型匹配
- **分类筛选**：在左侧侧栏点击分类名称，只显示该分类下的条目
- **全部条目**：点击 **All Entries** 清除筛选

### 切换语言 / Switch Language

在顶部工具栏右侧点击 **中文 / English** 按钮，即可在中文和英文界面间切换。语言偏好会自动保存。

### 锁定保险库 / Lock Vault

点击顶部工具栏的 **Lock** 按钮立即锁定。或等待 5 分钟无操作后自动锁定。

---

## 安全模型 / Security Model

```
主密码 (Master Password)
    │
    ▼ Argon2id (64 MiB · 3 iter · 4 parallelism)
    │
    ▼ 派生钥匙 (Derived Key)
    │
    ▼ AES-256-GCM 解密
    │
    ▼ 保险库钥匙 (Vault Key) ──── 用于加密/解密所有条目
```

| 层级 / Layer | 算法 / Algorithm | 说明 / Note |
|---|---|---|
| 密钥派生 | Argon2id | 抗 GPU/ASIC 暴力破解 |
| 条目加密 | AES-256-GCM | 每次加密生成随机 96-bit nonce |
| 钥匙包装 | AES-256-GCM | 保险库钥匙由派生钥匙加密存储 |
| 内存擦除 | `zeroize` crate | Drop 时自动零化 |
| 剪贴板限制 | `EmptyClipboard` + 混淆覆盖 | ❌ 不清除 Win+V 历史记录（WinRT ClearHistory 需 STA 线程，桌面应用不保证可用） |

---

## 技术栈 / Tech Stack

| 层级 / Layer | 技术 / Technology |
|---|---|
| 桌面框架 | Tauri v2 (Rust) |
| 前端框架 | Vue 3 (Composition API, TypeScript) |
| 构建工具 | Vite 6 |
| 加密 | `aes-gcm` 0.10, `argon2` 0.5 |
| 序列化 | Serde + Serde JSON |
| 国际化 | vue-i18n 9 |

---

## 项目结构 / Project Structure

```
f:\code\app\Vault/
├── src/                        # Vue Frontend
│   ├── App.vue                 # Root with auto-lock polling
│   ├── main.ts                 # Entry point + i18n setup
│   ├── i18n.ts                 # vue-i18n configuration
│   ├── locales/                # Translation files
│   │   ├── en.json             # English
│   │   └── zh-CN.json          # 中文
│   ├── composables/
│   │   └── useLocale.ts        # Locale switching composable
│   ├── types/index.ts          # TypeScript definitions
│   ├── stores/vault.ts         # Tauri invoke wrappers (singleton)
│   ├── assets/main.css         # Global styles
│   ├── views/
│   │   ├── UnlockView.vue      # Login/create vault
│   │   └── DashboardView.vue   # Three-panel layout
│   └── components/
│       ├── TopBar.vue          # Search, lock, new, import/export, locale
│       ├── AppSidebar.vue      # Category navigation
│       ├── EntryList.vue       # Entry list panel
│       ├── EntryDetail.vue     # View/show/copy secret
│       ├── EntryForm.vue       # Create/edit form
│       └── ConfirmDialog.vue   # Confirmation modal
└── src-tauri/src/              # Rust Backend
    ├── lib.rs                  # App setup, command registration
    ├── main.rs                 # Entry point
    ├── crypto.rs               # AES-256-GCM + Argon2id
    ├── models.rs               # Data structures
    ├── storage.rs              # File I/O
    ├── vault.rs                # State machine, key wrapping
    ├── commands.rs             # 17 Tauri IPC commands
    └── errors.rs               # Error types
```

---

## 命令列表 / Tauri Commands

| 命令 | 功能 |
|---|---|
| `get_vault_status` | 检查保险库状态 |
| `create_vault` | 创建新保险库 |
| `unlock_vault` | 解锁保险库 |
| `lock_vault` | 锁定保险库 |
| `check_auto_lock` | 检查自动锁定 |
| `get_auto_lock_remaining` | 获取锁定剩余时间 |
| `get_entries` | 获取所有条目摘要 |
| `get_entry` | 获取单个条目 |
| `view_secret` | 查看密钥（临时解密） |
| `copy_secret` | 复制到剪贴板（15秒清除） |
| `create_entry` | 创建条目 |
| `update_entry` | 更新条目 |
| `delete_entry` | 删除条目 |
| `get_categories` | 获取所有分类 |
| `export_vault` | 导出全部条目（独立密码加密） |
| `import_from_file` | 从文件导入并解密条目 |
| `write_export_file` | 将导出内容写入磁盘文件 |

---

## 开发 / Development

```bash
# 代码检查
cd src-tauri && cargo check

# TypeScript 类型检查
npx vue-tsc --noEmit

# 前端构建
npx vite build

# 完整构建
npm run tauri build
```

---

## 许可 / License

MIT
