package com.peacock.app.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.WifiOff
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.peacock.app.models.AppState
import com.peacock.app.models.DeviceInfo
import com.peacock.app.ui.theme.*
import java.text.SimpleDateFormat
import java.util.*

@Composable
fun DeviceListScreen(appState: AppState, onDeviceClick: (String) -> Unit) {
    val devices = appState.devices.values.filter { it.isOnline }.toList()
    var searchQuery by remember { mutableStateOf("") }
    val pc = LocalPeacockColors.current
    val dateFormat = remember { SimpleDateFormat("HH:mm", Locale.getDefault()) }

    val filtered = if (searchQuery.isBlank()) devices else devices.filter {
        it.deviceName.contains(searchQuery, ignoreCase = true) ||
        it.ipAddr.contains(searchQuery)
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(pc.background)
            .padding(horizontal = 16.dp)
    ) {
        Spacer(modifier = Modifier.height(16.dp))
        // Title row
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                "设备",
                fontSize = 30.sp,
                fontWeight = FontWeight.Bold,
                color = pc.primaryText
            )
            Spacer(modifier = Modifier.weight(1f))
            if (devices.isNotEmpty()) {
                Surface(
                    shape = CircleShape,
                    color = PrimaryTeal
                ) {
                    Text(
                        "${devices.size}",
                        modifier = Modifier.padding(horizontal = 8.dp, vertical = 2.dp),
                        color = Color.White,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(12.dp))

        // Search
        OutlinedTextField(
            value = searchQuery,
            onValueChange = { searchQuery = it },
            modifier = Modifier.fillMaxWidth(),
            placeholder = { Text("搜索设备...", color = pc.tertiaryText) },
            shape = RoundedCornerShape(12.dp),
            singleLine = true,
            colors = OutlinedTextFieldDefaults.colors(
                unfocusedContainerColor = pc.secondaryBg,
                focusedContainerColor = pc.secondaryBg,
                unfocusedBorderColor = Color.Transparent,
                focusedBorderColor = PrimaryTeal,
            )
        )

        Spacer(modifier = Modifier.height(12.dp))

        if (filtered.isEmpty()) {
            // Empty state
            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center
            ) {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    modifier = Modifier.offset(y = (-30).dp)
                ) {
                    Icon(
                        Icons.Default.WifiOff,
                        contentDescription = null,
                        modifier = Modifier.size(40.dp),
                        tint = pc.tertiaryText
                    )
                    Spacer(modifier = Modifier.height(12.dp))
                    Text(
                        "未发现设备",
                        fontSize = 16.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = pc.secondaryText
                    )
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(
                        "确保其他设备在同一局域网中",
                        fontSize = 14.sp,
                        color = pc.tertiaryText
                    )
                }
            }
        } else {
            LazyColumn(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                items(filtered) { device ->
                    DeviceRow(
                        device = device,
                        appState = appState,
                        dateFormat = dateFormat,
                        pc = pc,
                        onClick = { onDeviceClick(device.deviceId) }
                    )
                }
            }
        }
    }
}

@Composable
private fun DeviceRow(
    device: DeviceInfo,
    appState: AppState,
    dateFormat: SimpleDateFormat,
    pc: PeacockColors,
    onClick: () -> Unit
) {
    val lastMsg = appState.messages[device.deviceId]?.lastOrNull()
    val unread = appState.messages[device.deviceId]
        ?.count { it.direction == "received" && it.status != "read" } ?: 0

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .clickable(onClick = onClick)
            .padding(vertical = 12.dp, horizontal = 4.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        // Gradient avatar with platform icon
        Box(
            modifier = Modifier
                .size(48.dp)
                .clip(RoundedCornerShape(12.dp))
                .background(
                    Brush.linearGradient(
                        colors = listOf(PrimaryTeal, PrimaryDark),
                        start = Offset(0f, 0f),
                        end = Offset(48f, 48f)
                    )
                ),
            contentAlignment = Alignment.Center
        ) {
            Text(
                platformEmoji(device.platform),
                fontSize = 20.sp
            )
        }

        Spacer(modifier = Modifier.width(12.dp))

        // Name + subtitle
        Column(modifier = Modifier.weight(1f)) {
            Text(
                device.deviceName,
                fontWeight = FontWeight.SemiBold,
                fontSize = 16.sp,
                color = pc.primaryText,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            Text(
                lastMsg?.let { previewText(it.content, it.msgType) } ?: device.ipAddr,
                fontSize = 13.sp,
                color = pc.secondaryText,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }

        Spacer(modifier = Modifier.width(8.dp))

        // Right side: time + badge
        Column(horizontalAlignment = Alignment.End) {
            Text(
                lastMsg?.let { dateFormat.format(Date(it.timestamp)) } ?: "",
                fontSize = 12.sp,
                color = pc.tertiaryText
            )
            // Unread badge placeholder - for now just show green dot
            if (unread > 0) {
                Spacer(modifier = Modifier.height(4.dp))
                Surface(
                    shape = CircleShape,
                    color = PrimaryTeal,
                    modifier = Modifier.defaultMinSize(minWidth = 20.dp, minHeight = 20.dp)
                ) {
                    Text(
                        "$unread",
                        modifier = Modifier.padding(horizontal = 6.dp, vertical = 1.dp),
                        color = Color.White,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                }
            }
        }
    }
}

private fun platformEmoji(platform: String): String = when (platform.lowercase()) {
    "ios" -> "📱"
    "macos" -> "💻"
    "windows" -> "🖥️"
    "linux" -> "🐧"
    "android" -> "🤖"
    else -> "📡"
}

private fun previewText(content: String, msgType: String): String = when (msgType) {
    "file" -> "[文件] $content"
    "snippet" -> content
    else -> content
}
