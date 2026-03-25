/**
 * Detect if running inside Tauri
 */
export function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

// Cache the platform detection result
let _detectedPlatform: string | null = null;
let _isMobileResult: boolean | null = null;

/**
 * Async platform detection using Tauri OS plugin (accurate on all platforms)
 */
export async function detectPlatform(): Promise<string> {
  if (_detectedPlatform) return _detectedPlatform;

  if (!isTauri()) {
    _detectedPlatform = "web";
    return _detectedPlatform;
  }

  try {
    const { type } = await import("@tauri-apps/plugin-os");
    const osType = await type();
    // osType returns: "linux", "macos", "windows", "ios", "android"
    _detectedPlatform = osType;
  } catch {
    // Fallback to userAgent
    const ua = navigator.userAgent.toLowerCase();
    if (ua.includes("iphone") || ua.includes("ipad")) _detectedPlatform = "ios";
    else if (ua.includes("android")) _detectedPlatform = "android";
    else if (ua.includes("win")) _detectedPlatform = "windows";
    else if (ua.includes("mac")) _detectedPlatform = "macos";
    else if (ua.includes("linux")) _detectedPlatform = "linux";
    else _detectedPlatform = "unknown";
  }

  return _detectedPlatform;
}

/**
 * Sync platform getter (returns cached value or fallback)
 */
export function getCurrentPlatform(): string {
  if (_detectedPlatform) return _detectedPlatform;
  // Sync fallback before async detection completes
  if (!isTauri()) return "web";
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("iphone") || ua.includes("ipad")) return "ios";
  if (ua.includes("android")) return "android";
  if (ua.includes("win")) return "windows";
  if (ua.includes("mac")) return "macos";
  if (ua.includes("linux")) return "linux";
  return "unknown";
}

/**
 * Async mobile detection (use this before router init)
 */
export async function detectIsMobile(): Promise<boolean> {
  if (_isMobileResult !== null) return _isMobileResult;

  if (!isTauri()) {
    _isMobileResult = window.innerWidth <= 768;
    return _isMobileResult;
  }

  const platform = await detectPlatform();
  _isMobileResult = platform === "ios" || platform === "android";
  return _isMobileResult;
}

/**
 * Sync mobile check (returns cached value)
 */
export function isMobile(): boolean {
  if (_isMobileResult !== null) return _isMobileResult;
  // Fallback
  if (!isTauri()) return window.innerWidth <= 768;
  const platform = getCurrentPlatform();
  return platform === "ios" || platform === "android";
}
