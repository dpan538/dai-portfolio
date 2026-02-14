/**
 * Home page: Tag ↔ Project hover interaction, FLIP animation, SVG connection lines.
 * Single source of truth: tagToProjects. projectToTags is derived.
 */

export const tagToProjects: Record<string, string[]> = {
  printmaker: ["masks", "closing_time"],
  photographer: ["reframed_still", "closing_time", "quite_off"],
  conceptual_designer: ["gallery_design", "the_threes"],
  web_developer: ["quite_off", "the_threes"],
  visual_artist: ["masks", "reframed_still", "closing_time", "quite_off", "the_threes"],
  writer: ["the_threes", "quite_off"],
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

const FLIP_DURATION = 280;
const FLIP_EASING = "cubic-bezier(0.2, 0.8, 0.2, 1)";
const BEZIER_OFFSET = 120;

function prefersReducedMotion(): boolean {
  return typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function getRect(el: Element): DOMRect {
  return el.getBoundingClientRect();
}

function center(rect: DOMRect): { x: number; y: number } {
  return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
}

function scaleToViewBox(
  x: number,
  y: number,
  w: number,
  h: number
): { x: number; y: number } {
  return { x: (x / w) * 100, y: (y / h) * 100 };
}

function init() {
  const tagsContainer = document.querySelector(".home-tags");
  const svgOverlay = document.querySelector("#home-lines") as SVGElement | null;
  if (!tagsContainer || !svgOverlay) return;

  const tags = Array.from(tagsContainer.querySelectorAll<HTMLElement>("[data-tag]"));
  const projectLinks = Array.from(document.querySelectorAll<HTMLAnchorElement>("a[data-project]"));
  const reducedMotion = prefersReducedMotion();

  let activeTagKeys: string[] = [];
  let activeProjectKey: string | null = null;

  const svgNS = "http://www.w3.org/2000/svg";
  const viewport = () => ({ w: window.innerWidth, h: window.innerHeight });

  function clearActive() {
    tags.forEach((el) => el.classList.remove("is-active"));
    projectLinks.forEach((el) => el.classList.remove("is-active"));
    activeTagKeys = [];
    activeProjectKey = null;
    clearLines();
  }

  function clearLines() {
    if (!svgOverlay) return;
    const g = svgOverlay.querySelector("g");
    if (g) g.innerHTML = "";
  }

  function drawLines() {
    clearLines();
    const g = svgOverlay?.querySelector("g");
    if (!g) return;
    const { w, h } = viewport();
    const toV = (x: number, y: number) => scaleToViewBox(x, y, w, h);

    if (activeTagKeys.length === 1) {
      const tagKey = activeTagKeys[0];
      const projects = tagToProjects[tagKey] || [];
      const tagEl = tags.find((t) => t.getAttribute("data-tag") === tagKey);
      if (!tagEl) return;
      const tagRect = getRect(tagEl);
      const T = center(tagRect);
      const isVisualArtist = tagKey === "visual_artist";

      for (const projectKey of projects) {
        const link = projectLinks.find((a) => a.getAttribute("data-project") === projectKey);
        if (!link) continue;
        const linkRect = getRect(link);
        const P = center(linkRect);
        const c1x = P.x + BEZIER_OFFSET;
        const c1y = P.y;
        const c2x = T.x - BEZIER_OFFSET;
        const c2y = T.y;
        const p1 = toV(P.x, P.y);
        const c1 = toV(c1x, c1y);
        const c2 = toV(c2x, c2y);
        const p2 = toV(T.x, T.y);
        const path = document.createElementNS(svgNS, "path");
        path.setAttribute(
          "d",
          `M ${p1.x} ${p1.y} C ${c1.x} ${c1.y} ${c2.x} ${c2.y} ${p2.x} ${p2.y}`
        );
        path.setAttribute("fill", "none");
        path.setAttribute("stroke", isVisualArtist ? "rgba(255,255,255,0.25)" : "rgba(255,255,255,0.45)");
        path.setAttribute("stroke-width", "1.2");
        path.setAttribute("stroke-linecap", "round");
        g.appendChild(path);
      }
    } else if (activeProjectKey) {
      const tagKeys = projectToTags[activeProjectKey] || [];
      const link = projectLinks.find((a) => a.getAttribute("data-project") === activeProjectKey);
      if (!link) return;
      const linkRect = getRect(link);
      const P = center(linkRect);

      for (const tagKey of tagKeys) {
        const tagEl = tags.find((t) => t.getAttribute("data-tag") === tagKey);
        if (!tagEl) continue;
        const tagRect = getRect(tagEl);
        const T = center(tagRect);
        const c1x = P.x + BEZIER_OFFSET;
        const c1y = P.y;
        const c2x = T.x - BEZIER_OFFSET;
        const c2y = T.y;
        const p1 = toV(P.x, P.y);
        const c1 = toV(c1x, c1y);
        const c2 = toV(c2x, c2y);
        const p2 = toV(T.x, T.y);
        const path = document.createElementNS(svgNS, "path");
        path.setAttribute(
          "d",
          `M ${p1.x} ${p1.y} C ${c1.x} ${c1.y} ${c2.x} ${c2.y} ${p2.x} ${p2.y}`
        );
        path.setAttribute("fill", "none");
        path.setAttribute(
          "stroke",
          tagKey === "visual_artist" ? "rgba(255,255,255,0.25)" : "rgba(255,255,255,0.45)"
        );
        path.setAttribute("stroke-width", "1.2");
        path.setAttribute("stroke-linecap", "round");
        g.appendChild(path);
      }
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
      (el as HTMLElement).style.transform = `translate(${dx}px, ${dy}px)`;
      (el as HTMLElement).style.transition = "none";
    });
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        tags.forEach((el) => {
          const html = el as HTMLElement;
          html.style.transition = `transform ${FLIP_DURATION}ms ${FLIP_EASING}`;
          html.style.transform = "translate(0, 0)";
        });
        setTimeout(() => {
          tags.forEach((el) => {
            const html = el as HTMLElement;
            html.style.transition = "";
            html.style.transform = "";
          });
          drawLines();
        }, FLIP_DURATION);
      });
    });
  }

  function setActiveTag(tagKey: string) {
    const projects = tagToProjects[tagKey] || [];
    runFlip(() => {
      activeTagKeys = [tagKey];
      activeProjectKey = null;
      tags.forEach((el) => {
        el.classList.toggle("is-active", el.getAttribute("data-tag") === tagKey);
      });
      projectLinks.forEach((el) => {
        el.classList.toggle("is-active", projects.includes(el.getAttribute("data-project") || ""));
      });
    });
  }

  function setActiveProject(projectKey: string) {
    const tagKeys = projectToTags[projectKey] || [];
    runFlip(() => {
      activeProjectKey = projectKey;
      activeTagKeys = tagKeys;
      tags.forEach((el) => {
        el.classList.toggle("is-active", tagKeys.includes(el.getAttribute("data-tag") || ""));
      });
      projectLinks.forEach((el) => {
        el.classList.toggle("is-active", el.getAttribute("data-project") === projectKey);
      });
    });
  }

  tags.forEach((el) => {
    const key = el.getAttribute("data-tag");
    if (!key) return;
    el.addEventListener("mouseenter", () => setActiveTag(key));
    el.addEventListener("mouseleave", () => clearActive());
  });

  projectLinks.forEach((el) => {
    const key = el.getAttribute("data-project");
    if (!key) return;
    el.addEventListener("mouseenter", () => setActiveProject(key));
    el.addEventListener("mouseleave", () => clearActive());
  });

  window.addEventListener("resize", () => {
    if (activeTagKeys.length > 0 || activeProjectKey) drawLines();
  });

  const svg = svgOverlay as SVGElement & { viewBox: string };
  function updateSvgViewBox() {
    const { w, h } = viewport();
    svg.setAttribute("viewBox", `0 0 ${w} ${h}`);
  }
  updateSvgViewBox();
  window.addEventListener("resize", updateSvgViewBox);
}

if (typeof document !== "undefined") {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
}
