#!/usr/bin/env node
/**
 * Pre-push / pre-deploy checks for daipan.art.
 * Run after `npm run build`. Ensures dist has required files and sitemap is valid.
 */

const fs = require("fs");
const path = require("path");

const DIST = path.join(__dirname, "..", "dist");
const SITE_URL = "https://daipan.art";

const requiredPaths = [
  "index.html",
  "robots.txt",
  "sitemap-index.xml",
  "sitemap-0.xml",
  "about/index.html",
  "contact/index.html",
  "closing_time/index.html",
  "gallery_design/index.html",
  "masks/index.html",
  "quite_off/index.html",
  "reframed_still/index.html",
  "the_threes/index.html",
  "404.html",
];

let failed = 0;

// 1. Check required files exist
console.log("[test-push] Checking dist output...\n");
for (const p of requiredPaths) {
  const full = path.join(DIST, p);
  if (fs.existsSync(full)) {
    console.log(`  ✓ ${p}`);
  } else {
    console.log(`  ✗ MISSING: ${p}`);
    failed++;
  }
}

// 2. robots.txt content
const robotsPath = path.join(DIST, "robots.txt");
if (fs.existsSync(robotsPath)) {
  const robots = fs.readFileSync(robotsPath, "utf8");
  if (!robots.includes("Allow:") || !robots.includes("Sitemap:")) {
    console.log("\n  ✗ robots.txt should contain Allow and Sitemap");
    failed++;
  }
  if (!robots.includes(SITE_URL)) {
    console.log("\n  ✗ robots.txt Sitemap should use " + SITE_URL);
    failed++;
  }
}

// 3. Sitemap contains daipan.art
const sitemapPath = path.join(DIST, "sitemap-index.xml");
if (fs.existsSync(sitemapPath)) {
  const sitemap = fs.readFileSync(sitemapPath, "utf8");
  if (!sitemap.includes(SITE_URL)) {
    console.log("\n  ✗ sitemap-index.xml should reference " + SITE_URL);
    failed++;
  }
}

const sitemap0Path = path.join(DIST, "sitemap-0.xml");
if (fs.existsSync(sitemap0Path)) {
  const sitemap0 = fs.readFileSync(sitemap0Path, "utf8");
  const urlCount = (sitemap0.match(/<loc>/g) || []).length;
  if (urlCount < 9) {
    console.log("\n  ✗ sitemap-0.xml expected at least 9 URLs, got " + urlCount);
    failed++;
  }
}

// 4. Homepage has canonical and title
const indexPath = path.join(DIST, "index.html");
if (fs.existsSync(indexPath)) {
  const html = fs.readFileSync(indexPath, "utf8");
  if (!html.includes('href="https://daipan.art"') && !html.includes("href=\"https://daipan.art\"")) {
    console.log("\n  ✗ index.html should contain canonical https://daipan.art");
    failed++;
  }
  if (!html.includes("<title>")) {
    console.log("\n  ✗ index.html should have a title");
    failed++;
  }
}

console.log("");
if (failed > 0) {
  console.log("[test-push] FAILED: " + failed + " check(s) failed.\n");
  process.exit(1);
}
console.log("[test-push] All checks passed. Safe to push.\n");
process.exit(0);
