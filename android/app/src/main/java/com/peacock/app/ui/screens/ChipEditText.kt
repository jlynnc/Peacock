package com.peacock.app.ui.screens

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.RectF
import android.graphics.Typeface
import android.text.*
import android.text.method.LinkMovementMethod
import android.text.style.*
import android.view.ActionMode
import android.view.Gravity
import android.view.Menu
import android.view.MenuItem
import android.widget.EditText
import android.widget.Toast
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView

/**
 * A Compose wrapper around native EditText that renders [[text]] as inline green chips.
 *
 * - Brackets [[ and ]] are hidden (zero-width ReplacementSpan)
 * - Chip content has green rounded background
 * - Tap on chip → copy to clipboard
 * - Text selection → context menu has "标记为快速复制"
 * - Backspace at chip edge → deletes entire [[text]]
 */
@Composable
fun ChipEditText(
    content: String,
    onContentChange: (String) -> Unit,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    val density = LocalDensity.current
    val isDark = isSystemInDarkTheme()
    val textColor = if (isDark) 0xFFFFFFFF.toInt() else 0xFF000000.toInt()
    val hintColor = if (isDark) 0xFF48484A.toInt() else 0xFFC7C7CC.toInt()
    val padPx = with(density) { 14.dp.roundToPx() }
    val padVPx = with(density) { 10.dp.roundToPx() }
    val minHPx = with(density) { 200.dp.roundToPx() }

    // Track whether we're updating from Compose to avoid feedback loops
    val updatingFromCompose = remember { mutableStateOf(false) }

    AndroidView(
        factory = { ctx ->
            EditText(ctx).apply {
                background = null
                setPadding(padPx, padVPx, padPx, padVPx)
                textSize = 14f
                typeface = Typeface.MONOSPACE
                setTextColor(textColor)
                hint = "输入内容..."
                setHintTextColor(hintColor)
                gravity = Gravity.TOP or Gravity.START
                minimumHeight = minHPx
                inputType = InputType.TYPE_CLASS_TEXT or
                    InputType.TYPE_TEXT_FLAG_MULTI_LINE or
                    InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS

                // Enable ClickableSpan handling
                movementMethod = ChipLinkMovementMethod(ctx)

                // Custom selection action: "标记为快速复制"
                customSelectionActionModeCallback = object : ActionMode.Callback {
                    override fun onCreateActionMode(mode: ActionMode, menu: Menu): Boolean {
                        menu.add(0, 1001, 10, "标记为快速复制")
                        return true
                    }
                    override fun onPrepareActionMode(mode: ActionMode, menu: Menu) = false
                    override fun onActionItemClicked(mode: ActionMode, item: MenuItem): Boolean {
                        if (item.itemId == 1001) {
                            val start = selectionStart
                            val end = selectionEnd
                            if (start < end) {
                                val editable = text
                                editable.insert(end, "]]")
                                editable.insert(start, "[[")
                                setSelection(start + end - start + 4)
                            }
                            mode.finish()
                            return true
                        }
                        return false
                    }
                    override fun onDestroyActionMode(mode: ActionMode) {}
                }

                // Text change listener
                addTextChangedListener(object : TextWatcher {
                    override fun beforeTextChanged(s: CharSequence, start: Int, count: Int, after: Int) {}
                    override fun onTextChanged(s: CharSequence, start: Int, before: Int, count: Int) {}
                    override fun afterTextChanged(s: Editable) {
                        if (updatingFromCompose.value) return
                        onContentChange(s.toString())
                        applyChipSpans(s, ctx)
                    }
                })

                // Initial text + spans
                setText(content)
                applyChipSpans(editableText, ctx)
                setSelection(content.length)
            }
        },
        update = { editText ->
            if (editText.text.toString() != content) {
                updatingFromCompose.value = true
                val cursor = editText.selectionStart.coerceAtMost(content.length)
                editText.setText(content)
                editText.setSelection(cursor)
                applyChipSpans(editText.editableText, context)
                updatingFromCompose.value = false
            }
        },
        modifier = modifier
    )
}

/**
 * Custom MovementMethod that handles chip taps (copy to clipboard)
 * and passes other touches to default behavior.
 */
private class ChipLinkMovementMethod(private val context: Context) : LinkMovementMethod() {
    override fun onTouchEvent(
        widget: android.widget.TextView,
        buffer: Spannable,
        event: android.view.MotionEvent
    ): Boolean {
        if (event.action == android.view.MotionEvent.ACTION_UP) {
            val x = event.x.toInt() - widget.totalPaddingLeft + widget.scrollX
            val y = event.y.toInt() - widget.totalPaddingTop + widget.scrollY
            val layout = widget.layout ?: return super.onTouchEvent(widget, buffer, event)
            val line = layout.getLineForVertical(y)
            val off = layout.getOffsetForHorizontal(line, x.toFloat())

            val clickSpans = buffer.getSpans(off, off, ChipClickSpan::class.java)
            if (clickSpans.isNotEmpty()) {
                clickSpans[0].onClick(widget)
                return true
            }
        }
        return super.onTouchEvent(widget, buffer, event)
    }
}

