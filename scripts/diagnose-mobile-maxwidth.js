#!/usr/bin/env node
/**
 * Diagnose why mobile home .home_content max-width changes have no effect.
 * Run: npm run build && node scripts/diagnose-mobile-maxwidth.js
 */
const fs = require("fs");
const path = require("path");

const distDir = path.join(__dirname, "../dist");
const htmlPath = path.join(distDir, "index.html");

if (!fs.existsSync(htmlPath)) {
  console.error("Run 'npm run build' first.");
  process.exit(1);
}

const html = fs.readFileSync(htmlPath, "utf8");

// Get inline CSS from index
const styleMatch = html.match(/<style[^>]*>([\s\S]*?)<\/style>/);
const inlineCss = styleMatch ? styleMatch[1].replace(/\s+/g, " ") : "";

// Get linked layout CSS
const linkMatch = html.match(/href="(\/_astro\/[^"]+\.css)"/);
let layoutCss = "";
if (linkMatch) {
  const cssPath = path.join(distDir, linkMatch[1].replace(/^\//, ""));
  if (fs.existsSync(cssPath)) {
    layoutCss = fs.readFileSync(cssPath, "utf8").replace(/\s+/g, " ");
  }
}

const allCss = inlineCss + " " + layoutCss;

console.log("\n=== Mobile max-width Diagnostic ===\n");

// 1. Layout's .content > * rule (THE CULPRIT)
const contentChildRule = layoutCss.match(/\.content[^{]*>\s*\*[^{]*\{[^}]*max-width[^}]*\}/);
console.log("1. Layout has .content > * { max-width }:", !!contentChildRule);
if (contentChildRule) {
  const val = contentChildRule[0].match(/max-width\s*:\s*([^;]+)/);
  console.log("   -> Applies max-width:", val ? val[1].trim() : "?", "to ALL .content children");
}

// 2. Override in layout for mobile home
const layoutOverride = layoutCss.match(/content--home\.mobile_main\s*>\s*[^{]*\{[^}]*max-width\s*:\s*28ch/);
console.log("\n2. Layout override .content--home.mobile_main > * { max-width: 28ch }:", !!layoutOverride);

// 3. Index page mobile rule
const indexRule = inlineCss.match(/mobile_main[^.]*\.home_content[^{]*\{[^}]*max-width\s*:\s*([^;!]+)/);
console.log("\n3. Index .mobile_main .home_content max-width:", indexRule ? indexRule[1].trim() : "NOT FOUND");

// 4. Which rule wins (last one in cascade with matching selector)
console.log("\n4. Cascade: Layout loads first (link), then inline (index). Override must be in layout or have !important in index.");

console.log("\n=== End Diagnostic ===\n");
