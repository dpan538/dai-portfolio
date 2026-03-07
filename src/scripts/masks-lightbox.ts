/**
 * Masks page: desktop and mobile lightbox.
 */
export function initMasksLightbox(): void {
    const lightboxDesktop = document.getElementById("lightbox-desktop");
    const lightboxImageDesktop = document.getElementById("lightbox-image-desktop");
    const imagesDesktop = document.querySelectorAll(".masks-desktop .image");
    if (imagesDesktop.length && lightboxDesktop && lightboxImageDesktop) {
        initDesktop(lightboxDesktop, lightboxImageDesktop, [...imagesDesktop]);
    }

    const lightbox = document.getElementById("lightbox");
    const lightboxImage = document.getElementById("lightbox-image");
    const imagesMobile = document.querySelectorAll(".masks-image");
    if (lightbox && lightboxImage) {
        initMobile(lightbox, lightboxImage, [...imagesMobile]);
    }
}

function initDesktop(
    lightbox: HTMLElement,
    lightboxImage: HTMLElement,
    images: Element[]
): void {
    const prevBtn = lightbox.querySelector(".prev");
    const nextBtn = lightbox.querySelector(".next");
    const sources = images
        .map((el) => el.querySelector<HTMLImageElement>("img")?.src)
        .filter((s): s is string => Boolean(s));
    let current = 0;

    function open(i: number): void {
        current = i;
        lightboxImage.setAttribute("src", sources[current] || "");
        lightbox.classList.remove("hidden");
    }

    images.forEach((el, i) => {
        el.addEventListener("click", () => open(i));
        el.addEventListener("mouseenter", () =>
            el.closest(".row")?.classList.add("is-image-hovered")
        );
        el.addEventListener("mouseleave", () =>
            el.closest(".row")?.classList.remove("is-image-hovered")
        );
    });

    prevBtn?.addEventListener("click", () => {
        current = (current - 1 + sources.length) % sources.length;
        lightboxImage.setAttribute("src", sources[current] || "");
    });
    nextBtn?.addEventListener("click", () => {
        current = (current + 1) % sources.length;
        lightboxImage.setAttribute("src", sources[current] || "");
    });
    lightbox.addEventListener("click", (e) => {
        if (e.target === lightbox) lightbox.classList.add("hidden");
    });
}

function initMobile(
    lightbox: HTMLElement,
    lightboxImage: HTMLElement,
    images: Element[]
): void {
    const prevBtn = lightbox.querySelector(".prev");
    const nextBtn = lightbox.querySelector(".next");
    const closeBtn = lightbox.querySelector(".masks-lightbox__close");
    const sources = images
        .map((el) => el.querySelector<HTMLImageElement>("img")?.src)
        .filter((s): s is string => Boolean(s));
    let current = 0;

    function openLightbox(i: number): void {
        current = i;
        lightboxImage.setAttribute("src", sources[current] || "");
        lightbox.classList.remove("hidden");
        document.body.style.overflow = "hidden";
        window.dispatchEvent(new Event("lenis:pause"));
    }

    function closeLightbox(): void {
        lightbox.classList.add("hidden");
        document.body.style.overflow = "";
        window.dispatchEvent(new Event("lenis:resume"));
    }

    images.forEach((el, i) => {
        el.addEventListener("click", () => openLightbox(i));
        el.addEventListener("mouseenter", () =>
            el.closest(".masks-work")?.classList.add("is-hovered")
        );
        el.addEventListener("mouseleave", () =>
            el.closest(".masks-work")?.classList.remove("is-hovered")
        );
    });
    prevBtn?.addEventListener("click", (e) => {
        e.stopPropagation();
        current = (current - 1 + sources.length) % sources.length;
        lightboxImage.setAttribute("src", sources[current] || "");
    });
    nextBtn?.addEventListener("click", (e) => {
        e.stopPropagation();
        current = (current + 1) % sources.length;
        lightboxImage.setAttribute("src", sources[current] || "");
    });
    closeBtn?.addEventListener("click", (e) => {
        e.stopPropagation();
        closeLightbox();
    });
    lightbox.addEventListener("click", (e) => {
        if (e.target === lightbox) closeLightbox();
    });
    document.addEventListener("keydown", (e) => {
        if (e.key === "Escape" && !lightbox.classList.contains("hidden"))
            closeLightbox();
        if (!lightbox.classList.contains("hidden")) {
            if (e.key === "ArrowLeft") {
                e.preventDefault();
                current = (current - 1 + sources.length) % sources.length;
                lightboxImage.setAttribute("src", sources[current] || "");
            }
            if (e.key === "ArrowRight") {
                e.preventDefault();
                current = (current + 1) % sources.length;
                lightboxImage.setAttribute("src", sources[current] || "");
            }
        }
    });
    let tx = 0;
    lightbox.addEventListener(
        "touchstart",
        (e) => {
            tx = e.touches[0].clientX;
        },
        { passive: true }
    );
    lightbox.addEventListener(
        "touchend",
        (e) => {
            const d = e.changedTouches[0].clientX - tx;
            if (Math.abs(d) > 50) {
                current =
                    d > 0
                        ? (current - 1 + sources.length) % sources.length
                        : (current + 1) % sources.length;
                lightboxImage.setAttribute("src", sources[current] || "");
            }
        },
        { passive: true }
    );
}
