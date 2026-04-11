package com.peacock.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import com.peacock.app.models.AppState
import com.peacock.app.ui.theme.PeacockTheme
import com.peacock.app.ui.screens.*

class MainActivity : ComponentActivity() {
    private lateinit var appState: AppState

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        appState = AppState(applicationContext)
        appState.init()

        setContent {
            PeacockTheme {
                MainScreen(appState)
            }
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        appState.stop()
    }
}

@Composable
fun MainScreen(appState: AppState) {
    var selectedTab by remember { mutableIntStateOf(0) }
    var chatDeviceId by remember { mutableStateOf<String?>(null) }
    var editSnippetId by remember { mutableStateOf<String?>(null) }

    if (chatDeviceId != null) {
        ChatScreen(
            appState = appState,
            deviceId = chatDeviceId!!,
            onBack = { chatDeviceId = null }
        )
        return
    }

    if (editSnippetId != null) {
        SnippetEditorScreen(
            appState = appState,
            snippetId = editSnippetId!!,
            onBack = { editSnippetId = null }
        )
        return
    }

    Scaffold(
        bottomBar = {
            NavigationBar {
                NavigationBarItem(
                    selected = selectedTab == 0,
                    onClick = { selectedTab = 0 },
                    icon = { Icon(Icons.Default.Wifi, contentDescription = null) },
                    label = { Text("设备") }
                )
                NavigationBarItem(
                    selected = selectedTab == 1,
                    onClick = { selectedTab = 1 },
                    icon = { Icon(Icons.Default.Description, contentDescription = null) },
                    label = { Text("片段") }
                )
                NavigationBarItem(
                    selected = selectedTab == 2,
                    onClick = { selectedTab = 2 },
                    icon = { Icon(Icons.Default.Settings, contentDescription = null) },
                    label = { Text("设置") }
                )
            }
        }
    ) { padding ->
        Box(modifier = Modifier.padding(padding)) {
            when (selectedTab) {
                0 -> DeviceListScreen(appState) { deviceId -> chatDeviceId = deviceId }
                1 -> SnippetListScreen(appState) { snippetId -> editSnippetId = snippetId }
                2 -> SettingsScreen(appState)
            }
        }
    }
}
