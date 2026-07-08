# 子网掩码计算工具 (subnet-calc)

一个用 Rust + [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui) 编写的跨平台桌面子网计算工具，界面支持中文。

## 功能

- **基础计算 (Basic)**：输入 `IP/掩码`（或 `IP 掩码`、纯 IP），计算网络地址、广播地址、子网掩码、反掩码 (wildcard)、CIDR、总地址数 / 可用地址数、首末可用主机、IP 地址类别（A/B/C/D/E）、是否私有地址。支持全角数字、全角点号与多余空格的自动归一化。
- **IP 校验 (Check)**：判断 `IP 是否属于某子网`、`两个子网是否包含 / 重叠`，并对多个子网做**路由聚合 (aggregate)**，输出合并后的超网。
- **进制转换 (Convert)**：IPv4 地址在「点分十进制 / 二进制 / 十六进制 / 整数」四种表示之间互转。
- **VLSM 分配 (VLSM)**：给定基础网段与若干主机数需求，按最大块优先依次分配，输出每个子网的网段、掩码、可用范围；空间不足的需求会被列出失败原因。

## 构建与运行

需要 Rust 工具链（建议 stable 最新版）：

```bash
cargo run --release      # 运行
cargo build --release    # 仅构建，产物在 target/release/subnet-calc
cargo test               # 运行单元测试用例
```

> 提示：eframe 依赖原生窗口后端。在 Linux 上通常需要安装系统库，例如
> `libxkbcommon`、`libwayland-client`、`libx11`、`libfontconfig`（具体包名随发行版而异）。

## 项目结构

```
src/
  main.rs            # 应用入口与窗口配置
  theme.rs           # UI 主题配色
  core/              # 纯计算核心（无 UI 依赖，带单元测试）
    ip.rs            # IPv4 解析、类别、私有/回环判断、进制转换
    mask.rs          # 掩码 <-> CIDR 互转、反掩码
    subnet.rs        # 子网分析主逻辑
    vlsm.rs          # VLSM 分配
    aggregate.rs     # 包含 / 重叠 / 聚合
    normalize.rs     # 全角与空格归一化
    error.rs         # 错误类型
  ui/                # egui 界面（四个标签页）
assets/
  cjk_font.otf       # 内置中文字体，保证中文 UI 正常显示
```

核心计算逻辑与界面解耦，且自带单元测试，便于在不启动 GUI 的情况下验证正确性。

## License

本仓库采用根目录 LICENSE 文件中的许可证（详见 LICENSE）。
