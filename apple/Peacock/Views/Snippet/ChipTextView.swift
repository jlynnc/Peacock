import UIKit
import SwiftUI

/// Custom attribute to mark chip ranges. Value is the chip text.
private let chipAttrKey = NSAttributedString.Key("peacock.chip")

// MARK: - ChipTextView

/// A UITextView that renders [[text]] as inline green chips.
/// - Display: [[]] are hidden, chip text shown with green background
/// - Storage: raw text with [[]] preserved
/// - Tap chip: copy to clipboard + flash
/// - Long press chip: context menu (copy / unmark)
/// - Select text → system menu "标记为快速复制"
class ChipTextView: UITextView {

    var onRawTextChange: ((String) -> Void)?
    var onSelectionChange: ((NSRange) -> Void)?

    /// The raw text with [[]] markers — source of truth
    private var rawText: String = ""
    /// Mapping: for each character in display text, the index in rawText
    private var displayToRawMap: [Int] = []
    /// Chip ranges in display coordinates
    private var chipDisplayRanges: [(range: NSRange, text: String)] = []

    private var isInternalUpdate = false
    private var longPressedChipText: String?

    // MARK: - Setup

    override init(frame: CGRect, textContainer: NSTextContainer?) {
        super.init(frame: frame, textContainer: textContainer)
        setup()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        setup()
    }

    private func setup() {
        font = UIFont.monospacedSystemFont(ofSize: 14, weight: .regular)
        textColor = UIColor.label
        backgroundColor = .clear
        textContainerInset = UIEdgeInsets(top: 4, left: 0, bottom: 4, right: 0)
        isScrollEnabled = false
        delegate = self

        // Tap for chip copy
        let tap = UITapGestureRecognizer(target: self, action: #selector(handleTap(_:)))
        tap.delegate = self
        addGestureRecognizer(tap)

        // Long press for chip context menu
        let longPress = UILongPressGestureRecognizer(target: self, action: #selector(handleLongPress(_:)))
        longPress.delegate = self
        addGestureRecognizer(longPress)
    }

    override var canBecomeFirstResponder: Bool { true }

    override func canPerformAction(_ action: Selector, withSender sender: Any?) -> Bool {
        if action == #selector(contextCopyChip) || action == #selector(contextUnmarkChip) {
            return longPressedChipText != nil
        }
        if action == #selector(markAsQuickCopy) {
            return selectedRange.length > 0 && longPressedChipText == nil
        }
        return super.canPerformAction(action, withSender: sender)
    }

    @objc func markAsQuickCopy() {
        guard selectedRange.length > 0 else { return }
        let displayNS = (attributedText.string as NSString)
        guard selectedRange.location + selectedRange.length <= displayNS.length else { return }
        let selectedText = displayNS.substring(with: selectedRange)
        guard !selectedText.isEmpty else { return }

        // Check not already inside a chip
        for chip in chipDisplayRanges {
            if NSIntersectionRange(selectedRange, chip.range).length > 0 {
                return // Don't mark text that's already in a chip
            }
        }

        // Map display selection back to raw positions
        guard selectedRange.location < displayToRawMap.count,
              selectedRange.location + selectedRange.length - 1 < displayToRawMap.count else { return }
        let rawStart = displayToRawMap[selectedRange.location]
        let rawEnd = displayToRawMap[selectedRange.location + selectedRange.length - 1] + 1

        // Insert [[]] in raw text
        var newRaw = rawText
        let endIdx = newRaw.index(newRaw.startIndex, offsetBy: rawEnd)
        newRaw.insert(contentsOf: "]]", at: endIdx)
        let startIdx = newRaw.index(newRaw.startIndex, offsetBy: rawStart)
        newRaw.insert(contentsOf: "[[", at: startIdx)

        rawText = newRaw
        onRawTextChange?(rawText)
        rebuildDisplay()
    }

    // MARK: - Public: set/get raw text

    func setRawText(_ raw: String) {
        guard !isInternalUpdate else { return }
        rawText = raw
        rebuildDisplay()
    }

    func getRawText() -> String {
        return rawText
    }

    // MARK: - Display Building

    private func rebuildDisplay() {
        isInternalUpdate = true
        let savedOffset = selectedRange.location

        let (attrStr, map, chips) = buildDisplayString(from: rawText)
        displayToRawMap = map
        chipDisplayRanges = chips

        attributedText = attrStr

        // Restore cursor — map raw offset to display offset
        let displayLen = attrStr.length
        let safeLoc = min(savedOffset, displayLen)
        selectedRange = NSRange(location: safeLoc, length: 0)

        isInternalUpdate = false
    }

