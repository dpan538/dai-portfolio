/**
 * Home page: Tag ↔ Project hover interaction, FLIP animation, SVG connection lines.
 * Single source of truth: tagToProjects. projectToTags is derived.
 * SVG uses real pixel coordinates. Anchors and control points are precisely defined.
 */

export const tagToProjects: Record<string, string[]> = {
  printmaker: ["masks", "closing_time"],
  photographer: ["reframed_still", "closing_time", "quite_off"],
  conceptual_designer: ["gallery_design", "the_threes"],
  web_developer: ["quite_off", "the_threes"],
  visual_artist: ["masks", "reframed_still", "closing_time", "quite_off", "the_threes"],
  writer: ["the_threes", "quite_off"],
};

/** 6 tag accent colors (low saturation, distinct). Used for pill background and line stroke. */
export const TAG_ACCENTS: Record<string, string> = {
  web_developer: "#4a6fa5",
  conceptual_designer: "#6b5b8a",
  visual_artist: "#8b6b5c",
  printmaker: "#4a7d6b",
  photographer: "#9a7b4a",
  writer: "#5c6b8a",
};

function buildProjectToTags(): Record<string, string[]> {
  const projectToTags: Record<string, string[]> = {};
  for (const [tag, projects] of Object.entries(tagToProjects)) {
    for (const p of projects) {
      if (!projectToTags[p]) projectToTags[p] = [];
      projectToTags[p].push(tag);
    }
  }
  return projectToTags;
}

const projectToTags = buildProjectToTags();

const FLIP_DURATION = 240;
const FLIP_EASING = "cubic-bezier(0.2, 0.8, 0.2, 1)";
const CLEAR_DELAY_MS = 120;
const LINE_Y_OFFSET_STEP = 6;
const BEZIER_DX = 140;

/** Project anchor: right edge + 10px, visual midline 0.55 */
function projectAnchor(rect: DOMRect): { x: number; y: number } {
  return { x: rect.right + 10, y: rect.top + rect.height * 0.55 };
}

/** Tag anchor: left edge - 10px, visual midline 0.55 */
function tagAnchor(rect: DOMRect): { x: number; y: number } {
  return { x: rect.left - 10, y: rect.top + rect.height * 0.55 };
}

