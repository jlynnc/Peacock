package com.peacock.app

import android.content.Context
import android.net.wifi.WifiManager
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
    private var multicastLock: WifiManager.MulticastLock? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        // Acquire multicast lock so Android doesn't filter out UDP multicast packets
        val wifiManager = applicationContext.getSystemService(Context.WIFI_SERVICE) as WifiManager
        multicastLock = wifiManager.createMulticastLock("peacock_multicast").apply {
            setReferenceCounted(true)
            acquire()
        }

        appState = AppState(applicationContext)
        appState.init()

        setContent {
            PeacockTheme {
                MainScreen(appState)
            }
        }
    }

    override fun onResume() {
        super.onResume()
        // Re-acquire multicast lock if released
        if (multicastLock?.isHeld == false) {
            multicastLock?.acquire()
        }
    }

    override fun onPause() {
        super.onPause()
        // Don't stop UDP service — it needs to keep running to receive FileAccept etc.
    }

    override fun onDestroy() {
        super.onDestroy()
        appState.stop()
        multicastLock?.release()
    }
}

@Composable
fun MainScreen(appState: AppState) {
    var selectedTab by remember { mutableIntStateOf(0) }
    var chatDeviceId by remember { mutableStateOf<String?>(null) }
    var editSnippetId by remember { mutableStateOf<String?>(null) }

    // Show pending file offer dialog
    val pendingOffer = appState.pendingOffers.firstOrNull()
    if (pendingOffer != null) {
        FileOfferDialog(
            task = pendingOffer,
            senderName = appState.devices[pendingOffer.deviceId]?.deviceName ?: "未知设备",
            onAccept = { appState.acceptFileOffer(pendingOffer.transferId) },
            onReject = { appState.rejectFileOffer(pendingOffer.transferId) }
        )
    }

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
