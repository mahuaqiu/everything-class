# 右键定位窗口功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 同时支持鼠标左键和右键进行窗口定位，解决菜单窗口点击左键消失无法定位的问题。

**Architecture:** 修改定位模式检测逻辑，添加 VK_RBUTTON 检测，使用 `any_button_down` 替代原有的 `left_button_down`。

**Tech Stack:** Rust, eframe/egui, Windows API (GetAsyncKeyState, VK_RBUTTON)

---

## 文件结构

**修改文件:**
- `src/main.rs` - 定位模式检测逻辑（约 265-311 行）

---

### Task 1: 修改代码实现双键检测

**Files:**
- Modify: `src/main.rs:157,279,283-310`

- [ ] **Step 1: 修改导入语句（第 279 行）**

将现有导入：
```rust
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
```

修改为：
```rust
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};
```

- [ ] **Step 2: 添加右键状态检测（第 283-285 行）**

在现有的 `left_button_down` 检测后添加：
```rust
let left_button_down = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) < 0 };
let right_button_down = unsafe { GetAsyncKeyState(VK_RBUTTON.0 as i32) < 0 };
let any_button_down = left_button_down || right_button_down;
```

- [ ] **Step 3: 修改按下判断条件（第 286 行）**

将：
```rust
if left_button_down && !self.locate_dragging {
```

修改为：
```rust
if any_button_down && !self.locate_dragging {
```

- [ ] **Step 4: 修改松开判断条件（第 292 行）**

将：
```rust
if !left_button_down && self.locate_dragging {
```

修改为：
```rust
if !any_button_down && self.locate_dragging {
```

- [ ] **Step 5: 更新提示文案（第 157 行）**

将：
```rust
self.message = "按住鼠标左键拖动到目标窗口，松开后定位".to_string();
```

修改为：
```rust
self.message = "按住鼠标左键或右键拖动到目标窗口，松开后定位".to_string();
```

- [ ] **Step 6: 编译验证**

Run: `cargo build`
Expected: 编译成功，无错误

---

### Task 2: 手动测试验证

**Files:**
- None (手动测试)

- [ ] **Step 1: 运行程序**

Run: `cargo run`
Expected: 程序启动，显示主窗口

- [ ] **Step 2: 测试左键定位**

操作：
1. 点击"定位"按钮进入定位模式
2. 按住左键拖动到任意窗口
3. 松开左键

Expected: 定位成功，显示目标窗口信息

- [ ] **Step 3: 测试右键定位**

操作：
1. 点击"定位"按钮进入定位模式
2. 按住右键拖动到右键菜单或其他弹出窗口
3. 松开右键

Expected: 定位成功，显示目标窗口信息（右键菜单弹出不影响结果）

- [ ] **Step 4: 测试双键行为**

操作：
1. 同时按下左键和右键
2. 松开全部按键

Expected: 定位成功（全部松开后触发）

---

### Task 3: 提交代码

**Files:**
- `src/main.rs`

- [ ] **Step 1: 提交改动**

```bash
git add src/main.rs
git commit -m "feat: 支持右键定位窗口"
```

Expected: 提交成功

- [ ] **Step 2: 查看提交历史**

Run: `git log --oneline -3`
Expected: 显示最新提交 "feat: 支持右键定位窗口"