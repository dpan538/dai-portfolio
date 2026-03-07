/**
 * Lenis smooth scroll - mobile only (experiment)
 */
import Lenis from "lenis";
import "lenis/dist/lenis.css";

const MOBILE_BREAKPOINT = 768;

function isMobile(): boolean {
  return typeof window !== "undefined" && window.innerWidth <= MOBILE_BREAKPOINT;
}

let lenis: Lenis | null = null;
let rafId: number | null = null;

function initLenis(): void {
  if (lenis) return;
  lenis = new Lenis({
    duration: 1.2,
    easing: (t) => Math.min(1, 1.001 - Math.pow(2, -10 * t)),
    smoothWheel: true,
  });

  function raf(time: number): void {
    lenis?.raf(time);
    rafId = requestAnimationFrame(raf);
  }
  rafId = requestAnimationFrame(raf);
}

function destroyLenis(): void {
  if (rafId != null) {
    cancelAnimationFrame(rafId);
    rafId = null;
  }
  lenis?.destroy();
  lenis = null;
}

function handleResize(): void {
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
}
