/**
 * 构建时脚本：扫描 public/projects 下的图片，生成响应式尺寸版本，输出到 public/_optimized/
 * 同时生成 src/data/image-dimensions.json 供 OptimizedImage 组件使用
 * - 仅 resize，不加重压缩，quality 95，格式与原图一致
 */
import { readdirSync, writeFileSync, mkdirSync } from "fs";
import { join, extname, dirname } from "path";
import { fileURLToPath } from "url";
import sharp from "sharp";

const __dirname = dirname(fileURLToPath(import.meta.url));
const PUBLIC_PROJECTS = join(__dirname, "../public/projects");
const OUTPUT_OPTIMIZED = join(__dirname, "../public/_optimized");
const OUTPUT_JSON = join(__dirname, "../src/data/image-dimensions.json");

const EXTENSIONS = new Set([".jpg", ".jpeg", ".png", ".webp"]);
const WIDTHS = [400, 640, 800, 1024, 1280, 1600];
const QUALITY = 95;

function walkDir(dir, basePath = "", files = []) {
    const entries = readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
        const fullPath = join(dir, entry.name);
        const relPath = basePath ? `${basePath}/${entry.name}` : entry.name;
        if (entry.isDirectory()) {
            walkDir(fullPath, relPath, files);
        } else if (EXTENSIONS.has(extname(entry.name).toLowerCase())) {
            files.push({ fullPath, relPath });
        }
    }
    return files;
}

function addAlternatePaths(data) {
    const copy = { ...data };
    for (const [path, val] of Object.entries(data)) {
        if (path.startsWith("/projects/quite_off/")) {
            copy[path.replace("/projects/quite_off", "/quite_off")] = val;
        }
    }
    return copy;
}

async function processImage(fullPath, relPath) {
    const ext = extname(relPath).toLowerCase();
    const format = ext === ".png" ? "png" : "jpeg";
    const baseName = relPath.replace(ext, "");

    const meta = await sharp(fullPath).metadata();
    const origW = meta.width || 0;
    const origH = meta.height || 0;
    if (!origW || !origH) return null;

    const srcset = [];
    let targetWidths = WIDTHS.filter((w) => w <= origW);
    if (targetWidths.length === 0) targetWidths = [origW];

    for (const w of targetWidths) {
        const outRel = `projects/${baseName}_${w}w${ext}`;
        const outPath = join(OUTPUT_OPTIMIZED, outRel);
        mkdirSync(dirname(outPath), { recursive: true });
        await sharp(fullPath)
            .resize(w, null, { withoutEnlargement: true })
            [format](format === "png" ? {} : { quality: QUALITY })
            .toFile(outPath);
        srcset.push({ url: `/_optimized/${outRel}`, w });
    }
    return { width: origW, height: origH, srcset };
}

async function main() {
    const files = walkDir(PUBLIC_PROJECTS);
    const result = {};
    for (const { fullPath, relPath } of files) {
        try {
            const data = await processImage(fullPath, relPath);
            if (data) result[`/projects/${relPath}`] = data;
        } catch (e) {
            console.warn(`[get-image-dimensions] Skip ${relPath}:`, e.message);
        }
    }
    const withAlternates = addAlternatePaths(result);
    mkdirSync(dirname(OUTPUT_JSON), { recursive: true });
    writeFileSync(OUTPUT_JSON, JSON.stringify(withAlternates, null, 2), "utf8");
    console.log(`[get-image-dimensions] Generated ${Object.keys(withAlternates).length} images → public/_optimized/`);
}

main();
