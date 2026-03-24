/**
 * Detect if running inside Tauri
 */
export function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

/**
 * Detect current platform
 */
export function getCurrentPlatform(): string {
  if (!isTauri()) return "web";
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("win")) return "windows";
  if (ua.includes("mac")) return "macos";
  if (ua.includes("linux")) return "linux";
  if (ua.includes("android")) return "android";
  if (ua.includes("iphone") || ua.includes("ipad")) return "ios";
  return "unknown";
}
