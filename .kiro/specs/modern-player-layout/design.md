# 设计文档

## 概述

本设计文档描述了音乐播放器现代化布局重设计的技术实现方案。设计目标是创建一个视觉层次清晰、信息组织合理、符合现代播放器设计理念的终端用户界面。

设计采用三层垂直布局结构：
1. **顶部区域**：当前播放信息（Now Playing）
2. **中部区域**：曲目列表和可视化效果并排显示
3. **底部区域**：播放控制和进度条

这种布局模式参考了 Spotify、Apple Music 等现代播放器的设计理念，将最常用的播放控制放在底部，便于快速访问。

## 架构

### 布局架构

采用 ratatui 的 Layout 系统实现响应式布局：

```
┌─────────────────────────────────────────────────────────┐
│  Now Playing Area (固定高度: 5-7行)                      │
│  - 曲目标题、艺术家、专辑                                 │
│  - 播放状态图标                                          │
└─────────────────────────────────────────────────────────┘
┌──────────────────────────┬──────────────────────────────┐
│  Track List              │  Visualization               │
│  (50-60% 宽度)           │  (40-50% 宽度)               │
│  - 曲目序号              │  - 波形/频谱图                │
│  - 播放图标              │  - 实时更新                   │
│  - 曲目名称              │  - 颜色渐变                   │
│  (可滚动，占据剩余高度)   │                              │
└──────────────────────────┴──────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│  Playback Control Area (固定高度: 5-6行)                 │
│  - 进度条 (全宽)                                         │
│  - 时间显示 (当前/总时长)                                 │
│  - 音量指示器                                            │
│  - 控制提示                                              │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│  Status Bar (固定高度: 1行)                              │
│  - 快捷键提示                                            │
└─────────────────────────────────────────────────────────┘
```

### 响应式布局策略

- **宽度 >= 80 列**：完整布局，显示所有区域
- **宽度 < 80 列**：隐藏可视化区，曲目列表占据全宽
- **高度 < 20 行**：减少各区域高度，优先保证核心功能

## 组件和接口

### UI 模块结构

```rust
// src/ui/mod.rs
pub mod layout;      // 布局计算和响应式逻辑
pub mod widgets;     // 自定义 widget 组件
pub mod theme;       // 颜色主题和样式

// 主要接口
pub fn render_app(frame: &mut Frame, app: &App, area: Rect);
```

### 布局组件

#### 1. LayoutManager

负责计算和管理整体布局：

```rust
pub struct LayoutManager {
    terminal_size: Rect,
}

impl LayoutManager {
    pub fn new(size: Rect) -> Self;
    pub fn calculate_layout(&self) -> AppLayout;
    pub fn is_compact_mode(&self) -> bool;
}

pub struct AppLayout {
    pub now_playing: Rect,
    pub track_list: Rect,
    pub visualization: Option<Rect>,  // 紧凑模式下为 None
    pub playback_control: Rect,
    pub status_bar: Rect,
}
```

#### 2. NowPlayingWidget

显示当前播放信息：

```rust
pub struct NowPlayingWidget<'a> {
    track: Option<&'a Track>,
    status: PlaybackStatus,
}

impl<'a> Widget for NowPlayingWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer);
}
```

#### 3. TrackListWidget

显示曲目列表：

```rust
pub struct TrackListWidget<'a> {
    tracks: &'a [Track],
    selected: usize,
    playing: Option<usize>,
}

impl<'a> Widget for TrackListWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer);
}
```

#### 4. VisualizationWidget

显示音频可视化：

```rust
pub struct VisualizationWidget<'a> {
    wave_data: &'a [u64],
    is_playing: bool,
}

impl<'a> Widget for VisualizationWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer);
}
```

#### 5. PlaybackControlWidget

显示播放控制和进度：

```rust
pub struct PlaybackControlWidget {
    position: Duration,
    total: Option<Duration>,
    volume: f32,
    status: PlaybackStatus,
}

impl Widget for PlaybackControlWidget {
    fn render(self, area: Rect, buf: &mut Buffer);
}
```

#### 6. StatusBarWidget

显示状态栏和快捷键：

```rust
pub struct StatusBarWidget<'a> {
    hints: &'a [(&'a str, &'a str)],  // (key, description)
}

impl<'a> Widget for StatusBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer);
}
```

