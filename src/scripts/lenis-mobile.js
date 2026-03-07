import Lenis from "lenis";
import "lenis/dist/lenis.css";

const MOBILE_BREAKPOINT = 768;

function isMobile() {
  return typeof window !== "undefined" && window.innerWidth <= MOBILE_BREAKPOINT;
}

let lenis = null;
let rafId = null;

function initLenis() {
  if (lenis) return;
  lenis = new Lenis({
    duration: 1.5,
    easing: (t) => Math.min(1, 1.001 - Math.pow(2, -10 * t)),
    smoothWheel: true,
  });
  function raf(time) {
    lenis?.raf(time);
    rafId = requestAnimationFrame(raf);
  }
  rafId = requestAnimationFrame(raf);
  if (document.body.classList.contains("mobile-overlay-open")) {
    lenis.stop();
  }
}

function destroyLenis() {
  if (rafId != null) {
    cancelAnimationFrame(rafId);
    rafId = null;
  }
  lenis?.destroy();
  lenis = null;
}

function handleResize() {
  if (isMobile()) {
    initLenis();
  } else {
    destroyLenis();
  }
}

if (typeof window !== "undefined") {
  if (isMobile()) {
    initLenis();
  }
  window.addEventListener("resize", handleResize);
  window.addEventListener("lenis:pause", () => lenis?.stop());
  window.addEventListener("lenis:resume", () => lenis?.start());
}