    private func buildDisplayString(from raw: String) -> (NSAttributedString, [Int], [(range: NSRange, text: String)]) {
        let result = NSMutableAttributedString()
        var map: [Int] = [] // displayIndex -> rawIndex
        var chips: [(range: NSRange, text: String)] = []

        let defaultAttrs: [NSAttributedString.Key: Any] = [
            .font: UIFont.monospacedSystemFont(ofSize: 14, weight: .regular),
            .foregroundColor: UIColor.label
        ]

        let chipTextAttrs: [NSAttributedString.Key: Any] = [
            .font: UIFont.monospacedSystemFont(ofSize: 13, weight: .medium),
            .foregroundColor: UIColor(red: 5/255, green: 100/255, blue: 80/255, alpha: 1),
            .backgroundColor: UIColor(red: 13/255, green: 148/255, blue: 136/255, alpha: 0.15),
        ]

        let pattern = /\[\[(.+?)\]\]/
        var lastEnd = raw.startIndex
        var displayOffset = 0

        for match in raw.matches(of: pattern) {
            // Plain text before chip
            let before = String(raw[lastEnd..<match.range.lowerBound])
            if !before.isEmpty {
                result.append(NSAttributedString(string: before, attributes: defaultAttrs))
                let rawOffset = raw.distance(from: raw.startIndex, to: lastEnd)
                for i in 0..<before.utf16.count {
                    map.append(rawOffset + i)
                }
                displayOffset += before.utf16.count
            }

            // Chip — display only the inner text, no [[]]
            let chipText = String(match.1)
            let chipStart = displayOffset

            // Add space before chip for visual separation
            let chipStr = NSMutableAttributedString(string: " ", attributes: defaultAttrs)
            // Map the space to the [[ position
            let rawMatchStart = raw.distance(from: raw.startIndex, to: match.range.lowerBound)
            map.append(rawMatchStart)
            displayOffset += 1

            // Chip text with styling + custom attribute
            var chipAttrs = chipTextAttrs
            chipAttrs[chipAttrKey] = chipText
            chipStr.append(NSAttributedString(string: chipText, attributes: chipAttrs))
            // Map chip display chars to raw chars (offset by 2 for [[)
            let rawChipStart = rawMatchStart + 2 // skip [[
            for i in 0..<chipText.utf16.count {
                map.append(rawChipStart + i)
            }
            displayOffset += chipText.utf16.count

            // Space after
            chipStr.append(NSAttributedString(string: " ", attributes: defaultAttrs))
            let rawMatchEnd = raw.distance(from: raw.startIndex, to: match.range.upperBound)
            map.append(rawMatchEnd - 1) // map to ]] position
            displayOffset += 1

            result.append(chipStr)

            let chipRange = NSRange(location: chipStart + 1, length: chipText.utf16.count)
            chips.append((range: chipRange, text: chipText))

            lastEnd = match.range.upperBound
        }

        // Remaining plain text
        let remaining = String(raw[lastEnd...])
        if !remaining.isEmpty {
            result.append(NSAttributedString(string: remaining, attributes: defaultAttrs))
            let rawOffset = raw.distance(from: raw.startIndex, to: lastEnd)
            for i in 0..<remaining.utf16.count {
                map.append(rawOffset + i)
            }
        }

        return (result, map, chips)
    }

    // MARK: - Tap → Copy chip

    @objc private func handleTap(_ gesture: UITapGestureRecognizer) {
        guard gesture.state == .ended else { return }
        let point = gesture.location(in: self)
        guard let chipText = chipTextAt(point) else { return }

        UIPasteboard.general.string = chipText
        flashChip(at: point)
    }

    // MARK: - Long Press → Context Menu

    @objc private func handleLongPress(_ gesture: UILongPressGestureRecognizer) {
        guard gesture.state == .began else { return }
        let point = gesture.location(in: self)
        guard let chipText = chipTextAt(point) else { return }

        longPressedChipText = chipText

        let menu = UIMenuController.shared
        let copyItem = UIMenuItem(title: "复制", action: #selector(contextCopyChip))
        let unmarkItem = UIMenuItem(title: "取消标记", action: #selector(contextUnmarkChip))
        menu.menuItems = [copyItem, unmarkItem]

        // Position menu near the tap
        let rect = CGRect(x: point.x - 20, y: point.y - 10, width: 40, height: 20)
        menu.showMenu(from: self, rect: rect)
    }

    @objc private func contextCopyChip() {
        guard let text = longPressedChipText else { return }
        UIPasteboard.general.string = text
        longPressedChipText = nil
    }

    @objc private func contextUnmarkChip() {
        guard let chipText = longPressedChipText else { return }
        longPressedChipText = nil

        // Remove [[ and ]] from raw text
        let marker = "[[\(chipText)]]"
        if let range = rawText.range(of: marker) {
            rawText.replaceSubrange(range, with: chipText)
            onRawTextChange?(rawText)
            rebuildDisplay()
        }
    }

    // MARK: - Helpers

    private func chipTextAt(_ point: CGPoint) -> String? {
        guard let position = closestPosition(to: point) else { return nil }
        let offset = self.offset(from: beginningOfDocument, to: position)
        guard let attrText = attributedText, offset < attrText.length else { return nil }
        return attrText.attribute(chipAttrKey, at: offset, effectiveRange: nil) as? String
    }

    private func flashChip(at point: CGPoint) {
        guard let position = closestPosition(to: point) else { return }
        let offset = self.offset(from: beginningOfDocument, to: position)
        guard let attrText = attributedText, offset < attrText.length else { return }

        var chipRange = NSRange()
        guard attrText.attribute(chipAttrKey, at: offset, longestEffectiveRange: &chipRange,
                                  in: NSRange(location: 0, length: attrText.length)) != nil else { return }

        let flash = UIColor(red: 13/255, green: 148/255, blue: 136/255, alpha: 0.4)
        let normal = UIColor(red: 13/255, green: 148/255, blue: 136/255, alpha: 0.15)

        textStorage.addAttribute(.backgroundColor, value: flash, range: chipRange)
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) { [weak self] in
            guard let self, chipRange.location + chipRange.length <= self.textStorage.length else { return }
            self.textStorage.addAttribute(.backgroundColor, value: normal, range: chipRange)
        }
    }
}

// MARK: - UITextViewDelegate

extension ChipTextView: UITextViewDelegate {

