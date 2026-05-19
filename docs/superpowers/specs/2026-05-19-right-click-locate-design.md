# 右键定位窗口功能设计

## 背景

当前定位功能只支持鼠标左键拖拽。用户反馈某些菜单窗口点击左键会消失，无法定位。需要支持右键定位。

## 需求

同时支持鼠标左键和右键进行窗口定位，用户可以自由选择。

## 设计方案

采用智能双键检测方案：同时检测左键和右键状态，用户按住任一按键开始拖拽，松开后完成定位。

### 改动范围

**修改文件**: `src/main.rs`

**改动位置**: `MyApp::update` 中的定位模式检测逻辑（约 265-311 行）

### 实现细节

**改动点清单**（均在 `src/main.rs`）：

1. **第 279 行**：修改现有导入，添加 `VK_RBUTTON`
2. **第 283 行**：添加右键状态检测变量
3. **第 286 行**：按下条件改为 `any_button_down`
4. **第 292 行**：松开条件改为 `!any_button_down`
5. **第 157 行**：更新提示文案

```rust
// 第 279 行：修改导入（添加 VK_RBUTTON）
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};

// 第 283-285 行：双键检测
let left_button_down = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) < 0 };
let right_button_down = unsafe { GetAsyncKeyState(VK_RBUTTON.0 as i32) < 0 };
let any_button_down = left_button_down || right_button_down;

// 第 286-288 行：按下判断
if any_button_down && !self.locate_dragging {
    ...
}

// 第 292-309 行：松开判断
if !any_button_down && self.locate_dragging {
    ...
}
```

### 逻辑流程

1. 进入定位模式 → 显示提示"按住鼠标左键或右键拖动到目标窗口，松开后定位"
2. 任一按键按下 → `locate_dragging = true`，提示"拖动到目标窗口，松开鼠标..."
3. 任一按键松开 → `WindowFromPoint` 获取窗口，完成定位，退出定位模式

### 潜在问题

右键松开后可能触发目标窗口的右键菜单。这是 Windows 正常行为，不影响定位功能（`WindowFromPoint` 已在松开瞬间完成窗口识别）。

## 测试要点

1. 左键定位：验证原有功能正常
2. 右键定位：对右键菜单、弹出窗口进行定位测试
3. 右键松开后菜单弹出：验证定位结果正确显示