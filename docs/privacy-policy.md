# Privacy Policy

**Last updated: April 9, 2026**

## Overview

Peacock ("the App") is a local network file and message transfer tool. Your privacy is fundamental to how Peacock is designed and built. This policy explains how the App handles your data.

## Data Collection

**Peacock does not collect, transmit, or store any personal data on external servers.**

Specifically:
- No user accounts or registration required
- No analytics or tracking of any kind
- No crash reporting sent to external services
- No advertising or ad-related data collection
- No cookies or web tracking

## Local Data Storage

The App stores the following data **locally on your device only**:

- **Messages**: Text messages exchanged between devices are stored in a local SQLite database on each device
- **Snippets**: Text snippets you create are stored locally
- **Settings**: Your preferences (device name, download directory, language, theme) are stored locally
- **Transfer history**: File transfer records are stored locally

This data never leaves your device except when explicitly sent to another device on your local network by your action.

## Network Communication

Peacock communicates **exclusively over your local network (LAN/Wi-Fi)**:

- **Device Discovery**: UDP multicast and broadcast packets are sent on port 52000 within your local network to discover other devices running Peacock
- **Messaging**: TCP connections on port 52001 are used for text messages between devices on your local network
- **File Transfer**: Dynamic TCP ports are used for file transfers between devices on your local network

**No data is ever sent to the internet.** Peacock does not require an internet connection to function.

## Local Network Permission

Peacock requires local network access permission to discover devices and transfer data on your Wi-Fi network. This permission is used solely for the App's core functionality. No data is sent outside your local network.

## Third-Party Services

Peacock does not integrate with any third-party services, SDKs, or APIs.

## Children's Privacy

Peacock does not knowingly collect any information from children under 13 years of age.

## Changes to This Policy

We may update this Privacy Policy from time to time. Any changes will be reflected in the "Last updated" date above.

## Contact

If you have questions about this Privacy Policy, please contact:

- GitHub: [github.com/jlynnc/Peacock](https://github.com/jlynnc/Peacock)
- Email: omicronbai@gmail.com

---

# 隐私政策

**最后更新：2026年4月9日**

## 概述

Peacock（"本应用"）是一款局域网文件与消息传输工具。隐私保护是 Peacock 设计和构建的核心原则。本政策说明本应用如何处理您的数据。

## 数据收集

**Peacock 不会在外部服务器上收集、传输或存储任何个人数据。**

具体而言：
- 无需用户账号或注册
- 不进行任何分析或追踪
- 不向外部服务发送崩溃报告
- 无广告或广告相关数据收集
- 无 Cookie 或网页追踪

## 本地数据存储

本应用仅在**您的设备本地**存储以下数据：

- **消息**：设备间交换的文字消息存储在每台设备的本地 SQLite 数据库中
- **片段**：您创建的文本片段存储在本地
- **设置**：您的偏好设置（设备名称、下载目录、语言、主题）存储在本地
- **传输记录**：文件传输记录存储在本地

这些数据不会离开您的设备，除非您主动将其发送到本地网络上的另一台设备。

## 网络通信

Peacock **仅通过本地网络（LAN/Wi-Fi）通信**：

- **设备发现**：在本地网络的 52000 端口发送 UDP 组播和广播数据包以发现其他运行 Peacock 的设备
- **消息传输**：在本地网络的 52001 端口使用 TCP 连接进行设备间文字消息传输
- **文件传输**：使用动态 TCP 端口在本地网络的设备间进行文件传输

**不会向互联网发送任何数据。** Peacock 不需要互联网连接即可正常使用。

## 本地网络权限

Peacock 需要本地网络访问权限以发现设备并在 Wi-Fi 网络上传输数据。该权限仅用于本应用的核心功能，不会将数据发送到本地网络之外。

## 第三方服务

Peacock 不集成任何第三方服务、SDK 或 API。

## 儿童隐私

Peacock 不会有意收集 13 岁以下儿童的任何信息。

## 政策变更

我们可能会不时更新本隐私政策。任何变更将在上方的"最后更新"日期中体现。

## 联系方式

如对本隐私政策有疑问，请联系：

- GitHub：[github.com/jlynnc/Peacock](https://github.com/jlynnc/Peacock)
- 邮箱：omicronbai@gmail.com
