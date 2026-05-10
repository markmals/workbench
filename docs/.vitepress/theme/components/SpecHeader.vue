<script setup lang="ts">
import { computed } from "vue";
import { useData } from "vitepress";

const { frontmatter } = useData();

const meta = computed(() => {
    const fm = frontmatter.value as Record<string, unknown>;
    return {
        id: typeof fm.id === "string" ? fm.id : null,
        kind: typeof fm.kind === "string" ? fm.kind : null,
        status: typeof fm.status === "string" ? fm.status : null,
        dependsOn: Array.isArray(fm["depends-on"]) ? (fm["depends-on"] as string[]) : [],
    };
});

const visible = computed(() => meta.value.id !== null || meta.value.kind !== null);
</script>

<template>
    <div v-if="visible" class="spec-header">
        <div class="spec-header__row">
            <span
                v-if="meta.kind"
                :class="['spec-header__kind', `spec-header__kind--${meta.kind}`]"
            >
                {{ meta.kind }}
            </span>
            <code v-if="meta.id" class="spec-header__id">{{ meta.id }}</code>
            <span v-if="meta.status" class="spec-header__status">
                {{ meta.status }}
            </span>
        </div>
        <div v-if="meta.dependsOn.length > 0" class="spec-header__deps">
            <span class="spec-header__deps-label">depends on</span>
            <code v-for="dep in meta.dependsOn" :key="dep" class="spec-header__dep">{{ dep }}</code>
        </div>
    </div>
</template>

<style scoped>
.spec-header {
    margin-bottom: 1.5rem;
    padding: 0.75rem 1rem;
    border: 1px solid var(--vp-c-divider);
    border-radius: 0.5rem;
    background-color: var(--vp-c-bg-soft);
    font-size: 0.875rem;
}

.spec-header__row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
}

.spec-header__kind {
    display: inline-block;
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    font-family: var(--vp-font-family-mono);
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    background-color: var(--vp-c-default-soft);
    color: var(--vp-c-text-1);
}

.spec-header__kind--story {
    background-color: rgba(100, 108, 255, 0.15);
    color: var(--vp-c-brand-1);
}
.spec-header__kind--use-case {
    background-color: rgba(100, 108, 255, 0.1);
    color: var(--vp-c-brand-1);
}
.spec-header__kind--flow {
    background-color: rgba(100, 108, 255, 0.1);
    color: var(--vp-c-brand-1);
}
.spec-header__kind--domain {
    background-color: rgba(56, 189, 248, 0.15);
    color: rgb(56, 189, 248);
}
.spec-header__kind--view-model {
    background-color: rgba(34, 197, 94, 0.15);
    color: rgb(34, 197, 94);
}
.spec-header__kind--error {
    background-color: rgba(239, 68, 68, 0.15);
    color: rgb(239, 68, 68);
}
.spec-header__kind--narrative {
    background-color: rgba(168, 85, 247, 0.15);
    color: rgb(168, 85, 247);
}
.spec-header__kind--architecture,
.spec-header__kind--design-system,
.spec-header__kind--conventions {
    background-color: rgba(245, 158, 11, 0.15);
    color: rgb(245, 158, 11);
}

.spec-header__id {
    font-family: var(--vp-font-family-mono);
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--vp-c-text-1);
    background: transparent;
    padding: 0;
}

.spec-header__status {
    font-family: var(--vp-font-family-mono);
    font-size: 0.75rem;
    color: var(--vp-c-text-2);
    margin-left: auto;
}

.spec-header__deps {
    margin-top: 0.5rem;
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
    align-items: center;
    font-size: 0.75rem;
}

.spec-header__deps-label {
    color: var(--vp-c-text-3);
    font-family: var(--vp-font-family-mono);
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.spec-header__dep {
    font-family: var(--vp-font-family-mono);
    font-size: 0.75rem;
    background: var(--vp-c-default-soft);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
}
</style>
