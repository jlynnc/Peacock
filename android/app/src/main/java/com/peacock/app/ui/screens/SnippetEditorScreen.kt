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
import androidx.compose.foundation.text.BasicTextField
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
import androidx.compose.ui.text.*
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.peacock.app.models.AppState
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SnippetEditorScreen(appState: AppState, snippetId: String, onBack: () -> Unit) {
    val snippet = appState.snippets.find { it.id == snippetId }
    if (snippet == null) {
        onBack()
        return
    }

    var content by remember { mutableStateOf(snippet.content) }
    var note by remember { mutableStateOf(snippet.note) }
    val context = LocalContext.current
    val scope = rememberCoroutineScope()

    // Auto-save with debounce
    LaunchedEffect(content, note) {
        delay(600)
        appState.updateSnippet(snippetId, content = content, note = note)
    }

    // Parse [[...]] chips
    val chipRegex = remember { Regex("""\[\[(.+?)\]\]""") }
    val chips = remember(content) { chipRegex.findAll(content).map { it.groupValues[1] }.toList() }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(snippet.title, fontWeight = FontWeight.SemiBold) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
                actions = {
                    IconButton(onClick = {
                        val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                        clipboard.setPrimaryClip(ClipData.newPlainText("snippet", content))
                        Toast.makeText(context, "已复制", Toast.LENGTH_SHORT).show()
                    }) {
                        Icon(Icons.Default.ContentCopy, contentDescription = "Copy")
                    }
                    IconButton(onClick = {
                        appState.deleteSnippet(snippetId)
                        onBack()
                    }) {
                        Icon(Icons.Default.Delete, contentDescription = "Delete", tint = Color.Red)
                    }
                }
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(16.dp)
        ) {
            // Content editor
            OutlinedTextField(
                value = content,
                onValueChange = { content = it },
                modifier = Modifier.fillMaxWidth().heightIn(min = 200.dp),
                placeholder = { Text("输入内容...") },
                shape = RoundedCornerShape(8.dp)
            )

            // Quick copy chips
            if (chips.isNotEmpty()) {
                Spacer(modifier = Modifier.height(16.dp))
                Text("快速复制", fontSize = 13.sp, color = Color.Gray)
                Spacer(modifier = Modifier.height(8.dp))
                FlowRow(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    chips.forEach { chipText ->
                        Surface(
                            shape = RoundedCornerShape(6.dp),
                            color = Color(0xFF0D9488).copy(alpha = 0.1f),
                            modifier = Modifier.clickable {
                                val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                                clipboard.setPrimaryClip(ClipData.newPlainText("chip", chipText))
                                Toast.makeText(context, "已复制: ${chipText.take(20)}", Toast.LENGTH_SHORT).show()
                            }
                        ) {
                            Text(
                                chipText,
                                modifier = Modifier.padding(horizontal = 10.dp, vertical = 6.dp),
                                color = Color(0xFF0D9488),
                                fontSize = 14.sp
                            )
                        }
                    }
                }
            }

            // Note
            Spacer(modifier = Modifier.height(16.dp))
            OutlinedTextField(
                value = note,
                onValueChange = { note = it },
                modifier = Modifier.fillMaxWidth(),
                placeholder = { Text("备注（可选）") },
                shape = RoundedCornerShape(8.dp),
                minLines = 2
            )
        }
    }
}

@Composable
fun FlowRow(
    horizontalArrangement: Arrangement.Horizontal = Arrangement.Start,
    verticalArrangement: Arrangement.Vertical = Arrangement.Top,
    content: @Composable () -> Unit
) {
    // Simple flow layout using Row wrapping
    androidx.compose.foundation.layout.FlowRow(
        horizontalArrangement = horizontalArrangement,
        verticalArrangement = verticalArrangement
    ) {
        content()
    }
}
