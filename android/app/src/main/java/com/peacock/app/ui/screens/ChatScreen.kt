package com.peacock.app.ui.screens

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.peacock.app.models.AppState
import com.peacock.app.models.ChatMessage
import com.peacock.app.ui.theme.*
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChatScreen(appState: AppState, deviceId: String, onBack: () -> Unit) {
    val device = appState.devices[deviceId]
    val messages = appState.getOrCreateMessages(deviceId)
    var inputText by remember { mutableStateOf("") }
    var showPlusPanel by remember { mutableStateOf(false) }
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()
    val dateFormat = remember { SimpleDateFormat("HH:mm", Locale.getDefault()) }
    val context = LocalContext.current
    val pc = LocalPeacockColors.current

    val filePickerLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetContent()
    ) { uri: Uri? ->
        uri?.let {
            val cursor = context.contentResolver.query(it, null, null, null, null)
            var fileName = "file"
            var fileSize = 0L
            cursor?.use { c ->
                if (c.moveToFirst()) {
                    val nameIdx = c.getColumnIndex(android.provider.OpenableColumns.DISPLAY_NAME)
                    val sizeIdx = c.getColumnIndex(android.provider.OpenableColumns.SIZE)
                    if (nameIdx >= 0) fileName = c.getString(nameIdx)
                    if (sizeIdx >= 0) fileSize = c.getLong(sizeIdx)
                }
            }
            val cacheFile = java.io.File(context.cacheDir, fileName)
            context.contentResolver.openInputStream(it)?.use { input ->
                cacheFile.outputStream().use { output -> input.copyTo(output) }
            }
            if (fileSize == 0L) fileSize = cacheFile.length()
            appState.sendFile(deviceId, cacheFile.absolutePath, fileName, fileSize)
            showPlusPanel = false
        }
    }

    LaunchedEffect(messages.size) {
        if (messages.isNotEmpty()) {
            listState.animateScrollToItem(messages.size - 1)
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text(
                            device?.deviceName ?: "...",
                            fontWeight = FontWeight.SemiBold,
                            fontSize = 17.sp
                        )
                        if (device != null) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Box(
                                    modifier = Modifier
                                        .size(6.dp)
                                        .clip(CircleShape)
                                        .background(OnlineGreen)
                                )
                                Spacer(modifier = Modifier.width(4.dp))
                                Text("在线", fontSize = 11.sp, color = pc.secondaryText)
                            }
                        }
                    }
                },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = pc.background
                )
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .background(pc.background)
        ) {
            // Messages
            LazyColumn(
                modifier = Modifier
                    .weight(1f)
                    .padding(horizontal = 12.dp),
                state = listState,
                verticalArrangement = Arrangement.spacedBy(12.dp),
                contentPadding = PaddingValues(vertical = 8.dp)
            ) {
                items(messages) { msg ->
                    if (msg.msgType == "file") {
                        FileMessageCard(msg, appState, device?.deviceName ?: "", device?.platform ?: "", pc)
                    } else {
                        MessageBubble(msg, device?.deviceName ?: "", device?.platform ?: "", dateFormat, pc)
                    }
                }
            }

            // Plus panel
            AnimatedVisibility(
                visible = showPlusPanel,
                enter = slideInVertically(initialOffsetY = { it }),
                exit = slideOutVertically(targetOffsetY = { it })
            ) {
                PlusPanel(
                    pc = pc,
                    onFile = { filePickerLauncher.launch("*/*") },
                    onSnippet = {
                        // TODO: snippet picker
                        showPlusPanel = false
                    }
                )
            }

            // Input bar
            Surface(
                modifier = Modifier.fillMaxWidth(),
                color = pc.background,
                shadowElevation = 1.dp
            ) {
                Row(
                    modifier = Modifier
                        .padding(horizontal = 12.dp, vertical = 6.dp)
                        .fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    // Plus button
                    Box(
                        modifier = Modifier
                            .size(36.dp)
                            .clip(CircleShape)
                            .clickable { showPlusPanel = !showPlusPanel },
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            if (showPlusPanel) Icons.Default.Close else Icons.Default.Add,
                            contentDescription = "More",
                            modifier = Modifier.size(26.dp),
                            tint = pc.secondaryText
                        )
                    }

                    Spacer(modifier = Modifier.width(8.dp))

                    OutlinedTextField(
                        value = inputText,
                        onValueChange = { inputText = it },
                        modifier = Modifier
                            .weight(1f)
                            .defaultMinSize(minHeight = 36.dp),
                        placeholder = { Text("输入消息...", color = pc.tertiaryText) },
                        shape = RoundedCornerShape(18.dp),
                        singleLine = false,
                        maxLines = 4,
                        colors = OutlinedTextFieldDefaults.colors(
                            unfocusedContainerColor = pc.tertiaryFill,
                            focusedContainerColor = pc.tertiaryFill,
                            unfocusedBorderColor = Color.Transparent,
                            focusedBorderColor = PrimaryTeal,
                        )
                    )

                    Spacer(modifier = Modifier.width(8.dp))

                    // Send button
                    Box(
                        modifier = Modifier
                            .size(36.dp)
                            .clip(CircleShape)
                            .then(
                                if (inputText.isNotBlank())
                                    Modifier
                                        .background(PrimaryTeal)
                                        .clickable {
                                            appState.sendMessage(deviceId, inputText.trim())
                                            inputText = ""
                                            showPlusPanel = false
                                            scope.launch {
                                                if (messages.isNotEmpty())
                                                    listState.animateScrollToItem(messages.size - 1)
                                            }
                                        }
                                else Modifier
                            ),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            Icons.Default.ArrowUpward,
                            contentDescription = "Send",
                            modifier = Modifier.size(20.dp),
                            tint = if (inputText.isNotBlank()) Color.White else pc.tertiaryText
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun PlusPanel(pc: PeacockColors, onFile: () -> Unit, onSnippet: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(pc.background)
            .padding(vertical = 16.dp),
        horizontalArrangement = Arrangement.SpaceEvenly
    ) {
        PlusPanelItem("相册", Icons.Default.PhotoLibrary, BlueAccent, pc) { }
        PlusPanelItem("拍摄", Icons.Default.CameraAlt, OrangeAccent, pc) { }
        PlusPanelItem("文件", Icons.Default.Folder, PurpleAccent, pc, onFile)
        PlusPanelItem("片段", Icons.Default.Description, PrimaryTeal, pc, onSnippet)
    }
}

@Composable
private fun PlusPanelItem(
    label: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    color: Color,
    pc: PeacockColors,
    onClick: () -> Unit = {}
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier.clickable(onClick = onClick)
    ) {
        Box(
            modifier = Modifier
                .size(56.dp)
                .clip(RoundedCornerShape(14.dp))
                .background(color.copy(alpha = 0.1f)),
            contentAlignment = Alignment.Center
        ) {
            Icon(icon, contentDescription = label, tint = color, modifier = Modifier.size(24.dp))
        }
        Spacer(modifier = Modifier.height(8.dp))
        Text(label, fontSize = 12.sp, color = pc.secondaryText)
    }
}

@Composable
private fun MessageBubble(
    msg: ChatMessage,
    deviceName: String,
    platform: String,
    dateFormat: SimpleDateFormat,
    pc: PeacockColors
) {
    val isSent = msg.direction == "sent"

    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = if (isSent) Arrangement.End else Arrangement.Start,
        verticalAlignment = Alignment.Top
    ) {
        // Received avatar
        if (!isSent) {
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .background(pc.tertiaryFill),
                contentAlignment = Alignment.Center
            ) {
                Text(platformEmoji(platform), fontSize = 16.sp)
            }
            Spacer(modifier = Modifier.width(8.dp))
        }

        Column(
            horizontalAlignment = if (isSent) Alignment.End else Alignment.Start,
            modifier = Modifier.weight(1f, fill = false)
        ) {
            if (!isSent) {
                Text(
                    deviceName,
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Medium,
                    color = pc.secondaryText,
                    modifier = Modifier.padding(bottom = 3.dp)
                )
            }
            Surface(
                shape = RoundedCornerShape(18.dp),
                color = if (isSent) pc.sentBubbleBg else pc.receivedBubbleBg,
                border = androidx.compose.foundation.BorderStroke(
                    0.5.dp,
                    if (isSent) pc.sentBubbleBorder else pc.receivedBubbleBorder
                )
            ) {
                Text(
                    msg.content,
                    modifier = Modifier.padding(horizontal = 14.dp, vertical = 10.dp),
                    color = if (isSent) pc.sentBubbleText else pc.primaryText,
                    fontSize = 15.sp
                )
            }
            Text(
                dateFormat.format(Date(msg.timestamp)),
                fontSize = 11.sp,
                color = pc.tertiaryText,
                modifier = Modifier.padding(top = 3.dp)
            )
            if (msg.status == "failed") {
                Text("发送失败", fontSize = 11.sp, color = DangerRed)
            }
        }

        // Sent avatar
        if (isSent) {
            Spacer(modifier = Modifier.width(8.dp))
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .background(
                        Brush.linearGradient(
                            colors = listOf(PrimaryTeal, PrimaryDark),
                            start = Offset(0f, 0f),
                            end = Offset(36f, 36f)
                        )
                    ),
                contentAlignment = Alignment.Center
            ) {
                Text("Me", color = Color.White, fontSize = 12.sp, fontWeight = FontWeight.Bold)
            }
        }
    }
}