### 主题系统

```rust
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub text: Color,
    pub text_dim: Color,
    pub border: Color,
    pub highlight: Color,
    pub progress: Color,
}

impl Theme {
    pub fn default() -> Self;
    pub fn style_title(&self) -> Style;
    pub fn style_text(&self) -> Style;
    pub fn style_highlight(&self) -> Style;
    pub fn style_border(&self) -> Style;
}
```

## 数据模型

现有的数据模型已经足够支持新布局，无需修改：

```rust
pub struct Track {
    pub id: u64,
    pub path: PathBuf,
    pub duration: Option<Duration>,
    pub title: Option<String>,
}

pub enum PlaybackStatus {
    Stopped,
    Paused,
    Playing,
}
```

### App 状态扩展

为支持新布局，App 结构需要添加一些字段：

```rust
struct App {
    // 现有字段...
    tracks: Vec<Track>,
    selected: usize,
    playing: Option<usize>,
    status: PlaybackStatus,
    position: Duration,
    total: Option<Duration>,
    volume: f32,
    wave: Vec<u64>,
    
    // 新增字段
    theme: Theme,              // 主题配置
    compact_mode: bool,        // 是否处于紧凑模式
}
```


## 正确性属性

*属性是一个特征或行为，应该在系统的所有有效执行中保持为真——本质上是关于系统应该做什么的形式化陈述。属性作为人类可读规范和机器可验证正确性保证之间的桥梁。*

### 属性 1: 布局三区域结构

*对于任意*终端尺寸，布局计算应该产生三个主要垂直区域（当前播放信息区、主内容区、播放控制区），并且这些区域按从上到下的顺序排列，没有重叠
**验证: 需求 1.1, 1.3**

### 属性 2: 当前播放信息包含曲目标题

*对于任意*正在播放的曲目，当前播放信息区的渲染输出应该包含该曲目的标题
**验证: 需求 2.1**

### 属性 3: 播放状态图标显示

*对于任意*播放状态（播放/暂停/停止），当前播放信息区的渲染输出应该包含对应的状态图标
**验证: 需求 2.4**

### 属性 4: 进度条占据全宽

*对于任意*终端宽度，播放控制区中进度条的宽度应该等于播放控制区的宽度（减去边框）
**验证: 需求 3.1**

### 属性 5: 时间信息显示

*对于任意*播放位置和总时长，播放控制区的渲染输出应该包含格式化的时间字符串（MM:SS 格式）
**验证: 需求 3.2**

### 属性 6: 音量指示器显示

*对于任意*音量值（0.0-2.0），播放控制区的渲染输出应该包含音量百分比显示
**验证: 需求 3.3**

### 属性 7: 中部区域水平分割

*对于任意*宽度大于等于 80 列的终端，中部主内容区应该被水平分为曲目列表区和可视化区两部分
**验证: 需求 4.1**

### 属性 8: 曲目列表宽度比例

*对于任意*中部区域宽度（非紧凑模式），曲目列表区的宽度应该占据 50-60% 的比例
**验证: 需求 4.2**

### 属性 9: 可视化区宽度比例

*对于任意*中部区域宽度（非紧凑模式），可视化区的宽度应该占据 40-50% 的比例
**验证: 需求 4.3**

### 属性 10: 曲目列表包含所有曲目

*对于任意*曲目列表，渲染输出应该包含所有曲目的序号和名称
**验证: 需求 5.1**

### 属性 11: 播放图标标记

*对于任意*正在播放的曲目索引，该曲目在列表中的渲染输出应该包含播放图标（▶）
**验证: 需求 5.2**

### 属性 12: 可视化渲染成功

*对于任意*波形数据数组，可视化 widget 的渲染应该成功完成而不出错
**验证: 需求 6.1**

### 属性 13: 停止状态可视化静态

*对于任意*停止或暂停状态，可视化数据应该趋向于静态（低值或零值）
**验证: 需求 6.3**

### 属性 14: 状态栏位于底部

*对于任意*布局计算结果，状态栏应该位于最底部且高度为 1 行
**验证: 需求 7.1**

### 属性 15: 小高度终端布局适配

*对于任意*高度小于 20 行的终端，布局计算的所有区域高度总和应该不超过可用高度
**验证: 需求 8.2**

## 错误处理

### 布局计算错误

