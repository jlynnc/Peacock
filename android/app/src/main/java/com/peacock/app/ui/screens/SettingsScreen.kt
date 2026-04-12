package com.peacock.app.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.peacock.app.models.AppState
import com.peacock.app.ui.theme.*

@Composable
fun SettingsScreen(appState: AppState) {
    var showNameDialog by remember { mutableStateOf(false) }
    val pc = LocalPeacockColors.current

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(pc.background)
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 16.dp)
    ) {
        Spacer(modifier = Modifier.height(16.dp))
        Text(
            "设置",
            fontSize = 30.sp,
            fontWeight = FontWeight.Bold,
            color = pc.primaryText
        )

        Spacer(modifier = Modifier.height(20.dp))

        // Device card
        Surface(
            shape = RoundedCornerShape(12.dp),
            color = pc.secondaryBg
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable { showNameDialog = true }
                    .padding(16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                // Avatar
                Box(
                    modifier = Modifier
                        .size(56.dp)
                        .clip(RoundedCornerShape(12.dp))
                        .background(
                            Brush.linearGradient(
                                colors = listOf(PrimaryTeal, PrimaryDark),
                                start = Offset(0f, 0f),
                                end = Offset(56f, 56f)
                            )
                        ),
                    contentAlignment = Alignment.Center
                ) {
                    Text("Me", color = Color.White, fontSize = 16.sp, fontWeight = FontWeight.Bold)
                }
                Spacer(modifier = Modifier.width(14.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        appState.deviceName.value,
                        fontWeight = FontWeight.SemiBold,
                        fontSize = 16.sp,
                        color = pc.primaryText
                    )
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Surface(
                            shape = RoundedCornerShape(4.dp),
                            color = PrimaryLight
                        ) {
                            Text(
                                "ANDROID",
                                modifier = Modifier.padding(horizontal = 6.dp, vertical = 1.dp),
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Medium,
                                color = PrimaryTeal
                            )
                        }
                    }
                }
                Icon(
                    Icons.Default.Edit,
                    contentDescription = "Edit",
                    tint = pc.tertiaryText,
                    modifier = Modifier.size(18.dp)
                )
            }
        }

        Spacer(modifier = Modifier.height(24.dp))

        // Transfer section
        SectionHeader("传输", pc)
        SettingsGroup(pc) {
            SettingsRow(
                icon = Icons.Default.Folder,
                label = "默认下载目录",
                value = "Peacock Downloads",
                pc = pc
            )
            HorizontalDivider(modifier = Modifier.padding(start = 44.dp), color = pc.separator, thickness = 0.5.dp)
            SettingsRow(
                icon = Icons.Default.Download,
                label = "自动接收文件",
                pc = pc
            ) {
                Switch(
                    checked = false,
                    onCheckedChange = { },
                    colors = SwitchDefaults.colors(checkedTrackColor = PrimaryTeal)
                )
            }
            HorizontalDivider(modifier = Modifier.padding(start = 44.dp), color = pc.separator, thickness = 0.5.dp)
            SettingsRow(
                icon = Icons.Default.SwapHoriz,
                label = "最大并发传输",
                value = "10",
                pc = pc
            )
        }

        Spacer(modifier = Modifier.height(24.dp))

        // Appearance section
        SectionHeader("外观", pc)
        SettingsGroup(pc) {
            SettingsRow(
                icon = Icons.Default.DarkMode,
                label = "暗色主题",
                value = "跟随系统",
                pc = pc
            )
            HorizontalDivider(modifier = Modifier.padding(start = 44.dp), color = pc.separator, thickness = 0.5.dp)
            SettingsRow(
                icon = Icons.Default.Language,
                label = "语言",
                value = "简体中文",
                pc = pc
            )
        }

        Spacer(modifier = Modifier.height(24.dp))

        // About section
        SectionHeader("关于 Peacock", pc)
        SettingsGroup(pc) {
            SettingsRow(
                icon = Icons.Default.Info,
                label = "版本",
                value = "v0.1.0",
                pc = pc
            )
            HorizontalDivider(modifier = Modifier.padding(start = 44.dp), color = pc.separator, thickness = 0.5.dp)
            SettingsRow(
                icon = Icons.Default.Router,
                label = "协议版本",
                value = "PCOK v1",
                pc = pc
            )
            HorizontalDivider(modifier = Modifier.padding(start = 44.dp), color = pc.separator, thickness = 0.5.dp)
            SettingsRow(
                icon = Icons.Default.Key,
                label = "设备 ID",
                value = appState.deviceIdStr.take(8) + "...",
                pc = pc
            )
        }

        Spacer(modifier = Modifier.height(32.dp))
    }

    // Rename dialog
    if (showNameDialog) {
        var newName by remember { mutableStateOf(appState.deviceName.value) }
        AlertDialog(
            onDismissRequest = { showNameDialog = false },
            title = { Text("修改设备名称") },
            text = {
                OutlinedTextField(
                    value = newName,
                    onValueChange = { newName = it },
                    singleLine = true,
                    shape = RoundedCornerShape(8.dp)
                )
            },
            confirmButton = {
                TextButton(onClick = {
                    if (newName.isNotBlank()) {
                        appState.updateDeviceName(newName.trim())
                    }
                    showNameDialog = false
                }) {
                    Text("确认", color = PrimaryTeal)
                }
            },
            dismissButton = {
                TextButton(onClick = { showNameDialog = false }) {
                    Text("取消")
                }
            }
        )
    }
}

@Composable
private fun SectionHeader(title: String, pc: PeacockColors) {
    Text(
        title,
        fontSize = 13.sp,
        fontWeight = FontWeight.Medium,
        color = pc.secondaryText,
        modifier = Modifier.padding(start = 4.dp, bottom = 8.dp)
    )
}

@Composable
private fun SettingsGroup(pc: PeacockColors, content: @Composable ColumnScope.() -> Unit) {
    Surface(
        shape = RoundedCornerShape(12.dp),
        color = pc.secondaryBg
    ) {
        Column(content = content)
    }
}

@Composable
private fun SettingsRow(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    label: String,
    value: String? = null,
    pc: PeacockColors,
    onClick: (() -> Unit)? = null,
    trailing: @Composable (() -> Unit)? = null
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .then(if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier)
            .padding(horizontal = 14.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            icon,
            contentDescription = null,
            modifier = Modifier.size(20.dp),
            tint = pc.secondaryText
        )
        Spacer(modifier = Modifier.width(10.dp))
        Text(
            label,
            fontSize = 15.sp,
            color = pc.primaryText,
            modifier = Modifier.weight(1f)
        )
        if (trailing != null) {
            trailing()
        } else if (value != null) {
            Text(
                value,
                fontSize = 14.sp,
                color = pc.secondaryText
            )
        }
    }
}
