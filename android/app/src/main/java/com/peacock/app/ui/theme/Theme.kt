package com.peacock.app.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color

// Peacock-specific colors that don't map to Material3 slots
data class PeacockColors(
    val background: Color,
    val secondaryBg: Color,
    val tertiaryFill: Color,
    val primaryText: Color,
    val secondaryText: Color,
    val tertiaryText: Color,
    val separator: Color,
    val sentBubbleBg: Color,
    val sentBubbleBorder: Color,
    val sentBubbleText: Color,
    val receivedBubbleBg: Color,
    val receivedBubbleBorder: Color,
)

val LocalPeacockColors = staticCompositionLocalOf {
    PeacockColors(
        background = LightBackground,
        secondaryBg = LightSecondaryBg,
        tertiaryFill = LightTertiaryFill,
        primaryText = LightPrimaryText,
        secondaryText = LightSecondaryText,
        tertiaryText = LightTertiaryText,
        separator = LightSeparator,
        sentBubbleBg = SentBubbleBg,
        sentBubbleBorder = SentBubbleBorder,
        sentBubbleText = SentBubbleText,
        receivedBubbleBg = LightTertiaryFill,
        receivedBubbleBorder = Color(0x4DE5E5EA),
    )
}

private val LightPeacockColors = PeacockColors(
    background = LightBackground,
    secondaryBg = LightSecondaryBg,
    tertiaryFill = LightTertiaryFill,
    primaryText = LightPrimaryText,
    secondaryText = LightSecondaryText,
    tertiaryText = LightTertiaryText,
    separator = LightSeparator,
    sentBubbleBg = SentBubbleBg,
    sentBubbleBorder = SentBubbleBorder,
    sentBubbleText = SentBubbleText,
    receivedBubbleBg = LightTertiaryFill,
    receivedBubbleBorder = Color(0x4DE5E5EA),
)

private val DarkPeacockColors = PeacockColors(
    background = DarkBackground,
    secondaryBg = DarkSecondaryBg,
    tertiaryFill = DarkTertiaryFill,
    primaryText = DarkPrimaryText,
    secondaryText = DarkSecondaryText,
    tertiaryText = DarkTertiaryText,
    separator = DarkSeparator,
    sentBubbleBg = DarkSentBubbleBg,
    sentBubbleBorder = Color(0x330D9488),
    sentBubbleText = DarkSentBubbleText,
    receivedBubbleBg = DarkTertiaryFill,
    receivedBubbleBorder = Color(0x4D38383A),
)

private val LightColorScheme = lightColorScheme(
    primary = PrimaryTeal,
    onPrimary = Color.White,
    secondary = PrimaryDark,
    background = LightBackground,
    surface = LightBackground,
    surfaceVariant = LightSecondaryBg,
    onBackground = LightPrimaryText,
    onSurface = LightPrimaryText,
    onSurfaceVariant = LightSecondaryText,
    outline = LightSeparator,
)

private val DarkColorScheme = darkColorScheme(
    primary = PrimaryTeal,
    onPrimary = Color.White,
    secondary = PrimaryDark,
    background = DarkBackground,
    surface = DarkBackground,
    surfaceVariant = DarkSecondaryBg,
    onBackground = DarkPrimaryText,
    onSurface = DarkPrimaryText,
    onSurfaceVariant = DarkSecondaryText,
    outline = DarkSeparator,
)

@Composable
fun PeacockTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit
) {
    val colorScheme = if (darkTheme) DarkColorScheme else LightColorScheme
    val peacockColors = if (darkTheme) DarkPeacockColors else LightPeacockColors

    CompositionLocalProvider(LocalPeacockColors provides peacockColors) {
        MaterialTheme(
            colorScheme = colorScheme,
            typography = Typography,
            content = content
        )
    }
}