function prefersReducedMotion(): boolean {
  return typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function getRect(el: Element): DOMRect {
  return el.getBoundingClientRect();
}

function init() {
  const tags = Array.from(document.querySelectorAll<HTMLElement>(".id-tag[data-tag]"));
  const svgOverlay = document.querySelector("#home-lines") as SVGElement | null;
  const projectLinks = Array.from(document.querySelectorAll<HTMLAnchorElement>("a[data-project]"));
  if (!tags.length || !svgOverlay) return;

  const reducedMotion = prefersReducedMotion();
  let activeTagKeys: string[] = [];
  let activeProjectKey: string | null = null;
  let clearTimeoutId: ReturnType<typeof setTimeout> | null = null;

  const svgNS = "http://www.w3.org/2000/svg";
  const viewport = () => ({ w: window.innerWidth, h: window.innerHeight });

  function updateSvgViewBox() {
    const { w, h } = viewport();
    svgOverlay.setAttribute("viewBox", `0 0 ${w} ${h}`);
  }

  function clearActive() {
    tags.forEach((el) => el.classList.remove("is-active"));
    projectLinks.forEach((el) => el.classList.remove("is-highlight"));
    activeTagKeys = [];
    activeProjectKey = null;
    clearLines();
  }

  function scheduleClear() {
    if (clearTimeoutId) clearTimeout(clearTimeoutId);
    clearTimeoutId = setTimeout(() => {
      clearTimeoutId = null;
      clearActive();
    }, CLEAR_DELAY_MS);
  }

  function cancelClear() {
    if (clearTimeoutId) {
      clearTimeout(clearTimeoutId);
      clearTimeoutId = null;
    }
  }

  function clearLines() {
    const defs = svgOverlay.querySelector("defs");
    const g = svgOverlay.querySelector("g");
    if (defs) defs.innerHTML = "";
    if (g) g.innerHTML = "";
  }

  function lineOpacity(tagKey: string): number {
    return tagKey === "visual_artist" ? 0.22 : 0.48;
  }

  function drawLines() {
    clearLines();
    const defs = svgOverlay.querySelector("defs");
    const g = svgOverlay.querySelector("g");
    if (!defs || !g) return;

    const pathId = () => `line-${Math.random().toString(36).slice(2, 10)}`;
    const gradId = () => `grad-${Math.random().toString(36).slice(2, 10)}`;

    if (activeTagKeys.length === 1) {
      const tagKey = activeTagKeys[0];
      const projects = tagToProjects[tagKey] || [];
      const tagEl = tags.find((t) => t.getAttribute("data-tag") === tagKey);
      if (!tagEl) return;
      const tagR = getRect(tagEl);
      const accent = TAG_ACCENTS[tagKey] ?? "#6a6a6a";
      const opacity = lineOpacity(tagKey);
      const n = projects.length;

      projects.forEach((projectKey, i) => {
        const link = projectLinks.find((a) => a.getAttribute("data-project") === projectKey);
        if (!link) return;
        const linkR = getRect(link);
        const offset = (i - (n - 1) / 2) * LINE_Y_OFFSET_STEP;
        let start = projectAnchor(linkR);
        let end = tagAnchor(tagR);
        start = { x: start.x, y: start.y + offset };
        end = { x: end.x, y: end.y + offset };
        const c1x = start.x + BEZIER_DX;
        const c1y = start.y;
        const c2x = end.x - BEZIER_DX;
        const c2y = end.y;

        const id = pathId();
        const gid = gradId();
        const linear = document.createElementNS(svgNS, "linearGradient");
        linear.setAttribute("id", gid);
        linear.setAttribute("gradientUnits", "userSpaceOnUse");
        linear.setAttribute("x1", String(start.x));
        linear.setAttribute("y1", String(start.y));
        linear.setAttribute("x2", String(end.x));
        linear.setAttribute("y2", String(end.y));
        const stop1 = document.createElementNS(svgNS, "stop");
        stop1.setAttribute("offset", "0%");
        stop1.setAttribute("stop-color", accent);
        stop1.setAttribute("stop-opacity", String(opacity * 0.6));
        const stop2 = document.createElementNS(svgNS, "stop");
        stop2.setAttribute("offset", "100%");
        stop2.setAttribute("stop-color", accent);
        stop2.setAttribute("stop-opacity", String(opacity));
        linear.appendChild(stop1);
        linear.appendChild(stop2);
        defs.appendChild(linear);

        const path = document.createElementNS(svgNS, "path");
        path.setAttribute("id", id);
        path.setAttribute("d", `M ${start.x} ${start.y} C ${c1x} ${c1y} ${c2x} ${c2y} ${end.x} ${end.y}`);
        path.setAttribute("fill", "none");
        path.setAttribute("stroke", `url(#${gid})`);
        path.setAttribute("stroke-width", "1.35");
        path.setAttribute("stroke-linecap", "round");
        path.setAttribute("stroke-linejoin", "round");
        g.appendChild(path);
      });
    } else if (activeProjectKey) {
      const tagKeys = projectToTags[activeProjectKey] || [];
      const link = projectLinks.find((a) => a.getAttribute("data-project") === activeProjectKey);
      if (!link) return;
      const linkR = getRect(link);
      const n = tagKeys.length;

      tagKeys.forEach((tagKey, i) => {
        const tagEl = tags.find((t) => t.getAttribute("data-tag") === tagKey);
        if (!tagEl) return;
        const tagR = getRect(tagEl);
        const offset = (i - (n - 1) / 2) * LINE_Y_OFFSET_STEP;
        let start = projectAnchor(linkR);
        let end = tagAnchor(tagR);
        start = { x: start.x, y: start.y + offset };
        end = { x: end.x, y: end.y + offset };
        const c1x = start.x + BEZIER_DX;
        const c1y = start.y;
        const c2x = end.x - BEZIER_DX;
        const c2y = end.y;

        const accent = TAG_ACCENTS[tagKey] ?? "#6a6a6a";
        const opacity = lineOpacity(tagKey);
        const gid = gradId();
        const linear = document.createElementNS(svgNS, "linearGradient");
        linear.setAttribute("id", gid);
        linear.setAttribute("gradientUnits", "userSpaceOnUse");
        linear.setAttribute("x1", String(start.x));
        linear.setAttribute("y1", String(start.y));
        linear.setAttribute("x2", String(end.x));
        linear.setAttribute("y2", String(end.y));
        const stop1 = document.createElementNS(svgNS, "stop");
        stop1.setAttribute("offset", "0%");
        stop1.setAttribute("stop-color", accent);
        stop1.setAttribute("stop-opacity", String(opacity * 0.6));
        const stop2 = document.createElementNS(svgNS, "stop");
        stop2.setAttribute("offset", "100%");
        stop2.setAttribute("stop-color", accent);
        stop2.setAttribute("stop-opacity", String(opacity));
        linear.appendChild(stop1);
        linear.appendChild(stop2);
        defs.appendChild(linear);

        const path = document.createElementNS(svgNS, "path");
        path.setAttribute("d", `M ${start.x} ${start.y} C ${c1x} ${c1y} ${c2x} ${c2y} ${end.x} ${end.y}`);
        path.setAttribute("fill", "none");
        path.setAttribute("stroke", `url(#${gid})`);
        path.setAttribute("stroke-width", "1.35");
        path.setAttribute("stroke-linecap", "round");
        path.setAttribute("stroke-linejoin", "round");
        g.appendChild(path);
      });
    }
  }

  function runFlip(activate: () => void) {
    if (reducedMotion) {
      activate();
      drawLines();
      return;
    }
    const firstRects = new Map<Element, DOMRect>();
    tags.forEach((el) => firstRects.set(el, getRect(el)));
    activate();
    const lastRects = new Map<Element, DOMRect>();
    tags.forEach((el) => lastRects.set(el, getRect(el)));
    tags.forEach((el) => {
      const first = firstRects.get(el);
      const last = lastRects.get(el);
      if (!first || !last) return;
      const dx = first.left - last.left;
      const dy = first.top - last.top;
      el.style.transform = `translate(${dx}px, ${dy}px)`;
      el.style.transition = "none";
    });
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        tags.forEach((el) => {
          el.style.transition = `transform ${FLIP_DURATION}ms ${FLIP_EASING}`;
          el.style.transform = "translate(0, 0)";
        });
        setTimeout(() => {
          tags.forEach((el) => {
            el.style.transition = "";
            el.style.transform = "";
          });
          drawLines();
        }, FLIP_DURATION);
      });
    });
  }

  function setActiveTag(tagKey: string) {
    cancelClear();
    const projects = tagToProjects[tagKey] || [];
    runFlip(() => {
      activeTagKeys = [tagKey];
      activeProjectKey = null;
      tags.forEach((el) => {
        el.classList.toggle("is-active", el.getAttribute("data-tag") === tagKey);
      });
      projectLinks.forEach((el) => {
        el.classList.toggle("is-highlight", projects.includes(el.getAttribute("data-project") || ""));
      });
    });
  }

  function setActiveProject(projectKey: string) {
    cancelClear();
    const tagKeys = projectToTags[projectKey] || [];
    runFlip(() => {
      activeProjectKey = projectKey;
      activeTagKeys = tagKeys;
      tags.forEach((el) => {
        el.classList.toggle("is-active", tagKeys.includes(el.getAttribute("data-tag") || ""));
      });
      projectLinks.forEach((el) => {
        el.classList.toggle("is-highlight", el.getAttribute("data-project") === projectKey);
      });
    });
  }

  tags.forEach((el) => {
    const key = el.getAttribute("data-tag");
    if (!key) return;
    el.addEventListener("mouseenter", () => setActiveTag(key));
    el.addEventListener("mouseleave", () => scheduleClear());
  });

  projectLinks.forEach((el) => {
    const key = el.getAttribute("data-project");
    if (!key) return;
    el.addEventListener("mouseenter", () => setActiveProject(key));
    el.addEventListener("mouseleave", () => scheduleClear());
  });

  window.addEventListener("resize", () => {
    updateSvgViewBox();
    if (activeTagKeys.length > 0 || activeProjectKey) drawLines();
  });

  updateSvgViewBox();
}

if (typeof document !== "undefined") {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
}
