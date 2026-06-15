// @ts-check
import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';

// https://astro.build/config
export default defineConfig({
    site: "https://daipan.art",
    integrations: [
        sitemap({
            filter: (page) => !page.includes("/quite_off_mobile/"),
            serialize: (item) => ({
                ...item,
                lastmod: new Date("2026-06-15"),
            }),
        }),
    ],
});
