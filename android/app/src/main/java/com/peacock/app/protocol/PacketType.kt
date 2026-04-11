package com.peacock.app.protocol

enum class PacketType(val value: UShort) {
    Announce(1u),
    Bye(2u),
    AnnounceResponse(3u),
    Text(10u),
    FileOffer(20u),
    FileAccept(21u),
    FileReject(22u),
    FileChunk(23u),
    Clipboard(30u),
    SnippetShare(31u),
    Ack(99u);

    companion object {
        fun fromValue(v: UShort): PacketType? = entries.find { it.value == v }
    }
}
