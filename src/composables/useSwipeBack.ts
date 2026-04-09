import { onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";

/**
 * iOS-style swipe-back gesture with visual feedback.
 * Swipe from the left edge to slide the page right and go back.
 */
export function useSwipeBack() {
  const router = useRouter();

  let startX = 0;
  let startY = 0;
  let tracking = false;
  let overlay: HTMLElement | null = null;

  function createOverlay() {
    overlay = document.createElement("div");
    overlay.style.cssText =
      "position:fixed;inset:0;z-index:9999;pointer-events:none;" +
      "background:linear-gradient(to right, rgba(0,0,0,0.08) 0%, transparent 30%);opacity:0;transition:opacity 0.1s";
    document.body.appendChild(overlay);
  }

  function removeOverlay() {
    if (overlay) {
      overlay.remove();
      overlay = null;
    }
  }

  function onTouchStart(e: TouchEvent) {
    const touch = e.touches[0];
    if (touch.clientX < 30) {
      startX = touch.clientX;
      startY = touch.clientY;
      tracking = true;
      createOverlay();
    }
  }

  function onTouchMove(e: TouchEvent) {
    if (!tracking) return;
    const touch = e.touches[0];
    const dx = touch.clientX - startX;
    const dy = Math.abs(touch.clientY - startY);

    // Cancel if vertical
    if (dy > 30 && dy > dx) {
      tracking = false;
      removeOverlay();
      document.body.style.transform = "";
      return;
    }

    if (dx > 0) {
      // Move the page with the finger, capped at 40% of screen
      const offset = Math.min(dx, window.innerWidth * 0.4);
      document.body.style.transform = `translateX(${offset}px)`;
      document.body.style.transition = "none";
      if (overlay) overlay.style.opacity = String(Math.min(dx / 80, 1));
    }
  }

  function onTouchEnd(e: TouchEvent) {
    if (!tracking) return;
    tracking = false;

    const touch = e.changedTouches[0];
    const dx = touch.clientX - startX;
    const dy = Math.abs(touch.clientY - startY);

    if (dx > 80 && dy < dx * 0.5) {
      // Complete the swipe — animate off screen then navigate back
      document.body.style.transition = "transform 0.2s ease-out";
      document.body.style.transform = `translateX(${window.innerWidth}px)`;
      setTimeout(() => {
        document.body.style.transition = "";
        document.body.style.transform = "";
        removeOverlay();
        router.back();
      }, 200);
    } else {
      // Cancel — snap back
      document.body.style.transition = "transform 0.2s ease-out";
      document.body.style.transform = "";
      setTimeout(() => {
        document.body.style.transition = "";
        removeOverlay();
      }, 200);
    }
  }

  function onTouchCancel() {
    tracking = false;
    document.body.style.transition = "transform 0.2s ease-out";
    document.body.style.transform = "";
    setTimeout(() => {
      document.body.style.transition = "";
      removeOverlay();
    }, 200);
  }

  onMounted(() => {
    document.addEventListener("touchstart", onTouchStart, { passive: true });
    document.addEventListener("touchmove", onTouchMove, { passive: true });
    document.addEventListener("touchend", onTouchEnd, { passive: true });
    document.addEventListener("touchcancel", onTouchCancel, { passive: true });
  });

  onUnmounted(() => {
    document.removeEventListener("touchstart", onTouchStart);
    document.removeEventListener("touchmove", onTouchMove);
    document.removeEventListener("touchend", onTouchEnd);
    document.removeEventListener("touchcancel", onTouchCancel);
    removeOverlay();
    document.body.style.transform = "";
    document.body.style.transition = "";
  });
}
