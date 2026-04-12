package com.peacock.app.ui.screens

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.widget.Toast
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.peacock.app.models.AppState
import com.peacock.app.ui.theme.*
import kotlinx.coroutines.delay

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SnippetEditorScreen(appState: AppState, snippetId: String, onBack: () -> Unit) {
    val snippet = appState.snippets.find { it.id == snippetId }
    if (snippet == null) {
        onBack()
        return
    }

    var contentTfv by remember {
        mutableStateOf(TextFieldValue(snippet.content, TextRange(snippet.content.length)))
    }
    val content = contentTfv.text
    var note by remember { mutableStateOf(snippet.note) }
    var showShareDialog by remember { mutableStateOf(false) }
    var saveStatus by remember { mutableStateOf("") }
    val context = LocalContext.current
    val pc = LocalPeacockColors.current
    val hasSelection = contentTfv.selection.length > 0

    // Auto-save with debounce
    LaunchedEffect(content, note) {
        saveStatus = "保存中..."
        delay(600)
        appState.updateSnippet(snippetId, content = content, note = note)
        saveStatus = "已保存"
        delay(1500)
        saveStatus = ""
    }


    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(snippet.title, fontWeight = FontWeight.SemiBold, fontSize = 17.sp)
                },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = pc.background)
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .background(pc.background)
        ) {
            // Toolbar
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                // Save status
                Text(
                    saveStatus,
                    fontSize = 12.sp,
                    color = pc.secondaryText
                )
                Spacer(modifier = Modifier.weight(1f))
                // Copy
                IconButton(
                    onClick = {
                        val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                        clipboard.setPrimaryClip(ClipData.newPlainText("snippet", content))
                        Toast.makeText(context, "已复制", Toast.LENGTH_SHORT).show()
                    },
                    modifier = Modifier.size(36.dp)
                ) {
                    Icon(Icons.Default.ContentCopy, contentDescription = "Copy",
                        modifier = Modifier.size(16.dp), tint = pc.secondaryText)
                }
                // Share
                IconButton(
                    onClick = { showShareDialog = true },
                    modifier = Modifier.size(36.dp)
                ) {
                    Icon(Icons.Default.Share, contentDescription = "Share",
                        modifier = Modifier.size(16.dp), tint = pc.secondaryText)
                }
                // Delete
                IconButton(
                    onClick = {
                        appState.deleteSnippet(snippetId)
                        onBack()
                    },
                    modifier = Modifier.size(36.dp)
                ) {
                    Icon(Icons.Default.Delete, contentDescription = "Delete",
                        modifier = Modifier.size(16.dp), tint = DangerRed)
                }
            }

            HorizontalDivider(color = pc.separator, thickness = 0.5.dp)

            // Content area (scrollable)
            Column(
                modifier = Modifier
                    .weight(1f)
                    .verticalScroll(rememberScrollState())
                    .padding(16.dp)
            ) {
                // Content editor with inline chip rendering (native EditText)
                ChipEditText(
                    content = content,
                    onContentChange = { newText ->
                        contentTfv = TextFieldValue(newText, TextRange(newText.length))
                    },
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(min = 200.dp)
                )

            }

            // Fixed bottom note
            HorizontalDivider(color = pc.separator, thickness = 0.5.dp)
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 10.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    Icons.Default.StickyNote2,
                    contentDescription = null,
                    modifier = Modifier.size(12.dp),
                    tint = pc.tertiaryText
                )
                Spacer(modifier = Modifier.width(8.dp))
                OutlinedTextField(
                    value = note,
                    onValueChange = { note = it },
                    modifier = Modifier.weight(1f),
                    placeholder = { Text("备注", color = pc.tertiaryText, fontSize = 13.sp) },
                    singleLine = true,
                    textStyle = androidx.compose.ui.text.TextStyle(
                        fontSize = 13.sp,
                        color = pc.secondaryText
                    ),
                    colors = OutlinedTextFieldDefaults.colors(
                        unfocusedContainerColor = Color.Transparent,
                        focusedContainerColor = Color.Transparent,
                        unfocusedBorderColor = Color.Transparent,
                        focusedBorderColor = Color.Transparent,
                    )
                )
            }
        }
    }

    // Share dialog
    if (showShareDialog) {
        val onlineDevices = appState.devices.values.filter { it.isOnline }.toList()
        AlertDialog(
            onDismissRequest = { showShareDialog = false },
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
                                        appState.shareSnippet(snippetId, device.deviceId)
                                        showShareDialog = false
                                        Toast.makeText(context, "已分享到 ${device.deviceName}", Toast.LENGTH_SHORT).show()
                                    },
                                shape = RoundedCornerShape(8.dp)
                            ) {
                                Row(
                                    modifier = Modifier.padding(12.dp),
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
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
                TextButton(onClick = { showShareDialog = false }) { Text("取消") }
            }
        )
    }
}

