/**
 * Diagnostic script: verify mobile home page structure and styles
 * Run: node scripts/check-mobile-home.js
 */
const fs = require("fs");
const path = require("path");

const htmlPath = path.join(__dirname, "../dist/index.html");
const html = fs.readFileSync(htmlPath, "utf8");

const checks = [];
let passed = 0;
let failed = 0;

function check(name, condition) {
  if (condition) {
    checks.push(`✓ ${name}`);
    passed++;
  } else {
    checks.push(`✗ ${name}`);
    failed++;
  }
}

// DOM structure
check("header.mobile_header exists", html.includes('class="mobile_header"'));
check("main has mobile_main class", html.includes("mobile_main"));
check("home_content exists", html.includes("home_content"));
check("site_footer exists", html.includes("site_footer"));
check("copyright text present", html.includes("©") && html.includes("Dai Pan"));

// Mobile styles
check("mobile_main overflow-y: auto", html.includes("overflow-y:auto") || html.includes("overflow-y: auto"));
check("mobile_main max-width 36ch", html.includes("36ch"));
check("site_footer display:block on mobile", html.includes("site_footer") && (html.includes("display:block") || html.includes("display: block")));

// content--home override
check("content--home.mobile_main overflow override", html.includes("content--home") && html.includes("overflow-y") && html.includes("auto"));

console.log("\n=== Mobile Home Page Diagnostic ===\n");
checks.forEach((c) => console.log(c));
console.log(`\n${passed} passed, ${failed} failed\n`);
process.exit(failed > 0 ? 1 : 0);
