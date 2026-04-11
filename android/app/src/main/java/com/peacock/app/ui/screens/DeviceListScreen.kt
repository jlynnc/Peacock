package com.peacock.app.ui.screens

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.peacock.app.models.AppState

@Composable
fun DeviceListScreen(appState: AppState, onDeviceClick: (String) -> Unit) {
    val devices = appState.devices.values.filter { it.isOnline }.toList()

    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Text(
            "设备",
            fontSize = 28.sp,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.padding(bottom = 16.dp)
        )

        if (devices.isEmpty()) {
            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text("正在搜索设备...", color = Color.Gray, fontSize = 16.sp)
                    Spacer(modifier = Modifier.height(8.dp))
                    Text("请确保设备在同一局域网", color = Color.Gray, fontSize = 13.sp)
                }
            }
        } else {
            LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                items(devices) { device ->
                    Card(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickable { onDeviceClick(device.deviceId) },
                        shape = RoundedCornerShape(12.dp)
                    ) {
                        Row(
                            modifier = Modifier.padding(16.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            // Green dot
                            Box(
                                modifier = Modifier.size(10.dp),
                                contentAlignment = Alignment.Center
                            ) {
                                Surface(
                                    modifier = Modifier.size(10.dp),
                                    shape = RoundedCornerShape(5.dp),
                                    color = Color(0xFF34C759)
                                ) {}
                            }
                            Spacer(modifier = Modifier.width(12.dp))
                            Column(modifier = Modifier.weight(1f)) {
                                Text(
                                    device.deviceName,
                                    fontWeight = FontWeight.Medium,
                                    fontSize = 16.sp
                                )
                                Text(
                                    "${device.platform} · ${device.ipAddr}",
                                    fontSize = 13.sp,
                                    color = Color.Gray
                                )
                            }
                            // Last message preview
                            val lastMsg = appState.messages[device.deviceId]?.lastOrNull()
                            if (lastMsg != null) {
                                Text(
                                    lastMsg.content.take(20),
                                    fontSize = 12.sp,
                                    color = Color.Gray,
                                    maxLines = 1
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
