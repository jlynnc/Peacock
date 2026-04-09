import { onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";

/**
 * iOS-style swipe-back gesture.
 * Swipe from the left edge (within 30px) to go back.
 */
export function useSwipeBack() {
  const router = useRouter();

  let startX = 0;
  let startY = 0;
  let tracking = false;

  function onTouchStart(e: TouchEvent) {
    const touch = e.touches[0];
    // Only start tracking if touch begins near left edge
    if (touch.clientX < 30) {
      startX = touch.clientX;
      startY = touch.clientY;
      tracking = true;
    }
  }

  function onTouchEnd(e: TouchEvent) {
    if (!tracking) return;
    tracking = false;

    const touch = e.changedTouches[0];
    const dx = touch.clientX - startX;
    const dy = Math.abs(touch.clientY - startY);

    // Swipe right at least 80px, and mostly horizontal
    if (dx > 80 && dy < dx * 0.5) {
      router.back();
    }
  }

  function onTouchCancel() {
    tracking = false;
  }

  onMounted(() => {
    document.addEventListener("touchstart", onTouchStart, { passive: true });
    document.addEventListener("touchend", onTouchEnd, { passive: true });
    document.addEventListener("touchcancel", onTouchCancel, { passive: true });
  });

  onUnmounted(() => {
    document.removeEventListener("touchstart", onTouchStart);
    document.removeEventListener("touchend", onTouchEnd);
    document.removeEventListener("touchcancel", onTouchCancel);
  });
}
