<template>
    <div class="sdd-hero">
        <div class="sdd-hero__content">
            <h1 class="sdd-hero__title">Spec-Driven Development</h1>
            <p class="sdd-hero__tagline">
                A spec-driven multiplatform app harness. One set of specs; native implementations on
                web, iOS, and Android.
            </p>
            <div class="sdd-hero__actions">
                <a href="/specs/CONVENTIONS" class="sdd-hero__button sdd-hero__button--primary">
                    Specs
                </a>
                <a :href="firstFeatureLink" class="sdd-hero__button"> Features </a>
            </div>
        </div>
        <div class="sdd-hero__art" aria-hidden="true">
            <img :src="heroImage" alt="" />
        </div>
    </div>
</template>

<script setup>
import { useData } from "vitepress";
import { computed } from "vue";
import heroImage from "../../../public/workbench-hero.png?url";

const { theme } = useData();
const firstFeatureLink = computed(() => {
    const featuresNav = theme.value?.nav?.find?.((n) => n.text === "Features");
    return featuresNav?.link ?? "/";
});
</script>

<style scoped>
.sdd-hero {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0;
    width: 100%;
    min-height: 30rem;
    border-bottom: 1px solid var(--vp-c-divider);
}

@media (min-width: 768px) {
    .sdd-hero {
        grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    }
}

.sdd-hero__content {
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 4rem 2.5rem;
    gap: 1.5rem;
    max-width: 36rem;
}

@media (min-width: 768px) {
    .sdd-hero__content {
        padding: 5rem 4rem;
    }
}

.sdd-hero__title {
    font-size: 2.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1.1;
    color: var(--vp-c-text-1);
    margin: 0;
}

@media (min-width: 768px) {
    .sdd-hero__title {
        font-size: 3.25rem;
    }
}

.sdd-hero__tagline {
    font-size: 1.125rem;
    line-height: 1.55;
    color: var(--vp-c-text-2);
    max-width: 28rem;
    margin: 0;
}

.sdd-hero__actions {
    display: flex;
    gap: 0.75rem;
    margin-top: 1rem;
    flex-wrap: wrap;
}

.sdd-hero__button {
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

.sdd-hero__button:hover {
    background-color: var(--vp-c-bg-mute);
    border-color: var(--vp-c-divider-2);
}

.sdd-hero__button--primary {
    background-color: var(--vp-c-brand-1);
    color: #1c1917;
    border-color: var(--vp-c-brand-1);
}

.sdd-hero__button--primary:hover {
    background-color: var(--vp-c-brand-2);
    border-color: var(--vp-c-brand-2);
    color: #fff;
}

.sdd-hero__art {
    position: relative;
    overflow: hidden;
    border-top: 1px solid var(--vp-c-divider);
}

@media (min-width: 768px) {
    .sdd-hero__art {
        border-top: none;
        border-left: 1px solid var(--vp-c-divider);
    }
}

.sdd-hero__art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    object-position: center;
    display: block;
}
</style>