    func textViewDidChange(_ textView: UITextView) {
        guard !isInternalUpdate else { return }

        // Convert display text back to raw text by examining attributes
        rawText = extractRawText()
        onRawTextChange?(rawText)

        // Re-render
        rebuildDisplay()
    }

    func textViewDidChangeSelection(_ textView: UITextView) {
        guard !isInternalUpdate else { return }
        onSelectionChange?(textView.selectedRange)

        // Update menu items based on selection
        if selectedRange.length > 0 {
            let markItem = UIMenuItem(title: "标记为快速复制", action: #selector(markAsQuickCopy))
            UIMenuController.shared.menuItems = [markItem]
        } else {
            UIMenuController.shared.menuItems = nil
        }
    }

    func textView(_ textView: UITextView, shouldChangeTextIn range: NSRange, replacementText text: String) -> Bool {
        // If deleting and range overlaps a chip, delete the entire chip
        if text.isEmpty && range.length > 0 {
            for chip in chipDisplayRanges {
                // Include the surrounding spaces in the chip "delete zone"
                let chipZone = NSRange(location: max(0, chip.range.location - 1),
                                       length: chip.range.length + 2)
                if NSIntersectionRange(range, chipZone).length > 0 {
                    // Delete entire chip from raw text
                    let marker = "[[\(chip.text)]]"
                    if let markerRange = rawText.range(of: marker) {
                        rawText.replaceSubrange(markerRange, with: "")
                        onRawTextChange?(rawText)
                        rebuildDisplay()
                        return false
                    }
                }
            }
        }

        // If typing inside a chip, move cursor to after the chip
        if !text.isEmpty {
            for chip in chipDisplayRanges {
                let chipZone = NSRange(location: chip.range.location,
                                       length: chip.range.length)
                if range.location > chipZone.location && range.location < chipZone.location + chipZone.length {
                    selectedRange = NSRange(location: chipZone.location + chipZone.length + 1, length: 0)
                    return true
                }
            }
        }

        return true
    }

    /// Reconstruct raw text from current display + attributes
    private func extractRawText() -> String {
        guard let attrText = attributedText else { return "" }
        var result = ""
        attrText.enumerateAttributes(in: NSRange(location: 0, length: attrText.length)) { attrs, range, _ in
            let segment = (attrText.string as NSString).substring(with: range)
            if let chipText = attrs[chipAttrKey] as? String {
                result += "[[\(chipText)]]"
            } else {
                // Skip the padding spaces around chips (single spaces adjacent to chips)
                result += segment
            }
        }
        // Clean up extra spaces that were padding around chips
        // The spaces we added for visual separation need to be removed from raw
        // Actually, just strip double spaces that result from chip padding
        return result
    }
}

// MARK: - UIGestureRecognizerDelegate

extension ChipTextView: UIGestureRecognizerDelegate {
    func gestureRecognizer(_ gestureRecognizer: UIGestureRecognizer, shouldRecognizeSimultaneouslyWith otherGestureRecognizer: UIGestureRecognizer) -> Bool {
        return true
    }

    override func gestureRecognizerShouldBegin(_ gestureRecognizer: UIGestureRecognizer) -> Bool {
        // Only intercept our custom gestures when on a chip
        if gestureRecognizer is UITapGestureRecognizer || gestureRecognizer is UILongPressGestureRecognizer {
            if gestureRecognizer.view === self {
                let point = gestureRecognizer.location(in: self)
                if chipTextAt(point) != nil {
                    return true
                }
                // Not on a chip — let default text view behavior handle it
            }
        }
        return true
    }
}

// MARK: - SwiftUI Wrapper

struct ChipTextViewRepresentable: UIViewRepresentable {
    @Binding var text: String
    @Binding var selectedRange: NSRange
    var onTextChange: () -> Void

    func makeUIView(context: Context) -> ChipTextView {
        let view = ChipTextView()
        view.onRawTextChange = { newRaw in
            DispatchQueue.main.async {
                self.text = newRaw
                self.onTextChange()
            }
        }
        view.onSelectionChange = { range in
            DispatchQueue.main.async {
                self.selectedRange = range
            }
        }
        view.setRawText(text)
        return view
    }

    func updateUIView(_ uiView: ChipTextView, context: Context) {
        if uiView.getRawText() != text {
            uiView.setRawText(text)
        }
    }
}