private fun applyChipSpans(editable: Editable, context: Context) {
    // Remove old spans
    editable.getSpans(0, editable.length, BracketHideSpan::class.java).forEach { editable.removeSpan(it) }
    editable.getSpans(0, editable.length, ChipBackgroundSpan::class.java).forEach { editable.removeSpan(it) }
    editable.getSpans(0, editable.length, ChipClickSpan::class.java).forEach { editable.removeSpan(it) }
    editable.getSpans(0, editable.length, ForegroundColorSpan::class.java).forEach {
        if (editable.getSpanFlags(it) and Spannable.SPAN_COMPOSING == 0) {
            // Only remove our chip foreground spans (tag with specific color)
            if (it.foregroundColor == 0xFF056440.toInt()) {
                editable.removeSpan(it)
            }
        }
    }

    val text = editable.toString()
    val regex = Regex("""\[\[(.+?)\]\]""")

    for (match in regex.findAll(text)) {
        val fullStart = match.range.first
        val fullEnd = match.range.last + 1
        val chipText = match.groupValues[1]

        // Hide [[ (zero-width, invisible)
        editable.setSpan(
            BracketHideSpan(),
            fullStart, fullStart + 2,
            Spannable.SPAN_EXCLUSIVE_EXCLUSIVE
        )
        // Hide ]]
        editable.setSpan(
            BracketHideSpan(),
            fullEnd - 2, fullEnd,
            Spannable.SPAN_EXCLUSIVE_EXCLUSIVE
        )

        // Chip background on the content (between [[ and ]])
        editable.setSpan(
            ChipBackgroundSpan(),
            fullStart + 2, fullEnd - 2,
            Spannable.SPAN_EXCLUSIVE_EXCLUSIVE
        )
        // Chip text color
        editable.setSpan(
            ForegroundColorSpan(0xFF056440.toInt()),
            fullStart + 2, fullEnd - 2,
            Spannable.SPAN_EXCLUSIVE_EXCLUSIVE
        )
        // Click handler
        editable.setSpan(
            ChipClickSpan(chipText, context),
            fullStart, fullEnd,
            Spannable.SPAN_EXCLUSIVE_EXCLUSIVE
        )
    }
}

/**
 * Makes [[ and ]] invisible: zero width, draws nothing.
 */
private class BracketHideSpan : ReplacementSpan() {
    override fun getSize(paint: Paint, text: CharSequence, start: Int, end: Int, fm: Paint.FontMetricsInt?): Int {
        // Maintain font metrics so line height doesn't collapse
        if (fm != null) {
            paint.getFontMetricsInt(fm)
        }
        return 0
    }

    override fun draw(canvas: Canvas, text: CharSequence, start: Int, end: Int,
                      x: Float, top: Int, y: Int, bottom: Int, paint: Paint) {
        // Draw nothing
    }
}

/**
 * Draws a rounded green background behind chip text.
 */
private class ChipBackgroundSpan : ReplacementSpan() {
    private val bgColor = 0x260D9488 // 15% teal
    private val borderColor = 0x4D0D9488 // 30% teal
    private val textColor = 0xFF056440.toInt()
    private val cornerRadius = 8f
    private val padH = 12f
    private val padV = 4f

    override fun getSize(paint: Paint, text: CharSequence, start: Int, end: Int, fm: Paint.FontMetricsInt?): Int {
        val textWidth = paint.measureText(text, start, end)
        if (fm != null) {
            paint.getFontMetricsInt(fm)
            fm.top -= padV.toInt()
            fm.bottom += padV.toInt()
            fm.ascent -= padV.toInt()
            fm.descent += padV.toInt()
        }
        return (textWidth + padH * 2).toInt()
    }

    override fun draw(canvas: Canvas, text: CharSequence, start: Int, end: Int,
                      x: Float, top: Int, y: Int, bottom: Int, paint: Paint) {
        val textWidth = paint.measureText(text, start, end)
        val rect = RectF(x, y + paint.ascent() - padV, x + textWidth + padH * 2, y + paint.descent() + padV)

        // Background
        val bgPaint = Paint(paint).apply { color = bgColor; style = Paint.Style.FILL }
        canvas.drawRoundRect(rect, cornerRadius, cornerRadius, bgPaint)

        // Border
        val borderPaint = Paint().apply {
            color = borderColor; style = Paint.Style.STROKE; strokeWidth = 1.5f; isAntiAlias = true
        }
        canvas.drawRoundRect(rect, cornerRadius, cornerRadius, borderPaint)

        // Text
        paint.color = textColor
        canvas.drawText(text, start, end, x + padH, y.toFloat(), paint)
    }
}

/**
 * Handles tap on a chip to copy its text.
 */
private class ChipClickSpan(
    private val chipText: String,
    private val context: Context
) : ClickableSpan() {
    override fun onClick(widget: android.view.View) {
        val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        clipboard.setPrimaryClip(ClipData.newPlainText("chip", chipText))
        Toast.makeText(context, "已复制: ${chipText.take(20)}", Toast.LENGTH_SHORT).show()
    }
    override fun updateDrawState(ds: TextPaint) {
        // Don't add underline or change color (handled by other spans)
    }
}
