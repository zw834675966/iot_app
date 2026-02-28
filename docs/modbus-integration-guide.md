# Rust Modbus 集成部署指南

> 适用版本：pure-admin-thin（能源管理系统）· Tauri v2 · Rust Edition 2024  
> Modbus 库：`tokio-modbus` v0.17.0（MIT/Apache-2.0）  
> 更新日期：2026-02-28

---

## 目录

1. [概述与适用场景](#1-概述与适用场景)
2. [架构设计](#2-架构设计)
3. [依赖配置](#3-依赖配置)
4. [模块结构](#4-模块结构)
5. [TCP 客户端实现](#5-tcp-客户端实现)
6. [RTU 串口客户端实现](#6-rtu-串口客户端实现)
7. [Tauri IPC 命令层集成](#7-tauri-ipc-命令层集成)
8. [错误处理扩展](#8-错误处理扩展)
9. [前端调用层集成](#9-前端调用层集成)
10. [Tauri 权限配置](#10-tauri-权限配置)
11. [Windows 平台串口注意事项](#11-windows-平台串口注意事项)
12. [测试策略](#12-测试策略)
13. [生产部署检查清单](#13-生产部署检查清单)
14. [常见问题排查](#14-常见问题排查)
15. [业务功能设计：网关与数据采集配置](#15-业务功能设计网关与数据采集配置)

---

## 1. 概述与适用场景

本项目是一个**离线优先**的工业/物联网桌面管理系统。Modbus 协议是工业自动化领域最广泛使用的现场总线协议，用于与 PLC、仪表、传感器等设备通信。

### Modbus 协议类型

| 类型             | 传输媒介    | 端口/接口                                  | 典型应用场景                            |
| ---------------- | ----------- | ------------------------------------------ | --------------------------------------- |
| **TCP**          | 以太网      | 默认端口 502                               | 远程 PLC、以太网仪表、工业网关          |
| **RTU**          | RS-485 串口 | COM 口（Windows）/ `/dev/ttyUSB*`（Linux） | 近端传感器、本地仪表盘、RS-485 总线设备 |
| **RTU over TCP** | 以太网      | 自定义端口                                 | 串口服务器、以太网转串口设备            |

### Modbus 数据模型

| 数据表                         | 功能码         | 读/写 | 数据类型      | 地址范围      |
| ------------------------------ | -------------- | ----- | ------------- | ------------- |
| 线圈（Coil）                   | FC01/FC05/FC15 | 读写  | 布尔（1 bit） | 0x0000–0xFFFF |
| 离散输入（Discrete Input）     | FC02           | 只读  | 布尔（1 bit） | 0x0000–0xFFFF |
| 保持寄存器（Holding Register） | FC03/FC06/FC16 | 读写  | UINT16        | 0x0000–0xFFFF |
| 输入寄存器（Input Register）   | FC04           | 只读  | UINT16        | 0x0000–0xFFFF |

---

## 2. 架构设计

Modbus 功能以独立 `modbus` 领域模块接入现有 DDD 分层架构，与 `auth`、`notice` 模块平级：

```
前端 Vue 3 (TypeScript)
     │  invoke("modbus_tcp_read_registers", {...})
     │  invoke("modbus_rtu_connect", {...})
     ▼
┌──────────────────────────────────────────────────┐
│  src-tauri/src/modbus/commands.rs                │
│  (Adapter Layer — Tauri IPC 接口)                │
│  - 参数校验 / 反序列化 / 结果封装                 │
└──────────────────────────────────────────────────┘
     │
     ▼
┌──────────────────────────────────────────────────┐
│  src-tauri/src/modbus/services.rs                │
│  (Domain Layer — 纯业务逻辑)                     │
│  - 连接管理 / 寄存器读写 / 数据转换               │
└──────────────────────────────────────────────────┘
     │
     ▼
┌──────────────────────────────────────────────────┐
│  tokio-modbus                                    │
│  (Infrastructure — Modbus 协议实现)               │
│  - TCP Client / RTU Client / Async               │
└──────────────────────────────────────────────────┘
     │
     ▼
工业设备（PLC / 传感器 / 仪表）
```

### 状态管理策略

Modbus 连接为**有状态资源**，需通过 Tauri 的 `State<T>` 机制在命令间共享：

```
tauri::Builder::default()
    .manage(ModbusState::new())   ← 注册全局状态
    .invoke_handler(...)
```

---

## 3. 依赖配置

### 3.1 修改 `src-tauri/Cargo.toml`

在 `[dependencies]` 节添加以下依赖：

```toml
# Modbus 协议支持（TCP + RTU）
tokio-modbus = { version = "0.17", default-features = false, features = ["tcp", "rtu"] }

# RTU 串口支持（仅在需要串口通信时添加）
tokio-serial = "5.4"

# Tokio 异步运行时（Modbus 异步 API 所需）
# 注意：Tauri v2 已内置 tokio，但需确认 full 特性
tokio = { version = "1", features = ["full"] }
```

> **特性选择说明**：
>
> - `features = ["tcp"]` — 仅 TCP 客户端（纯以太网场景，二进制最小化）
> - `features = ["rtu"]` — 仅 RTU 串口客户端
> - `features = ["tcp", "rtu"]` — 同时支持两种（推荐，能源管理场景通常需要两者）
> - `features = ["tcp-sync", "rtu-sync"]` — 同步阻塞 API（不推荐在 Tauri 中使用）

### 3.2 验证依赖可解析

```bash
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 > /dev/null
# 或直接 dry-run 构建
cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | head -20
```

---

## 4. 模块结构

在 `src-tauri/src/` 下新建 `modbus/` 目录：

```
src-tauri/src/modbus/
├── mod.rs          # 模块声明与公开类型
├── commands.rs     # Tauri IPC 接口层（#[tauri::command]）
├── services.rs     # 业务逻辑层（连接/读写操作）
├── models.rs       # 数据传输对象（DTO）
├── state.rs        # 全局连接状态管理
└── README.md       # 模块文档
```

---

## 5. TCP 客户端实现

### 5.1 `modbus/models.rs` — 数据模型

```rust
//! Modbus 数据传输对象（DTO）

use serde::{Deserialize, Serialize};

/// TCP 连接参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcpConnectParams {
    /// 目标设备 IP 地址，如 "192.168.1.100"
    pub host: String,
    /// Modbus TCP 端口，标准为 502
    pub port: u16,
    /// Slave/Unit ID（1-247），默认为 1
    #[serde(default = "default_slave_id")]
    pub slave_id: u8,
}

fn default_slave_id() -> u8 {
    1
}

/// RTU 串口连接参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RtuConnectParams {
    /// 串口路径，Windows 示例："COM3"，Linux 示例："/dev/ttyUSB0"
    pub port_name: String,
    /// 波特率，常见值：9600 / 19200 / 38400 / 115200
    pub baud_rate: u32,
    /// Slave 地址（1-247）
    pub slave_id: u8,
}

/// 读取寄存器参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadRegistersParams {
    /// 起始寄存器地址（0-based）
    pub start_address: u16,
    /// 读取数量（1-125 个寄存器）
    pub count: u16,
}

/// 写入单个寄存器参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteSingleRegisterParams {
    /// 寄存器地址
    pub address: u16,
    /// 写入的 16 位无符号整数值
    pub value: u16,
}

/// 写入多个寄存器参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteMultipleRegistersParams {
    /// 起始寄存器地址
    pub start_address: u16,
    /// 写入的值列表
    pub values: Vec<u16>,
}

/// 读取线圈参数
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadCoilsParams {
    /// 起始线圈地址（0-based）
    pub start_address: u16,
    /// 读取数量（1-2000 个线圈）
    pub count: u16,
}

/// 寄存器读取响应
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistersData {
    /// 寄存器值列表（u16）
    pub values: Vec<u16>,
    /// 起始寄存器地址
    pub start_address: u16,
    /// 读取时的 Unix 毫秒时间戳
    pub timestamp_ms: u64,
}

/// 线圈读取响应
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoilsData {
    /// 线圈值列表（bool）
    pub values: Vec<bool>,
    pub start_address: u16,
    pub timestamp_ms: u64,
}

/// 连接状态响应
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    pub connected: bool,
    pub connection_type: String,   // "tcp" | "rtu" | "none"
    pub target: String,            // "192.168.1.100:502" 或 "COM3"
}
```

### 5.2 `modbus/state.rs` — 连接状态管理

```rust
//! Modbus 全局连接状态
//!
//! 使用 tokio::sync::Mutex 保证异步环境下的线程安全。

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_modbus::client::Context;

/// Modbus 客户端的活跃连接类型
pub enum ModbusConnection {
    /// TCP 连接，持有连接目标描述和 Context
    Tcp {
        target: String,
        ctx: Context,
    },
    /// RTU 串口连接
    Rtu {
        port_name: String,
        ctx: Context,
    },
}

/// 全局 Modbus 连接状态，包裹在 Arc<Mutex<>> 中以支持多命令并发访问
pub struct ModbusState {
    pub connection: Arc<Mutex<Option<ModbusConnection>>>,
}

impl ModbusState {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for ModbusState {
    fn default() -> Self {
        Self::new()
    }
}
```

### 5.3 `modbus/services.rs` — 业务逻辑层

```rust
//! Modbus 业务逻辑层
//!
//! 本层不包含任何 Tauri 特定宏，保持可独立测试。

use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio_modbus::client::tcp;
use tokio_modbus::prelude::*;

#[cfg(feature = "rtu")]
use tokio_modbus::client::rtu;

use crate::core::error::AppError;
use crate::modbus::models::{
    CoilsData, ConnectionStatus, ReadCoilsParams, ReadRegistersParams,
    RegistersData, RtuConnectParams, TcpConnectParams, WriteMultipleRegistersParams,
    WriteSingleRegisterParams,
};
use crate::modbus::state::{ModbusConnection, ModbusState};

/// 获取当前 Unix 毫秒时间戳
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ============================================================================
// TCP 连接
// ============================================================================

/// 建立 Modbus TCP 连接
///
/// # 错误
/// - `AppError::Modbus` — 连接失败（网络不通、端口被拒等）
pub async fn tcp_connect(
    state: &ModbusState,
    params: TcpConnectParams,
) -> Result<ConnectionStatus, AppError> {
    let addr: SocketAddr = format!("{}:{}", params.host, params.port)
        .parse()
        .map_err(|e| AppError::Modbus(format!("invalid address: {e}")))?;

    let slave = Slave(params.slave_id);
    let ctx = tcp::connect_slave(addr, slave)
        .await
        .map_err(|e| AppError::Modbus(format!("TCP connect failed: {e}")))?;

    let target = format!("{}:{}", params.host, params.port);
    let status = ConnectionStatus {
        connected: true,
        connection_type: "tcp".to_string(),
        target: target.clone(),
    };

    let mut guard = state.connection.lock().await;
    *guard = Some(ModbusConnection::Tcp { target, ctx });

    Ok(status)
}

// ============================================================================
// RTU 串口连接
// ============================================================================

/// 建立 Modbus RTU 串口连接
///
/// # 错误
/// - `AppError::Modbus` — 串口不存在、权限不足、参数错误
#[cfg(feature = "rtu")]
pub async fn rtu_connect(
    state: &ModbusState,
    params: RtuConnectParams,
) -> Result<ConnectionStatus, AppError> {
    use tokio_serial::SerialStream;

    let builder = tokio_serial::new(&params.port_name, params.baud_rate);
    let port = SerialStream::open(&builder)
        .map_err(|e| AppError::Modbus(format!("open serial port '{}' failed: {e}", params.port_name)))?;

    let slave = Slave(params.slave_id);
    let ctx = rtu::attach_slave(port, slave);

    let port_name = params.port_name.clone();
    let status = ConnectionStatus {
        connected: true,
        connection_type: "rtu".to_string(),
        target: params.port_name.clone(),
    };

    let mut guard = state.connection.lock().await;
    *guard = Some(ModbusConnection::Rtu { port_name, ctx });

    Ok(status)
}

// ============================================================================
// 断开连接
// ============================================================================

/// 断开当前 Modbus 连接
pub async fn disconnect(state: &ModbusState) -> Result<(), AppError> {
    let mut guard = state.connection.lock().await;
    if let Some(conn) = guard.take() {
        match conn {
            ModbusConnection::Tcp { mut ctx, .. }
            | ModbusConnection::Rtu { mut ctx, .. } => {
                ctx.disconnect()
                    .await
                    .map_err(|e| AppError::Modbus(format!("disconnect failed: {e}")))?;
            }
        }
    }
    Ok(())
}

// ============================================================================
// 读操作
// ============================================================================

/// 读取保持寄存器（FC03）
pub async fn read_holding_registers(
    state: &ModbusState,
    params: ReadRegistersParams,
) -> Result<RegistersData, AppError> {
    let mut guard = state.connection.lock().await;
    let ctx = get_context_mut(&mut guard)?;

    let values = ctx
        .read_holding_registers(params.start_address, params.count)
        .await
        .map_err(|e| AppError::Modbus(format!("read holding registers failed: {e}")))?
        .map_err(|e| AppError::Modbus(format!("Modbus exception: {e:?}")))?;

    Ok(RegistersData {
        values,
        start_address: params.start_address,
        timestamp_ms: now_ms(),
    })
}

/// 读取输入寄存器（FC04）
pub async fn read_input_registers(
    state: &ModbusState,
    params: ReadRegistersParams,
) -> Result<RegistersData, AppError> {
    let mut guard = state.connection.lock().await;
    let ctx = get_context_mut(&mut guard)?;

    let values = ctx
        .read_input_registers(params.start_address, params.count)
        .await
        .map_err(|e| AppError::Modbus(format!("read input registers failed: {e}")))?
        .map_err(|e| AppError::Modbus(format!("Modbus exception: {e:?}")))?;

    Ok(RegistersData {
        values,
        start_address: params.start_address,
        timestamp_ms: now_ms(),
    })
}

/// 读取线圈状态（FC01）
pub async fn read_coils(
    state: &ModbusState,
    params: ReadCoilsParams,
) -> Result<CoilsData, AppError> {
    let mut guard = state.connection.lock().await;
    let ctx = get_context_mut(&mut guard)?;

    let values = ctx
        .read_coils(params.start_address, params.count)
        .await
        .map_err(|e| AppError::Modbus(format!("read coils failed: {e}")))?
        .map_err(|e| AppError::Modbus(format!("Modbus exception: {e:?}")))?;

    Ok(CoilsData {
        values,
        start_address: params.start_address,
        timestamp_ms: now_ms(),
    })
}

/// 读取离散输入（FC02）
pub async fn read_discrete_inputs(
    state: &ModbusState,
    params: ReadCoilsParams,
) -> Result<CoilsData, AppError> {
    let mut guard = state.connection.lock().await;
    let ctx = get_context_mut(&mut guard)?;

    let values = ctx
        .read_discrete_inputs(params.start_address, params.count)
        .await
        .map_err(|e| AppError::Modbus(format!("read discrete inputs failed: {e}")))?
        .map_err(|e| AppError::Modbus(format!("Modbus exception: {e:?}")))?;

    Ok(CoilsData {
        values,
        start_address: params.start_address,
        timestamp_ms: now_ms(),
    })
}

// ============================================================================
// 写操作
// ============================================================================

/// 写入单个保持寄存器（FC06）
pub async fn write_single_register(
    state: &ModbusState,
    params: WriteSingleRegisterParams,
) -> Result<(), AppError> {
    let mut guard = state.connection.lock().await;
    let ctx = get_context_mut(&mut guard)?;

    ctx.write_single_register(params.address, params.value)
        .await
        .map_err(|e| AppError::Modbus(format!("write single register failed: {e}")))?
        .map_err(|e| AppError::Modbus(format!("Modbus exception: {e:?}")))?;

    Ok(())
}

/// 写入多个保持寄存器（FC16）
pub async fn write_multiple_registers(
    state: &ModbusState,
    params: WriteMultipleRegistersParams,
) -> Result<(), AppError> {
    let mut guard = state.connection.lock().await;
    let ctx = get_context_mut(&mut guard)?;

    ctx.write_multiple_registers(params.start_address, &params.values)
        .await
        .map_err(|e| AppError::Modbus(format!("write multiple registers failed: {e}")))?
        .map_err(|e| AppError::Modbus(format!("Modbus exception: {e:?}")))?;

    Ok(())
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 从连接状态中获取可变 Context 引用
fn get_context_mut<'a>(
    guard: &'a mut Option<ModbusConnection>,
) -> Result<&'a mut Context, AppError> {
    match guard.as_mut() {
        Some(ModbusConnection::Tcp { ctx, .. }) => Ok(ctx),
        Some(ModbusConnection::Rtu { ctx, .. }) => Ok(ctx),
        None => Err(AppError::Modbus("not connected".to_string())),
    }
}

/// 获取当前连接状态（不锁定连接）
pub async fn get_connection_status(state: &ModbusState) -> ConnectionStatus {
    let guard = state.connection.lock().await;
    match guard.as_ref() {
        Some(ModbusConnection::Tcp { target, .. }) => ConnectionStatus {
            connected: true,
            connection_type: "tcp".to_string(),
            target: target.clone(),
        },
        Some(ModbusConnection::Rtu { port_name, .. }) => ConnectionStatus {
            connected: true,
            connection_type: "rtu".to_string(),
            target: port_name.clone(),
        },
        None => ConnectionStatus {
            connected: false,
            connection_type: "none".to_string(),
            target: String::new(),
        },
    }
}
```

---

## 6. RTU 串口客户端实现

RTU 实现已包含在 `services.rs` 中（见 `rtu_connect` 函数，需启用 `rtu` feature）。

### Windows 串口路径格式

```
COM1, COM2, COM3, ...COM255
```

> Windows 对 COM10 及以上需特殊前缀处理，`tokio-serial` 已自动处理。

### Linux 串口路径格式

```
/dev/ttyUSB0  (USB 转串口适配器)
/dev/ttyS0    (主板原生串口)
/dev/ttyACM0  (USB CDC 设备)
```

### 常用波特率

| 波特率 | 适用场景               |
| ------ | ---------------------- |
| 9600   | 通用默认值，兼容性最好 |
| 19200  | 较高速率，Modbus 常见  |
| 38400  | 中高速                 |
| 115200 | 高速场景               |

---

## 7. Tauri IPC 命令层集成

### 7.1 `modbus/commands.rs`

```rust
//! Modbus Tauri IPC 命令层
//!
//! 所有命令使用 `async`，避免阻塞 Tauri 主线程。

use tauri::State;

use crate::core::error::{ApiResponse, AppError, AppResult};
use crate::modbus::models::{
    CoilsData, ConnectionStatus, ReadCoilsParams, ReadRegistersParams,
    RegistersData, RtuConnectParams, TcpConnectParams, WriteMultipleRegistersParams,
    WriteSingleRegisterParams,
};
use crate::modbus::services;
use crate::modbus::state::ModbusState;

// ============================================================================
// 连接管理命令
// ============================================================================

/// 建立 Modbus TCP 连接
///
/// 前端调用：`invoke("modbus_tcp_connect", { params: { host, port, slaveId } })`
#[tauri::command]
pub async fn modbus_tcp_connect(
    state: State<'_, ModbusState>,
    params: TcpConnectParams,
) -> AppResult<ConnectionStatus> {
    if params.host.trim().is_empty() {
        return Err(AppError::Validation("host is required".to_string()));
    }
    if params.port == 0 {
        return Err(AppError::Validation("port must be non-zero".to_string()));
    }
    let status = services::tcp_connect(&state, params).await?;
    Ok(ApiResponse::ok(status))
}

/// 建立 Modbus RTU 串口连接
///
/// 前端调用：`invoke("modbus_rtu_connect", { params: { portName, baudRate, slaveId } })`
#[tauri::command]
pub async fn modbus_rtu_connect(
    state: State<'_, ModbusState>,
    params: RtuConnectParams,
) -> AppResult<ConnectionStatus> {
    if params.port_name.trim().is_empty() {
        return Err(AppError::Validation("portName is required".to_string()));
    }
    if params.baud_rate == 0 {
        return Err(AppError::Validation("baudRate must be non-zero".to_string()));
    }
    #[cfg(feature = "rtu")]
    {
        let status = services::rtu_connect(&state, params).await?;
        return Ok(ApiResponse::ok(status));
    }
    #[cfg(not(feature = "rtu"))]
    {
        Err(AppError::Modbus("RTU feature not enabled".to_string()))
    }
}

/// 断开当前 Modbus 连接
///
/// 前端调用：`invoke("modbus_disconnect")`
#[tauri::command]
pub async fn modbus_disconnect(state: State<'_, ModbusState>) -> AppResult<()> {
    services::disconnect(&state).await?;
    Ok(ApiResponse::ok(()))
}

/// 获取当前连接状态
///
/// 前端调用：`invoke("modbus_connection_status")`
#[tauri::command]
pub async fn modbus_connection_status(
    state: State<'_, ModbusState>,
) -> AppResult<ConnectionStatus> {
    Ok(ApiResponse::ok(services::get_connection_status(&state).await))
}

// ============================================================================
// 读操作命令
// ============================================================================

/// 读取保持寄存器（FC03）
///
/// 前端调用：`invoke("modbus_read_holding_registers", { params: { startAddress, count } })`
#[tauri::command]
pub async fn modbus_read_holding_registers(
    state: State<'_, ModbusState>,
    params: ReadRegistersParams,
) -> AppResult<RegistersData> {
    if params.count == 0 || params.count > 125 {
        return Err(AppError::Validation(
            "count must be between 1 and 125".to_string(),
        ));
    }
    let data = services::read_holding_registers(&state, params).await?;
    Ok(ApiResponse::ok(data))
}

/// 读取输入寄存器（FC04）
///
/// 前端调用：`invoke("modbus_read_input_registers", { params: { startAddress, count } })`
#[tauri::command]
pub async fn modbus_read_input_registers(
    state: State<'_, ModbusState>,
    params: ReadRegistersParams,
) -> AppResult<RegistersData> {
    if params.count == 0 || params.count > 125 {
        return Err(AppError::Validation(
            "count must be between 1 and 125".to_string(),
        ));
    }
    let data = services::read_input_registers(&state, params).await?;
    Ok(ApiResponse::ok(data))
}

/// 读取线圈状态（FC01）
///
/// 前端调用：`invoke("modbus_read_coils", { params: { startAddress, count } })`
#[tauri::command]
pub async fn modbus_read_coils(
    state: State<'_, ModbusState>,
    params: ReadCoilsParams,
) -> AppResult<CoilsData> {
    if params.count == 0 || params.count > 2000 {
        return Err(AppError::Validation(
            "count must be between 1 and 2000".to_string(),
        ));
    }
    let data = services::read_coils(&state, params).await?;
    Ok(ApiResponse::ok(data))
}

/// 读取离散输入（FC02）
///
/// 前端调用：`invoke("modbus_read_discrete_inputs", { params: { startAddress, count } })`
#[tauri::command]
pub async fn modbus_read_discrete_inputs(
    state: State<'_, ModbusState>,
    params: ReadCoilsParams,
) -> AppResult<CoilsData> {
    if params.count == 0 || params.count > 2000 {
        return Err(AppError::Validation(
            "count must be between 1 and 2000".to_string(),
        ));
    }
    let data = services::read_discrete_inputs(&state, params).await?;
    Ok(ApiResponse::ok(data))
}

// ============================================================================
// 写操作命令
// ============================================================================

/// 写入单个保持寄存器（FC06）
///
/// 前端调用：`invoke("modbus_write_single_register", { params: { address, value } })`
#[tauri::command]
pub async fn modbus_write_single_register(
    state: State<'_, ModbusState>,
    params: WriteSingleRegisterParams,
) -> AppResult<()> {
    services::write_single_register(&state, params).await?;
    Ok(ApiResponse::ok(()))
}

/// 写入多个保持寄存器（FC16）
///
/// 前端调用：`invoke("modbus_write_multiple_registers", { params: { startAddress, values } })`
#[tauri::command]
pub async fn modbus_write_multiple_registers(
    state: State<'_, ModbusState>,
    params: WriteMultipleRegistersParams,
) -> AppResult<()> {
    if params.values.is_empty() {
        return Err(AppError::Validation("values cannot be empty".to_string()));
    }
    if params.values.len() > 123 {
        return Err(AppError::Validation(
            "values length cannot exceed 123".to_string(),
        ));
    }
    services::write_multiple_registers(&state, params).await?;
    Ok(ApiResponse::ok(()))
}
```

### 7.2 `modbus/mod.rs`

```rust
//! Modbus 领域模块

pub mod commands;
pub mod models;
pub mod services;
pub mod state;
```

### 7.3 注册模块到 `lib.rs`

在 `src-tauri/src/lib.rs` 中：

```rust
pub mod auth;
pub mod core;
pub mod db;
pub mod modbus;   // ← 新增
pub mod notice;

use tauri::Manager;
use crate::modbus::state::ModbusState;  // ← 新增

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ModbusState::new())   // ← 新增：注册 Modbus 全局状态
        .setup(|app| {
            // ... 现有的 db 初始化代码不变 ...
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 现有命令不变
            auth::commands::auth_login,
            auth::commands::auth_refresh_token,
            auth::commands::auth_get_async_routes,
            notice::commands::notice_get_unread_items,
            notice::commands::notice_get_read_items,
            notice::commands::notice_mark_read,
            // 新增 Modbus 命令
            modbus::commands::modbus_tcp_connect,
            modbus::commands::modbus_rtu_connect,
            modbus::commands::modbus_disconnect,
            modbus::commands::modbus_connection_status,
            modbus::commands::modbus_read_holding_registers,
            modbus::commands::modbus_read_input_registers,
            modbus::commands::modbus_read_coils,
            modbus::commands::modbus_read_discrete_inputs,
            modbus::commands::modbus_write_single_register,
            modbus::commands::modbus_write_multiple_registers,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri runtime startup failed");
}
```

---

## 8. 错误处理扩展

### 8.1 扩展 `core/error.rs`

在 `AppError` 枚举中添加 `Modbus` 变体：

```rust
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),

    #[error("database error: {0}")]
    Database(String),

    // 新增 Modbus 错误变体
    #[error("modbus error: {0}")]
    Modbus(String),
}
```

---

## 9. 前端调用层集成

### 9.1 TypeScript 类型定义

在 `src/types/` 下新建 `modbus.d.ts`：

```typescript
// src/types/modbus.d.ts

export interface TcpConnectParams {
  host: string;
  port: number;
  slaveId?: number; // 默认 1
}

export interface RtuConnectParams {
  portName: string; // Windows: "COM3", Linux: "/dev/ttyUSB0"
  baudRate: number; // 9600 / 19200 / 115200 等
  slaveId: number;
}

export interface ReadRegistersParams {
  startAddress: number; // 0-65535
  count: number; // 1-125
}

export interface ReadCoilsParams {
  startAddress: number;
  count: number; // 1-2000
}

export interface WriteSingleRegisterParams {
  address: number;
  value: number; // 0-65535
}

export interface WriteMultipleRegistersParams {
  startAddress: number;
  values: number[]; // 最多 123 个
}

export interface RegistersData {
  values: number[];
  startAddress: number;
  timestampMs: number;
}

export interface CoilsData {
  values: boolean[];
  startAddress: number;
  timestampMs: number;
}

export interface ConnectionStatus {
  connected: boolean;
  connectionType: "tcp" | "rtu" | "none";
  target: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T;
}
```

### 9.2 Modbus API 封装

在 `src/api/` 下新建 `modbus.ts`：

```typescript
// src/api/modbus.ts
import { invoke } from "@tauri-apps/api/core";
import type {
  ApiResponse,
  CoilsData,
  ConnectionStatus,
  ReadCoilsParams,
  ReadRegistersParams,
  RegistersData,
  RtuConnectParams,
  TcpConnectParams,
  WriteMultipleRegistersParams,
  WriteSingleRegisterParams
} from "@/types/modbus";

// ─── 连接管理 ─────────────────────────────────────────────

/** 建立 Modbus TCP 连接 */
export async function modbusTcpConnect(
  params: TcpConnectParams
): Promise<ConnectionStatus> {
  const res = await invoke<ApiResponse<ConnectionStatus>>(
    "modbus_tcp_connect",
    { params }
  );
  return res.data;
}

/** 建立 Modbus RTU 串口连接 */
export async function modbusRtuConnect(
  params: RtuConnectParams
): Promise<ConnectionStatus> {
  const res = await invoke<ApiResponse<ConnectionStatus>>(
    "modbus_rtu_connect",
    { params }
  );
  return res.data;
}

/** 断开 Modbus 连接 */
export async function modbusDisconnect(): Promise<void> {
  await invoke<ApiResponse<null>>("modbus_disconnect");
}

/** 获取当前连接状态 */
export async function modbusConnectionStatus(): Promise<ConnectionStatus> {
  const res = await invoke<ApiResponse<ConnectionStatus>>(
    "modbus_connection_status"
  );
  return res.data;
}

// ─── 读操作 ────────────────────────────────────────────────

/** 读取保持寄存器 FC03 */
export async function modbusReadHoldingRegisters(
  params: ReadRegistersParams
): Promise<RegistersData> {
  const res = await invoke<ApiResponse<RegistersData>>(
    "modbus_read_holding_registers",
    { params }
  );
  return res.data;
}

/** 读取输入寄存器 FC04 */
export async function modbusReadInputRegisters(
  params: ReadRegistersParams
): Promise<RegistersData> {
  const res = await invoke<ApiResponse<RegistersData>>(
    "modbus_read_input_registers",
    { params }
  );
  return res.data;
}

/** 读取线圈状态 FC01 */
export async function modbusReadCoils(
  params: ReadCoilsParams
): Promise<CoilsData> {
  const res = await invoke<ApiResponse<CoilsData>>("modbus_read_coils", {
    params
  });
  return res.data;
}

/** 读取离散输入 FC02 */
export async function modbusReadDiscreteInputs(
  params: ReadCoilsParams
): Promise<CoilsData> {
  const res = await invoke<ApiResponse<CoilsData>>(
    "modbus_read_discrete_inputs",
    { params }
  );
  return res.data;
}

// ─── 写操作 ────────────────────────────────────────────────

/** 写入单个寄存器 FC06 */
export async function modbusWriteSingleRegister(
  params: WriteSingleRegisterParams
): Promise<void> {
  await invoke<ApiResponse<null>>("modbus_write_single_register", { params });
}

/** 写入多个寄存器 FC16 */
export async function modbusWriteMultipleRegisters(
  params: WriteMultipleRegistersParams
): Promise<void> {
  await invoke<ApiResponse<null>>("modbus_write_multiple_registers", {
    params
  });
}
```

### 9.3 Vue 组件示例用法

```vue
<script setup lang="ts">
import { ref } from "vue";
import {
  modbusTcpConnect,
  modbusReadHoldingRegisters,
  modbusWriteSingleRegister,
  modbusDisconnect
} from "@/api/modbus";

const connected = ref(false);
const registers = ref<number[]>([]);
const errorMsg = ref("");

async function connect() {
  try {
    errorMsg.value = "";
    const status = await modbusTcpConnect({
      host: "192.168.1.100",
      port: 502,
      slaveId: 1
    });
    connected.value = status.connected;
  } catch (e) {
    errorMsg.value = String(e);
  }
}

async function readRegisters() {
  try {
    const data = await modbusReadHoldingRegisters({
      startAddress: 0x0000,
      count: 10
    });
    registers.value = data.values;
  } catch (e) {
    errorMsg.value = String(e);
  }
}

async function writeValue(address: number, value: number) {
  try {
    await modbusWriteSingleRegister({ address, value });
  } catch (e) {
    errorMsg.value = String(e);
  }
}
</script>
```

---

## 10. Tauri 权限配置

Modbus TCP 使用网络访问，RTU 使用串口访问。Tauri v2 的权限系统要求显式声明这些能力。

### 10.1 检查现有 capabilities 文件

```bash
ls src-tauri/capabilities/
```

### 10.2 如需访问网络（TCP 连接外部设备）

Tauri v2 中 Rust 侧发起的 TCP 连接（`tokio::net::TcpStream`）**不受 CSP/capabilities 限制**，因为这是后端代码直接调用系统 API。无需额外权限配置。

> **重要**：CSP 和 capabilities 仅限制 WebView 中的前端代码。Rust 命令函数中的 TCP 连接由操作系统直接管控，不通过 WebView 沙箱。

### 10.3 如需访问串口（RTU）

同上，Rust 侧的 `tokio-serial` 直接调用系统串口 API，**不需要 Tauri capabilities 配置**。

### 10.4 如未来需要前端直接访问（不推荐）

若需通过 WebView 直接访问，则需要在 `capabilities/*.json` 中显式添加相关权限。但本架构采用的是 **Rust 命令中转**模式，更安全，无需此配置。

---

## 11. Windows 平台串口注意事项

### 11.1 驱动安装

- USB 转 RS-485 适配器通常需要安装 **CP210x**（Silicon Labs）或 **CH340/CH341** 驱动
- 设备管理器中确认串口编号：`设备管理器 → 端口（COM 和 LPT）`

### 11.2 权限问题

Windows 上普通用户通常有权限访问 COM 口，无需额外配置。若遇到权限拒绝，以管理员身份运行即可。

### 11.3 串口独占问题

同一时刻只能有一个进程占用串口。使用前请确认没有其他程序（如设备调试工具）占用同一 COM 口。

### 11.4 COM 口号查询（PowerShell）

```powershell
# 列出所有可用串口
[System.IO.Ports.SerialPort]::GetPortNames()

# 或通过 WMI 查询
Get-WMIObject -Query "SELECT * FROM Win32_PnPEntity WHERE Name like '%(COM%'" | Select Name
```

---

## 12. 测试策略

### 12.1 单元测试（不依赖真实设备）

在 `modbus/services.rs` 底部添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::modbus::state::ModbusState;

    #[tokio::test]
    async fn test_disconnect_when_not_connected() {
        let state = ModbusState::new();
        // 未连接时断开不应报错
        assert!(services::disconnect(&state).await.is_ok());
    }

    #[tokio::test]
    async fn test_status_when_not_connected() {
        let state = ModbusState::new();
        let status = services::get_connection_status(&state).await;
        assert!(!status.connected);
        assert_eq!(status.connection_type, "none");
    }

    #[tokio::test]
    async fn test_read_when_not_connected_returns_error() {
        let state = ModbusState::new();
        let result = services::read_holding_registers(
            &state,
            crate::modbus::models::ReadRegistersParams {
                start_address: 0,
                count: 1,
            },
        )
        .await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not connected"));
    }
}
```

### 12.2 集成测试（需要 Modbus 模拟器）

推荐使用开源 Modbus 模拟器进行集成测试：

| 工具                        | 平台          | 说明                            |
| --------------------------- | ------------- | ------------------------------- |
| **Modbus Poll + Slave**     | Windows       | 商业工具，功能完整              |
| **diagslave**               | Linux/Windows | 命令行 Modbus TCP 服务器        |
| **modbus-server（Python）** | 跨平台        | `pip install pymodbus` 快速搭建 |

**Python 快速搭建测试服务器**：

```bash
pip install pymodbus
python -c "
from pymodbus.server import StartTcpServer
from pymodbus.datastore import ModbusSlaveContext, ModbusServerContext
from pymodbus.datastore import ModbusSequentialDataBlock

store = ModbusSlaveContext(
    hr=ModbusSequentialDataBlock(0, [100+i for i in range(100)])
)
context = ModbusServerContext(slaves=store, single=True)
print('Starting Modbus TCP server on 127.0.0.1:5020')
StartTcpServer(context=context, address=('127.0.0.1', 5020))
"
```

### 12.3 运行测试

```bash
# 仅运行 Modbus 模块测试
cargo test --manifest-path src-tauri/Cargo.toml modbus

# 运行全部测试
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## 13. 生产部署检查清单

### 13.1 编译阶段

- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` 零警告
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check` 格式无差异
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` 全部测试通过
- [ ] 确认 `Cargo.lock` 已提交（保证可复现构建）

### 13.2 功能验证

- [ ] TCP 连接建立 / 断开正常
- [ ] RTU 串口连接（如需）建立 / 断开正常
- [ ] 读取保持寄存器（FC03）数据正确
- [ ] 读取输入寄存器（FC04）数据正确
- [ ] 写入单个寄存器（FC06）生效
- [ ] 写入多个寄存器（FC16）生效
- [ ] 连接异常时（设备断电）错误信息清晰

### 13.3 边界情况验证

- [ ] 未连接时调用读写操作，前端收到 `"modbus error: not connected"`
- [ ] 连接超时时错误信息正确传递到前端
- [ ] 并发调用多个读命令不会产生死锁
- [ ] 应用退出时连接自动释放（Tauri state drop 时自动发生）

### 13.4 Windows 打包验证

```bash
pnpm build:tauri
pnpm tauri build
# 检查安装包是否生成
ls src-tauri/target/release/bundle/
```

---

## 14. 常见问题排查

### Q1: TCP 连接超时，报 `connection refused`

**原因**：目标设备未开启 Modbus TCP 服务，或 IP/端口填写错误。  
**排查**：

```bash
# 测试端口是否可达（PowerShell）
Test-NetConnection -ComputerName 192.168.1.100 -Port 502
```

### Q2: RTU 连接报 `No such file or directory` 或 `Access denied`

**原因（Linux）**：串口权限不足。  
**解决**：

```bash
sudo usermod -aG dialout $USER
# 重新登录后生效
```

**原因（Windows）**：串口被其他程序占用，或驱动未安装。  
**解决**：在设备管理器中确认 COM 号，关闭其他调试工具。

### Q3: 读取数据乱码或全零

**原因**：Slave ID 不匹配，或寄存器地址偏移错误。  
**排查**：确认设备手册中的 Slave ID 和寄存器地址（注意 0-based vs 1-based 差异）。

> **地址偏移说明**：Modbus 协议地址从 0 开始，但设备手册通常标注从 1 开始。
> 例如手册写 `40001`（保持寄存器 1）对应协议地址 `0x0000`。

### Q4: 编译报错 `rtu feature not enabled`

**原因**：`Cargo.toml` 中未启用 `rtu` feature。  
**解决**：修改 `tokio-modbus` 依赖：

```toml
tokio-modbus = { version = "0.17", features = ["tcp", "rtu"] }
```

### Q5: 前端收到 `"modbus error: not connected"` 但已调用连接

**原因**：连接调用可能因异常静默失败，或状态未正确传递。  
**排查**：先调用 `modbus_connection_status` 确认连接状态，再调用读写操作。

### Q6: 多个并发读取命令死锁

**原因**：`Mutex` 被单个长时间持有的操作阻塞。  
**解决**：确保每个 `services` 函数在完成后释放 `MutexGuard`（本文档实现已正确处理，不应发生）。如遇此问题，检查是否有额外的 `lock()` 调用未释放。

## 15. 业务功能设计：网关与数据采集配置

为了实现完整的能源管理系统设备接入，在底层 API 之上，需要构建一套**可视化配置与自动化轮询机制**。本章详细说明如何从业务层面设计“网关”、“设备”、“数据点”的配置与数据采集闭环。

### 15.1 配置 Modbus TCP 网关

在物联网业务逻辑中，物理网络设备（如串口服务器、带以太网模块的 PLC）被称为“网关”。

#### 1. 网关基础信息配置

前端提供网关管理界面，用户需配置以下基础要素：

- **名称**：用于业务标识的自定义别名（如“一号车间主网关”）。
- **IP 地址**：目标设备的局域网 IPv4/v6 地址（如 `192.168.1.100`）。
- **端口号**：标准 Modbus TCP 端口通常为 502。

#### 2. 连接测试与状态回显

在保存网关配置前，系统提供 **[点击测试]** 按钮验证设备可用性：

- **独立测试会话**：测试操作不应影响或占用系统全局的正常运行连接。后端需创建一个带有严格超时时间（如 3 秒）的即用即抛型测试会话。
- **状态可视化**：测试完成后，无论成功失败都立刻断开连接。前端根据后端返回的结果，在界面上通过状态指示灯（如绿灯代表连通，红灯代表超时或拒绝连接）及提示信息，直观显示网关的当前网络状态。

### 15.2 配置 Modbus 数据模型（从站与点表映射）

网关连通后，需在其下方挂载具体的“设备（从站）”并定义“数据点（测点）”的具体解析规则。

#### 1. 从站设备定义

- **从站地址 (Slave ID)**：配置设备在总线上的逻辑地址（范围 1-247）。
- **地址合法性范围**：可选配置起始与结束地址，用于在用户录入具体数据点时，提前拦截和校验地址是否越界。

#### 2. 数据点配置（点表）

在一个从站下，用户可通过表格动态新增需采集的数据点。每条数据需明确以下业务规则：

- **映射名称**：业务侧显示和引用的英文/拼音键名（如 `voltage_a`, `temperature`）。
- **功能码**：决定读写区域（如 `01 线圈`, `02 离散输入`, `03 保持寄存器`, `04 输入寄存器`）。
- **起始地址**：设备手册提供的原始数据首地址（注意区分 0-based 与 1-based 逻辑）。
- **数据条数**：连续读取的寄存器或线圈占用数量。
- **数据类型**：业务需要呈现的最终数据类型（如 `Boolean`, `UInt16`, `Int16`, `UInt32`, `Float32`, `String` 等），这决定了原始字节如何被解析。
- **字节序/端序**：针对跨寄存器数据（如 32 位浮点数），配置高低字节的拼接顺序（如 `AB`, `BA`, `ABCD`, `CDAB`）。

#### 3. 读写自适应测试

为确保点表配置准确，界面为每行数据提供独立的 **[读/写测试]** 功能：

- **自动匹配逻辑**：用户点击测试时，前端根据该行的“功能码”和“数据类型”，自动计算目标长度，并匹配底层的具体接口（例如：类型为 Float32、功能码为 03、必定对应读取 2 个保持寄存器的接口）。
- **数据编解码**：
  - **读测试**：系统读取底层原始字节数据后，依照配置的“数据类型”和“字节序”自动解码为人类可读的数值，并在界面直接显示结果。
  - **写测试**：若功能码支持写入（如 01 或 03），系统弹出数值输入框，将用户输入的业务数值编码为对应的双字节或多字节数组后，下发至目标设备。

### 15.3 配置采集周期与实时数据展示

数据模型配置无误后，系统进入工业控制场景下的自动化轮询与监控阶段。

#### 1. 采集周期配置

- 用户可在“网关”或“从站”层级配置独立的 **采集周期 (Polling Interval)**，单位为毫秒（如 2000ms 代表每两秒采集一次）。

#### 2. 后端自动化轮询机制

- **脱离前端依赖**：为避免前端浏览器休眠或页面切换导致采集停止，定时轮询任务必须完全由 **Rust 后端** 的异步守护进程（守护线程）接管。
- **读取指令合并与优化**：后端在发起轮询前，应分析当前设备下的所有数据点地址分布。将地址连续或相近的点位合并为一条长读取指令，从而大幅减少物理层面的网络请求次数，提升总线效率。
- **数据解析集成**：后端拿到合并后的原始响应报文，依据该从站的数据模型定义，将报文切片并解析为具体的业务键值对字典（例如 `{"voltage_a": 220.5, "running_status": true}`）。

#### 3. 实时状态推送与展示

- **事件实时广播**：后端解析重组完成后，将携带时间戳、网关 ID、从站 ID 及解析后业务数据的事件载荷，通过系统的进程间通信（IPC Events）机制主动推送到前端。
- **动态无刷新渲染**：前端无需使用定时器发送拉取请求，只需全局订阅该数据广播事件。当接收到匹配当前监控页面的数据时，直接驱动 UI 组件（如数据大屏、仪表盘、实时曲线图）进行动态无刷新重绘。

---

## 相关文档

- [tokio-modbus 官方文档](https://docs.rs/tokio-modbus/0.17.0/tokio_modbus/)
- [tokio-modbus GitHub 示例](https://github.com/slowtec/tokio-modbus/tree/main/examples)
- [Modbus 应用协议规范 v1.1b3](http://modbus.org/docs/Modbus_Application_Protocol_V1_1b3.pdf)
- [Tauri 框架约束规范](./tauri-framework-constraints.md)
- [项目开发进度](./development-progress.md)
- [后端工程指南](../src-tauri/README.md)
