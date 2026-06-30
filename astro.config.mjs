// @ts-check
import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';

const withoutTrailingSlash = (url) => url.replace(/(?<!:)\/$/, "");

// https://astro.build/config
export default defineConfig({
    site: "https://daipan.art",
    integrations: [
        sitemap({
            filter: (page) => !page.includes("/quite_off_mobile/"),
            serialize: (item) => ({
                ...item,
                url: withoutTrailingSlash(item.url),
                lastmod: new Date("2026-06-30"),
            }),
        }),
    ],
});
