/**
 * Home page: Tag ↔ Project hover, FLIP. No connection lines.
 */

export const tagToProjects: Record<string, string[]> = {
  dai_pan: ["about"],
  printmaker: ["masks", "closing_time"],
  photographer: ["reframed_still", "closing_time", "quite_off"],
  conceptual_designer: ["gallery_design", "the_threes"],
  web_developer: ["quite_off", "the_threes"],
  visual_artist: ["masks", "reframed_still", "closing_time", "quite_off", "the_threes"],
  writer: ["the_threes", "quite_off"],
};

export const TAG_ACCENTS: Record<string, string> = {
  dai_pan: "#2d3748",
  web_developer: "#4a6fa5",
  conceptual_designer: "#6b5b8a",
  visual_artist: "#8b6b5c",
  printmaker: "#4a7d6b",
  photographer: "#9a7b4a",
  writer: "#5c6b8a",
};

function buildProjectToTags(): Record<string, string[]> {
  const out: Record<string, string[]> = {};
  for (const [tag, projects] of Object.entries(tagToProjects)) {
    for (const p of projects) {
      if (!out[p]) out[p] = [];
      out[p].push(tag);
    }
  }
  return out;
}

const projectToTags = buildProjectToTags();

const FLIP_DURATION = 440;
const FLIP_EASING = "cubic-bezier(0.25, 0.1, 0.25, 1)";
const CLEAR_DELAY_MS = 120;

function prefersReducedMotion(): boolean {
  return typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function getRect(el: Element): DOMRect {
  return el.getBoundingClientRect();
}

function init() {
  const tags = Array.from(document.querySelectorAll<HTMLElement>(".id-tag[data-tag]"));
  const projectLinks = Array.from(document.querySelectorAll<HTMLAnchorElement>("a[data-project]"));
  if (!tags.length) return;

  const reducedMotion = prefersReducedMotion();
  let activeTagKeys: string[] = [];
  let activeProjectKey: string | null = null;
  let clearTimeoutId: ReturnType<typeof setTimeout> | null = null;

  function clearActive() {
    tags.forEach((el) => el.classList.remove("is-active"));
    projectLinks.forEach((el) => el.classList.remove("is-highlight"));
    activeTagKeys = [];
    activeProjectKey = null;
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

  function runFlip(activate: () => void) {
    if (reducedMotion) {
      activate();
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
      tags.forEach((el) => el.classList.toggle("is-active", el.getAttribute("data-tag") === tagKey));
      projectLinks.forEach((el) => el.classList.toggle("is-highlight", projects.includes(el.getAttribute("data-project") || "")));
    });
  }

  function setActiveProject(projectKey: string) {
    cancelClear();
    const tagKeys = projectToTags[projectKey] || [];
    runFlip(() => {
      activeProjectKey = projectKey;
      activeTagKeys = tagKeys;
      tags.forEach((el) => el.classList.toggle("is-active", tagKeys.includes(el.getAttribute("data-tag") || "")));
      projectLinks.forEach((el) => el.classList.toggle("is-highlight", el.getAttribute("data-project") === projectKey));
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
}

if (typeof document !== "undefined") {
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
}
