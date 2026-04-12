package com.peacock.app.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Description
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.peacock.app.models.TransferTask
import com.peacock.app.ui.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FileOfferDialog(
    task: TransferTask,
    senderName: String,
    onAccept: () -> Unit,
    onReject: () -> Unit
) {
    val pc = LocalPeacockColors.current
    val sheetState = rememberModalBottomSheetState()

    ModalBottomSheet(
        onDismissRequest = onReject,
        sheetState = sheetState,
        containerColor = pc.background,
        shape = RoundedCornerShape(topStart = 20.dp, topEnd = 20.dp)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 24.dp)
                .padding(bottom = 32.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            // File icon
            Icon(
                Icons.Default.Description,
                contentDescription = null,
                modifier = Modifier.size(48.dp),
                tint = PrimaryTeal
            )

            Spacer(modifier = Modifier.height(16.dp))

            // Sender name
            Text(
                senderName,
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold,
                color = pc.primaryText
            )
            Text(
                "想发送${if (task.isFolder) "文件夹" else "文件"}给你",
                fontSize = 14.sp,
                color = pc.secondaryText
            )

            Spacer(modifier = Modifier.height(16.dp))

            // File info
            Text(
                task.fileName,
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold,
                color = pc.primaryText
            )
            Text(
                formatFileSize(task.fileSize) +
                    if (task.isFolder) " · ${task.fileCount} 个文件" else "",
                fontSize = 14.sp,
                color = pc.secondaryText
            )

            Spacer(modifier = Modifier.height(24.dp))

            // Buttons
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                // Reject
                OutlinedButton(
                    onClick = onReject,
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(10.dp),
                    colors = ButtonDefaults.outlinedButtonColors(
                        contentColor = DangerRed
                    ),
                    border = androidx.compose.foundation.BorderStroke(1.dp, DangerRed.copy(alpha = 0.3f)),
                    contentPadding = PaddingValues(vertical = 12.dp)
                ) {
                    Text("拒绝")
                }
                // Accept
                Button(
                    onClick = onAccept,
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(10.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = PrimaryTeal),
                    contentPadding = PaddingValues(vertical = 12.dp)
                ) {
                    Text("接收")
                }
            }
        }
    }
}

fun formatFileSize(bytes: Long): String {
    if (bytes < 1024) return "$bytes B"
    val kb = bytes / 1024.0
    if (kb < 1024) return String.format("%.1f KB", kb)
    val mb = kb / 1024.0
    if (mb < 1024) return String.format("%.1f MB", mb)
    val gb = mb / 1024.0
    return String.format("%.2f GB", gb)
}

fun formatSpeed(bytesPerSec: Long): String {
    if (bytesPerSec < 1024) return "$bytesPerSec B/s"
    val kb = bytesPerSec / 1024.0
    if (kb < 1024) return String.format("%.1f KB/s", kb)
    val mb = kb / 1024.0
    return String.format("%.1f MB/s", mb)
}