@Composable
private fun FileMessageCard(
    msg: ChatMessage,
    appState: AppState,
    deviceName: String,
    platform: String,
    pc: PeacockColors
) {
    val isSent = msg.direction == "sent"
    val task = msg.transferId?.let { appState.transfers[it] }

    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = if (isSent) Arrangement.End else Arrangement.Start,
        verticalAlignment = Alignment.Top
    ) {
        if (!isSent) {
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .background(pc.tertiaryFill),
                contentAlignment = Alignment.Center
            ) {
                Text(platformEmoji(platform), fontSize = 16.sp)
            }
            Spacer(modifier = Modifier.width(8.dp))
        }

        Surface(
            shape = RoundedCornerShape(14.dp),
            color = pc.secondaryBg,
            modifier = Modifier.widthIn(min = 260.dp, max = 340.dp)
        ) {
            Column(modifier = Modifier.padding(12.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    // File icon
                    Box(
                        modifier = Modifier
                            .size(42.dp)
                            .clip(RoundedCornerShape(10.dp))
                            .background(PrimaryLight),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            Icons.Default.Description,
                            contentDescription = null,
                            tint = PrimaryTeal,
                            modifier = Modifier.size(22.dp)
                        )
                    }
                    Spacer(modifier = Modifier.width(10.dp))
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            msg.fileName ?: msg.content,
                            fontWeight = FontWeight.Medium,
                            fontSize = 13.sp,
                            color = PrimaryTeal,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis
                        )
                        Text(
                            formatFileSize(msg.fileSize),
                            fontSize = 11.sp,
                            color = pc.secondaryText
                        )
                    }
                }

                // Progress / status
                if (task != null) {
                    when (task.status) {
                        "active" -> {
                            Spacer(modifier = Modifier.height(8.dp))
                            LinearProgressIndicator(
                                progress = { task.progress },
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .height(4.dp)
                                    .clip(RoundedCornerShape(2.dp)),
                                color = PrimaryTeal,
                                trackColor = PrimaryTeal.copy(alpha = 0.2f)
                            )
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(top = 4.dp),
                                horizontalArrangement = Arrangement.SpaceBetween
                            ) {
                                Text(formatSpeed(task.speedBps), fontSize = 11.sp, color = pc.secondaryText)
                                Text(
                                    "${formatFileSize(task.bytesTransferred)} / ${formatFileSize(task.fileSize)}",
                                    fontSize = 11.sp, color = pc.secondaryText
                                )
                            }
                        }
                        "completed" -> {
                            Spacer(modifier = Modifier.height(8.dp))
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Box(
                                    modifier = Modifier
                                        .size(6.dp)
                                        .clip(CircleShape)
                                        .background(PrimaryTeal)
                                )
                                Spacer(modifier = Modifier.width(6.dp))
                                Text("已完成", fontSize = 12.sp, color = PrimaryTeal, fontWeight = FontWeight.Medium)
                            }
                        }
                        "failed" -> {
                            Spacer(modifier = Modifier.height(8.dp))
                            Text("传输失败", fontSize = 12.sp, color = DangerRed)
                        }
                        "rejected" -> {
                            Spacer(modifier = Modifier.height(8.dp))
                            Text("已拒绝", fontSize = 12.sp, color = pc.secondaryText)
                        }
                        "pending" -> {
                            Spacer(modifier = Modifier.height(8.dp))
                            if (!isSent) {
                                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                    Button(
                                        onClick = { appState.acceptFileOffer(task.transferId) },
                                        colors = ButtonDefaults.buttonColors(containerColor = PrimaryTeal),
                                        shape = RoundedCornerShape(10.dp),
                                        contentPadding = PaddingValues(horizontal = 24.dp, vertical = 10.dp)
                                    ) {
                                        Text("接收")
                                    }
                                    TextButton(
                                        onClick = { appState.rejectFileOffer(task.transferId) }
                                    ) {
                                        Text("拒绝", color = DangerRed)
                                    }
                                }
                            } else {
                                Text("等待对方接收...", fontSize = 11.sp, color = pc.secondaryText)
                            }
                        }
                    }
                }
            }
        }

        if (isSent) {
            Spacer(modifier = Modifier.width(8.dp))
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .background(
                        Brush.linearGradient(
                            colors = listOf(PrimaryTeal, PrimaryDark),
                            start = Offset(0f, 0f),
                            end = Offset(36f, 36f)
                        )
                    ),
                contentAlignment = Alignment.Center
            ) {
                Text("Me", color = Color.White, fontSize = 12.sp, fontWeight = FontWeight.Bold)
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
