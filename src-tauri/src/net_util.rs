/// iOS network utilities — bind sockets to Wi-Fi interface (en0)
///
/// On iOS, sockets may route through the wrong interface (cellular, VPN, etc.)
/// causing "No route to host" for LAN connections. Using IP_BOUND_IF forces
/// the socket through the Wi-Fi interface.

/// Bind a raw file descriptor to the en0 (Wi-Fi) interface on iOS.
/// No-op on other platforms.
#[cfg(target_os = "ios")]
pub fn bind_to_wifi(fd: std::os::unix::io::RawFd) -> std::io::Result<()> {
    let ifname = std::ffi::CString::new("en0").unwrap();
    let ifindex = unsafe { libc::if_nametoindex(ifname.as_ptr()) };
    if ifindex == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "en0 interface not found",
        ));
    }
    let ret = unsafe {
        libc::setsockopt(
            fd,
            libc::IPPROTO_IP,
            25, // IP_BOUND_IF
            &ifindex as *const u32 as *const libc::c_void,
            std::mem::size_of::<u32>() as u32,
        )
    };
    if ret != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(not(target_os = "ios"))]
pub fn bind_to_wifi(_fd: i32) -> std::io::Result<()> {
    Ok(())
}

/// Helper: bind a socket2::Socket to Wi-Fi on iOS
pub fn bind_socket_to_wifi(socket: &socket2::Socket) -> std::io::Result<()> {
    #[cfg(target_os = "ios")]
    {
        use std::os::unix::io::AsRawFd;
        bind_to_wifi(socket.as_raw_fd())
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = socket;
        Ok(())
    }
}
