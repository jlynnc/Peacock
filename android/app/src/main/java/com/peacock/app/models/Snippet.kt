package com.peacock.app.models

data class Snippet(
    val id: String,
    var title: String = "新建片段",
    var content: String = "",
    var tag: String = "",
    var note: String = "",
    var sortOrder: Int = 0,
    val createdAt: Long = System.currentTimeMillis() / 1000,
    var updatedAt: Long = System.currentTimeMillis() / 1000
)