- **极小终端尺寸**: 当终端尺寸小于最小可用尺寸（如 20x10）时，显示错误提示信息
- **区域计算溢出**: 确保所有区域尺寸计算不会导致整数溢出或负值

### 渲染错误

- **空曲目列表**: 显示友好的提示信息"未找到音频文件"
- **缺失曲目信息**: 当曲目标题为 None 时，使用文件路径作为后备显示
- **无效波形数据**: 当波形数据为空或无效时，显示静态可视化

### 主题和样式错误

- **颜色不支持**: 在不支持颜色的终端中，使用纯文本样式作为后备
- **样式应用失败**: 确保样式应用失败不会导致渲染中断

## 测试策略

### 单元测试

单元测试覆盖以下方面：

1. **布局计算逻辑**
   - 测试不同终端尺寸下的布局计算结果
   - 测试紧凑模式切换逻辑
   - 测试边界条件（最小尺寸、极大尺寸）

2. **Widget 渲染**
   - 测试各个 widget 的基本渲染功能
   - 测试空数据情况下的渲染
   - 测试特殊字符和长文本的处理

3. **主题系统**
   - 测试主题配置的加载和应用
   - 测试样式生成函数

4. **工具函数**
   - 测试时间格式化函数
   - 测试文本截断和对齐函数

### 属性测试

使用 `proptest` 库进行属性测试，每个测试运行至少 100 次迭代：

1. **布局属性测试**
   - 生成随机终端尺寸，验证布局计算的正确性属性
   - 验证区域不重叠、总和正确等不变量

2. **渲染属性测试**
   - 生成随机曲目数据，验证渲染输出包含必要信息
   - 生成随机播放状态，验证状态图标正确显示

3. **响应式布局属性测试**
   - 生成各种终端尺寸，验证响应式布局规则

每个属性测试必须使用注释明确标记对应的设计文档属性：
```rust
// **Feature: modern-player-layout, Property 1: 布局三区域结构**
#[test]
fn prop_layout_three_regions() { ... }
```

### 集成测试

集成测试验证整体渲染流程：

1. **完整界面渲染**: 测试从 App 状态到完整界面渲染的流程
2. **状态变化渲染**: 测试播放状态变化时界面的正确更新
3. **用户交互**: 测试键盘输入对界面的影响

### 测试工具

- **单元测试框架**: Rust 内置 `#[test]`
- **属性测试库**: `proptest` crate
- **断言库**: 标准 `assert!` 宏和 `pretty_assertions` crate
- **测试辅助**: 自定义测试辅助函数用于创建测试数据

## 实现注意事项

### 性能考虑

1. **布局缓存**: 当终端尺寸未改变时，缓存布局计算结果
2. **增量渲染**: 利用 ratatui 的差异渲染机制，只更新变化的部分
3. **波形数据优化**: 限制波形数据数组大小，避免内存过度使用

### 可维护性

1. **模块化设计**: 将布局、widget、主题分离到独立模块
2. **配置化**: 将布局参数（如宽度比例、高度）提取为常量，便于调整
3. **文档注释**: 为所有公共接口添加详细的文档注释

### 可扩展性

1. **主题系统**: 设计可扩展的主题系统，支持未来添加多种主题
2. **Widget 接口**: 使用统一的 Widget trait，便于添加新的 UI 组件
3. **布局策略**: 支持未来添加更多响应式布局策略

## 迁移策略

从现有实现迁移到新布局的步骤：

1. **创建新的 UI 模块结构**: 添加 `layout.rs`、`widgets.rs`、`theme.rs`
2. **实现布局管理器**: 实现 `LayoutManager` 和布局计算逻辑
3. **实现各个 Widget**: 逐个实现新的 widget 组件
4. **更新主渲染函数**: 修改 `main.rs` 中的渲染逻辑，使用新的布局和 widget
5. **保持功能兼容**: 确保所有现有功能在新布局中正常工作
6. **清理旧代码**: 移除旧的渲染代码

## 依赖项

- `ratatui`: ^0.26 - TUI 框架
- `crossterm`: ^0.27 - 终端控制
- `proptest`: ^1.4 - 属性测试（开发依赖）
- `pretty_assertions`: ^1.4 - 测试断言（开发依赖）

现有依赖项无需更改，只需添加测试相关的开发依赖。
