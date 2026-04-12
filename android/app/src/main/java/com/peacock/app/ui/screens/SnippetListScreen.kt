package com.peacock.app.ui.screens

import android.widget.Toast
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.peacock.app.models.AppState
import com.peacock.app.models.Snippet
import com.peacock.app.ui.theme.*
import java.text.SimpleDateFormat
import java.util.*
import kotlin.math.roundToInt

@Composable
fun SnippetListScreen(appState: AppState, onSnippetClick: (String) -> Unit) {
    val snippets = appState.snippets
    var searchQuery by remember { mutableStateOf("") }
    val dateFormat = remember { SimpleDateFormat("M/d HH:mm", Locale.getDefault()) }
    val pc = LocalPeacockColors.current
    val context = LocalContext.current

    // Track which snippet has swipe actions open
    var openSwipeId by remember { mutableStateOf<String?>(null) }

    // Dialogs
    var renameSnippetId by remember { mutableStateOf<String?>(null) }
    var shareSnippetId by remember { mutableStateOf<String?>(null) }

    val filtered = if (searchQuery.isBlank()) snippets else snippets.filter {
        it.title.contains(searchQuery, ignoreCase = true) ||
        it.content.contains(searchQuery, ignoreCase = true)
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(pc.background)
            .padding(horizontal = 16.dp)
    ) {
        Spacer(modifier = Modifier.height(16.dp))

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text("片段", fontSize = 30.sp, fontWeight = FontWeight.Bold, color = pc.primaryText)
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(CircleShape)
                    .background(PrimaryTeal)
                    .clickable {
                        val s = appState.createSnippet()
                        onSnippetClick(s.id)
                    },
                contentAlignment = Alignment.Center
            ) {
                Icon(Icons.Default.Add, contentDescription = "New", tint = Color.White, modifier = Modifier.size(20.dp))
            }
        }

        Spacer(modifier = Modifier.height(12.dp))

        OutlinedTextField(
            value = searchQuery,
            onValueChange = { searchQuery = it },
            modifier = Modifier.fillMaxWidth(),
            placeholder = { Text("搜索片段...", color = pc.tertiaryText) },
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
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Icon(Icons.Default.Description, contentDescription = null,
                        modifier = Modifier.size(40.dp), tint = pc.tertiaryText)
                    Spacer(modifier = Modifier.height(12.dp))
                    Text("暂无片段", fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = pc.secondaryText)
                    Spacer(modifier = Modifier.height(4.dp))
                    Text("点击右上角 + 创建", fontSize = 14.sp, color = pc.tertiaryText)
                }
            }
        } else {
            LazyColumn(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                items(filtered, key = { it.id }) { snippet ->
                    SwipeActionRow(
                        isOpen = openSwipeId == snippet.id,
                        onSwipeOpen = { openSwipeId = snippet.id },
                        onSwipeClose = { if (openSwipeId == snippet.id) openSwipeId = null },
                        actions = listOf(
                            SwipeAction(Icons.Default.Edit, BlueAccent, "重命名") {
                                renameSnippetId = snippet.id
                                openSwipeId = null
                            },
                            SwipeAction(Icons.Default.Share, PrimaryTeal, "分享") {
                                shareSnippetId = snippet.id
                                openSwipeId = null
                            },
                            SwipeAction(Icons.Default.PushPin, OrangeAccent, "置顶") {
                                appState.pinSnippetToTop(snippet.id)
                                openSwipeId = null
                            },
                            SwipeAction(Icons.Default.Delete, DangerRed, "删除") {
                                appState.deleteSnippet(snippet.id)
                                openSwipeId = null
                            }
                        )
                    ) {
                        SnippetRow(snippet, dateFormat, pc) {
                            if (openSwipeId != null) {
                                openSwipeId = null
                            } else {
                                onSnippetClick(snippet.id)
                            }
                        }
                    }
                }
            }
        }
    }

    // Rename dialog
    renameSnippetId?.let { id ->
        val snippet = appState.snippets.find { it.id == id }
        if (snippet != null) {
            var newName by remember { mutableStateOf(snippet.title) }
            AlertDialog(
                onDismissRequest = { renameSnippetId = null },
                title = { Text("重命名") },
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
                            appState.updateSnippet(id, title = newName.trim())
                        }
                        renameSnippetId = null
                    }) { Text("确认", color = PrimaryTeal) }
                },
                dismissButton = {
                    TextButton(onClick = { renameSnippetId = null }) { Text("取消") }
                }
            )
        }
    }

    // Share dialog
    shareSnippetId?.let { id ->
        val onlineDevices = appState.devices.values.filter { it.isOnline }.toList()
        AlertDialog(
            onDismissRequest = { shareSnippetId = null },
            title = { Text("分享到设备") },
            text = {
                if (onlineDevices.isEmpty()) {
                    Text("没有在线设备", color = pc.secondaryText)
                } else {
                    Column {
                        onlineDevices.forEach { device ->
                            Surface(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .clickable {
                                        appState.shareSnippet(id, device.deviceId)
                                        shareSnippetId = null
                                        Toast.makeText(context, "已分享到 ${device.deviceName}", Toast.LENGTH_SHORT).show()
                                    },
                                shape = RoundedCornerShape(8.dp)
                            ) {
                                Row(modifier = Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
                                    Icon(Icons.Default.Devices, contentDescription = null, tint = PrimaryTeal)
                                    Spacer(modifier = Modifier.width(12.dp))
                                    Column {
                                        Text(device.deviceName, fontWeight = FontWeight.Medium)
                                        Text("${device.platform} · ${device.ipAddr}", fontSize = 12.sp, color = pc.secondaryText)
                                    }
                                }
                            }
                        }
                    }
                }
            },
            confirmButton = {
                TextButton(onClick = { shareSnippetId = null }) { Text("取消") }
            }
        )
    }
}

