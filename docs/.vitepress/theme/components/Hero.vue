<template>
    <div class="wb-hero">
        <div class="wb-hero__content">
            <h1 class="wb-hero__title">Workbench</h1>
            <p class="wb-hero__tagline">
                A platform-agnostic, harness-agnostic agentic development process kit. No single
                artifact defines a feature: proposals record intent, tests encode behavior, guides
                explain it, code implements it, previews expose it, and independent review finds
                where they disagree.
            </p>
            <div class="wb-hero__actions">
                <a href="/VISION" class="wb-hero__button wb-hero__button--primary"> Vision </a>
                <a v-if="firstGuideLink" :href="firstGuideLink" class="wb-hero__button"> Guides </a>
            </div>
        </div>
        <div class="wb-hero__art" aria-hidden="true">
            <img :src="heroImage" alt="" />
        </div>
    </div>
</template>

<script setup>
import { useData } from "vitepress";
import { computed } from "vue";
import heroImage from "../../../public/workbench-hero.png?url";

const { theme } = useData();
const firstGuideLink = computed(() => {
    const guidesNav = theme.value?.nav?.find?.((item) => item.text === "Guides");
    return typeof guidesNav?.link === "string" ? guidesNav.link : null;
});
</script>

<style scoped>
.wb-hero {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0;
    width: 100%;
    min-height: 30rem;
    border-bottom: 1px solid var(--vp-c-divider);
}

@media (min-width: 768px) {
    .wb-hero {
        grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    }
}

.wb-hero__content {
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 4rem 2.5rem;
    gap: 1.5rem;
    max-width: 36rem;
}

@media (min-width: 768px) {
    .wb-hero__content {
        padding: 5rem 4rem;
    }
}

.wb-hero__title {
    font-size: 2.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1.1;
    color: var(--vp-c-text-1);
    margin: 0;
}

@media (min-width: 768px) {
    .wb-hero__title {
        font-size: 3.25rem;
    }
}

.wb-hero__tagline {
    font-size: 1.125rem;
    line-height: 1.55;
    color: var(--vp-c-text-2);
    max-width: 28rem;
    margin: 0;
}

.wb-hero__actions {
    display: flex;
    gap: 0.75rem;
    margin-top: 1rem;
    flex-wrap: wrap;
}

.wb-hero__button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.625rem 1.25rem;
    border-radius: 0.5rem;
    font-size: 0.9375rem;
    font-weight: 600;
    text-decoration: none;
    border: 1px solid var(--vp-c-divider);
    color: var(--vp-c-text-1);
    background-color: var(--vp-c-bg-soft);
    transition:
        background-color 0.15s ease,
        border-color 0.15s ease;
}

.wb-hero__button:hover {
    background-color: var(--vp-c-bg-mute);
    border-color: var(--vp-c-divider-2);
}

.wb-hero__button--primary {
    background-color: var(--vp-c-brand-1);
    color: #fff;
    border-color: var(--vp-c-brand-1);
}

.wb-hero__button--primary:hover {
    background-color: var(--vp-c-brand-2);
    border-color: var(--vp-c-brand-2);
    color: #fff;
}

.wb-hero__art {
    position: relative;
    overflow: hidden;
    border-top: 1px solid var(--vp-c-divider);
}

@media (min-width: 768px) {
    .wb-hero__art {
        border-top: none;
        border-left: 1px solid var(--vp-c-divider);
    }
}

.wb-hero__art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    object-position: center;
    display: block;
}
</style>
