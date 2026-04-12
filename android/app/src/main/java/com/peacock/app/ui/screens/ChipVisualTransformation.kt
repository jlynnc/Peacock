package com.peacock.app.ui.screens

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.sp
import com.peacock.app.ui.theme.PrimaryTeal

/**
 * VisualTransformation that renders [[text]] as inline green chips:
 * - Hides [[ and ]] brackets
 * - Adds green background + rounded style to chip content
 * - Maintains proper cursor offset mapping
 */
class ChipVisualTransformation : VisualTransformation {

    companion object {
        private val chipBg = Color(0x260D9488) // 15% opacity teal
        private val chipText = Color(0xFF056440)
        private val chipBorder = Color(0x4D0D9488) // 30% opacity
    }

    override fun filter(text: AnnotatedString): TransformedText {
        val original = text.text
        if (!original.contains("[[")) {
            return TransformedText(text, OffsetMapping.Identity)
        }

        // Parse [[...]] regions in original text
        val regions = mutableListOf<ChipRegion>() // regions in ORIGINAL text
        val regex = Regex("""\[\[(.+?)\]\]""")
        for (match in regex.findResults(original)) {
            regions.add(ChipRegion(
                originalStart = match.range.first,       // position of first [
                originalEnd = match.range.last + 1,      // position after last ]
                contentStart = match.range.first + 2,    // position of chip text start
                contentEnd = match.range.last - 1,       // position after chip text end
                content = match.groupValues[1]
            ))
        }

        if (regions.isEmpty()) {
            return TransformedText(text, OffsetMapping.Identity)
        }

        // Build transformed string (without [[ and ]])
        // And track offset mapping: for each original offset, what's the transformed offset
        val originalToTransformed = IntArray(original.length + 1)
        val builder = StringBuilder()
        val chipSpans = mutableListOf<Pair<Int, Int>>() // start..end in transformed text

        var origIdx = 0
        var transIdx = 0
        var regionIdx = 0

        while (origIdx < original.length) {
            if (regionIdx < regions.size && origIdx == regions[regionIdx].originalStart) {
                val region = regions[regionIdx]

                // Skip [[ (2 chars)
                originalToTransformed[origIdx] = transIdx
                originalToTransformed[origIdx + 1] = transIdx
                origIdx += 2

                // Chip content
                val chipStart = transIdx
                while (origIdx < region.contentEnd) {
                    originalToTransformed[origIdx] = transIdx
                    builder.append(original[origIdx])
                    origIdx++
                    transIdx++
                }
                val chipEnd = transIdx
                chipSpans.add(chipStart to chipEnd)

                // Skip ]] (2 chars)
                originalToTransformed[origIdx] = transIdx
                if (origIdx + 1 < original.length + 1) originalToTransformed[origIdx + 1] = transIdx
                origIdx += 2

                regionIdx++
            } else {
                originalToTransformed[origIdx] = transIdx
                builder.append(original[origIdx])
                origIdx++
                transIdx++
            }
        }
        originalToTransformed[original.length] = transIdx

        // Build reverse mapping: for each transformed offset, what's the original offset
        val transformedToOriginal = IntArray(transIdx + 1)
        var lastOriginal = 0
        for (oi in 0..original.length) {
            val ti = originalToTransformed[oi]
            if (ti < transformedToOriginal.size) {
                transformedToOriginal[ti] = oi
                lastOriginal = oi
            }
        }
        // Fill gaps
        for (ti in transformedToOriginal.indices) {
            if (ti > 0 && transformedToOriginal[ti] == 0 && ti != 0) {
                transformedToOriginal[ti] = transformedToOriginal[ti - 1]
            }
        }
        // Fix the last entry
        if (transIdx < transformedToOriginal.size) {
            transformedToOriginal[transIdx] = original.length
        }

        // Build AnnotatedString with chip styling
        val annotated = buildAnnotatedString {
            append(builder.toString())
            for ((start, end) in chipSpans) {
                addStyle(
                    SpanStyle(
                        color = chipText,
                        background = chipBg,
                        fontWeight = FontWeight.Medium,
                        fontSize = 13.sp
                    ),
                    start, end
                )
            }
        }

        val mapping = object : OffsetMapping {
            override fun originalToTransformed(offset: Int): Int {
                return originalToTransformed[offset.coerceIn(0, original.length)]
            }
            override fun transformedToOriginal(offset: Int): Int {
                return transformedToOriginal[offset.coerceIn(0, transIdx)]
            }
        }

        return TransformedText(annotated, mapping)
    }

    private fun Regex.findResults(input: String): List<MatchResult> {
        return findAll(input).toList()
    }

    private data class ChipRegion(
        val originalStart: Int,
        val originalEnd: Int,
        val contentStart: Int,
        val contentEnd: Int,
        val content: String
    )
}