// ── Swipe action row with 4 buttons ──

data class SwipeAction(
    val icon: ImageVector,
    val color: Color,
    val label: String,
    val onClick: () -> Unit
)

@Composable
private fun SwipeActionRow(
    isOpen: Boolean,
    onSwipeOpen: () -> Unit,
    onSwipeClose: () -> Unit,
    actions: List<SwipeAction>,
    content: @Composable () -> Unit
) {
    val density = LocalDensity.current
    val actionButtonWidth = with(density) { 64.dp.toPx() }
    val totalActionsWidth = actionButtonWidth * actions.size

    // Animate based on open/close state
    val targetOffset = if (isOpen) -totalActionsWidth else 0f
    val animatedOffset by animateFloatAsState(
        targetValue = targetOffset,
        animationSpec = tween(200),
        label = "swipeOffset"
    )

    var dragOffset by remember { mutableStateOf(0f) }
    var isDragging by remember { mutableStateOf(false) }

    val currentOffset = if (isDragging) dragOffset else animatedOffset

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clipToBounds()
    ) {
        // Action buttons (behind the content, aligned to the right)
        Row(
            modifier = Modifier
                .align(Alignment.CenterEnd)
                .height(IntrinsicSize.Min),
        ) {
            actions.forEach { action ->
                Box(
                    modifier = Modifier
                        .width(64.dp)
                        .fillMaxHeight()
                        .background(action.color)
                        .clickable { action.onClick() },
                    contentAlignment = Alignment.Center
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Icon(
                            action.icon,
                            contentDescription = action.label,
                            tint = Color.White,
                            modifier = Modifier.size(20.dp)
                        )
                        Text(action.label, color = Color.White, fontSize = 10.sp)
                    }
                }
            }
        }

        // Foreground content (slides left)
        Box(
            modifier = Modifier
                .offset { IntOffset(currentOffset.roundToInt(), 0) }
                .pointerInput(Unit) {
                    detectHorizontalDragGestures(
                        onDragStart = {
                            isDragging = true
                            dragOffset = if (isOpen) -totalActionsWidth else 0f
                        },
                        onDragEnd = {
                            isDragging = false
                            if (dragOffset < -totalActionsWidth * 0.3f) {
                                onSwipeOpen()
                            } else {
                                onSwipeClose()
                            }
                        },
                        onDragCancel = {
                            isDragging = false
                            onSwipeClose()
                        },
                        onHorizontalDrag = { _, dragAmount ->
                            dragOffset = (dragOffset + dragAmount).coerceIn(-totalActionsWidth, 0f)
                        }
                    )
                }
        ) {
            content()
        }
    }
}

@Composable
private fun SnippetRow(
    snippet: Snippet,
    dateFormat: SimpleDateFormat,
    pc: PeacockColors,
    onClick: () -> Unit
) {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick),
        shape = RoundedCornerShape(0.dp),
        color = pc.background
    ) {
        Row(
            modifier = Modifier.padding(vertical = 12.dp, horizontal = 4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    snippet.title,
                    fontWeight = FontWeight.Medium,
                    fontSize = 16.sp,
                    color = pc.primaryText,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                val preview = stripChipMarkers(snippet.content).take(60)
                if (preview.isNotBlank()) {
                    Text(
                        preview,
                        fontSize = 13.sp,
                        color = pc.secondaryText,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.padding(top = 2.dp)
                    )
                }
            }
            Spacer(modifier = Modifier.width(8.dp))
            Text(
                dateFormat.format(Date(snippet.updatedAt * 1000)),
                fontSize = 12.sp,
                color = pc.tertiaryText
            )
        }
    }
}

private fun stripChipMarkers(text: String): String {
    return text.replace(Regex("""\[\[(.+?)\]\]"""), "$1")
}
