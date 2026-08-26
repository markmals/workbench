import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";

const RECORD_TYPES = {
    proposals: "proposal",
    policies: "policy",
    decisions: "decision",
    guides: "guide",
};

const SCHEMAS = {
    proposal: {
        required: ["id", "title", "authors", "status", "pull-request", "supersedes"],
        lists: ["authors", "supersedes"],
        optionalLists: ["issues"],
        nonEmptyLists: ["authors"],
        statuses: ["draft", "accepted", "implemented", "rejected", "withdrawn", "superseded"],
    },
    policy: {
        required: ["id", "title", "status", "established-by", "supersedes"],
        lists: ["supersedes"],
        statuses: ["draft", "active", "superseded"],
    },
    decision: {
        required: ["id", "title", "status", "established-by", "supersedes"],
        lists: ["supersedes"],
        statuses: ["proposed", "accepted", "superseded"],
    },
    guide: {
        required: ["id", "title", "describes"],
        lists: ["describes"],
    },
    vision: {
        required: ["title", "updated"],
        lists: [],
    },
};

function violation(file, message) {
    return { path: file, message };
}

function artifactType(file) {
    const normalized = file.split(path.sep).join("/");
    if (normalized === "VISION.md") return "vision";
    return RECORD_TYPES[normalized.split("/").at(-2)];
}

function expectedId(file, type) {
    const filename = path.basename(file, ".md");
    if (type === "guide") return `guide.${filename}`;

    const match = filename.match(/^(\d{4})-[a-z0-9]+(?:-[a-z0-9]+)*$/);
    return match ? `${type}.${match[1]}` : null;
}

function referencesFor(artifact, type) {
    const { frontmatter } = artifact;
    const fields =
        type === "guide"
            ? ["describes"]
            : type === "vision"
              ? []
              : ["supersedes", "established-by"];

    return fields.flatMap((field) => {
        if (field === "established-by") {
            return typeof frontmatter[field] === "string"
                ? [{ field, id: frontmatter[field] }]
                : [];
        }
        return Array.isArray(frontmatter[field])
            ? frontmatter[field].map((id) => ({ field, id }))
            : [];
    });
}

// A proposal that documents the clarification convention necessarily writes the marker down.
// Only prose counts as an unresolved question, so fenced blocks and inline spans are stripped
// before the check. An author raising a real question writes it as prose, not as code.
export function withoutCode(markdown) {
    return markdown.replace(/^```[\s\S]*?^```/gm, "").replace(/`[^`\n]*`/g, "");
}

export function parseFrontmatter(source) {
    const lines = source.split(/\r?\n/);
    if (lines[0] !== "---") return { error: "missing YAML frontmatter" };

    const end = lines.indexOf("---", 1);
    if (end === -1) return { error: "unterminated YAML frontmatter" };

    const frontmatter = {};
    for (const line of lines.slice(1, end)) {
        if (line === "") continue;

        const match = line.match(/^([a-z][a-z-]*):[ \t]*(.*)$/);
        if (!match) return { error: `cannot parse frontmatter line "${line}"` };

        const [, key, rawValue] = match;
        if (Object.hasOwn(frontmatter, key)) return { error: `duplicate frontmatter key "${key}"` };

        if (!rawValue.startsWith("[")) {
            frontmatter[key] = rawValue;
            continue;
        }

        if (!rawValue.endsWith("]")) return { error: `cannot parse list value for "${key}"` };
        const contents = rawValue.slice(1, -1).trim();
        if (contents.includes("[") || contents.includes("]"))
            return { error: `cannot parse list value for "${key}"` };

        const values = contents === "" ? [] : contents.split(",").map((value) => value.trim());
        if (values.some((value) => value === ""))
            return { error: `cannot parse list value for "${key}"` };
        frontmatter[key] = values;
    }

    return { frontmatter, content: source };
}

export function validateArtifacts(inputArtifacts) {
    const artifacts = [...inputArtifacts].sort((left, right) =>
        left.path.localeCompare(right.path),
    );
    const violations = [];
    const ids = new Map();

    for (const artifact of artifacts) {
        const type = artifactType(artifact.path);
        const schema = SCHEMAS[type];
        if (!schema) continue;

        const frontmatter = artifact.frontmatter ?? {};
        for (const key of schema.required) {
            if (!Object.hasOwn(frontmatter, key) || frontmatter[key] === undefined) {
                violations.push(violation(artifact.path, `missing required key "${key}"`));
            }
        }
        for (const key of schema.required) {
            if (
                Object.hasOwn(frontmatter, key) &&
                !schema.lists.includes(key) &&
                typeof frontmatter[key] !== "string"
            ) {
                violations.push(violation(artifact.path, `key "${key}" must be a scalar`));
            }
        }
        for (const key of [...schema.lists, ...(schema.optionalLists ?? [])]) {
            if (Object.hasOwn(frontmatter, key) && !Array.isArray(frontmatter[key])) {
                violations.push(violation(artifact.path, `key "${key}" must be an inline list`));
            }
        }
        for (const key of schema.nonEmptyLists ?? []) {
            if (Array.isArray(frontmatter[key]) && frontmatter[key].length === 0) {
                violations.push(
                    violation(artifact.path, `key "${key}" must be a non-empty inline list`),
                );
            }
        }
        if (
            schema.statuses &&
            Object.hasOwn(frontmatter, "status") &&
            !schema.statuses.includes(frontmatter.status)
        ) {
            violations.push(
                violation(artifact.path, `invalid status "${frontmatter.status}" for ${type}`),
            );
        }
        if (
            type === "vision" &&
            Object.hasOwn(frontmatter, "updated") &&
            !/^\d{4}-\d{2}-\d{2}$/.test(frontmatter.updated)
        ) {
            violations.push(violation(artifact.path, 'key "updated" must use YYYY-MM-DD'));
        }

        if (type === "vision" || typeof frontmatter.id !== "string") continue;

        const expected = expectedId(artifact.path, type);
        if (expected === null) {
            violations.push(
                violation(
                    artifact.path,
                    "filename must begin with a four-digit record number and slug",
                ),
            );
        } else if (frontmatter.id !== expected) {
            violations.push(
                violation(
                    artifact.path,
                    `ID "${frontmatter.id}" must be "${expected}" for this filename`,
                ),
            );
        }

        const first = ids.get(frontmatter.id);
        if (first) {
            violations.push(
                violation(
                    artifact.path,
                    `duplicate ID "${frontmatter.id}"; first declared in ${first.path}`,
                ),
            );
        } else {
            ids.set(frontmatter.id, artifact);
        }
    }

    for (const artifact of artifacts) {
        const type = artifactType(artifact.path);
        if (!type) continue;

        for (const { field, id } of referencesFor(artifact, type)) {
            const target = ids.get(id);
            if (!target) {
                violations.push(
                    violation(artifact.path, `reference "${id}" in ${field} does not resolve`),
                );
            } else if (field === "supersedes" && target.frontmatter.status !== "superseded") {
                violations.push(
                    violation(
                        target.path,
                        `record superseded by ${artifact.frontmatter.id} must have status "superseded"`,
                    ),
                );
            }
        }

        if (
            type === "proposal" &&
            ["accepted", "implemented", "superseded"].includes(artifact.frontmatter.status) &&
            withoutCode(artifact.content).includes("[NEEDS CLARIFICATION:")
        ) {
            violations.push(
                violation(
                    artifact.path,
                    "finalized proposal contains a [NEEDS CLARIFICATION:] marker",
                ),
            );
        }
    }

    return violations;
}

export function collectArtifacts(root = process.cwd()) {
    const artifacts = [];
    const violations = [];
    const add = (relativePath) => {
        const source = readFileSync(path.join(root, relativePath), "utf8");
        const parsed = parseFrontmatter(source);
        if (parsed.error) {
            violations.push(violation(relativePath, parsed.error));
        } else {
            artifacts.push({ path: relativePath, ...parsed });
        }
    };

    if (existsSync(path.join(root, "VISION.md"))) add("VISION.md");
    for (const directory of Object.keys(RECORD_TYPES)) {
        const absoluteDirectory = path.join(root, directory);
        if (!existsSync(absoluteDirectory)) continue;
        for (const entry of readdirSync(absoluteDirectory, { withFileTypes: true })) {
            if (entry.isFile() && entry.name.endsWith(".md") && entry.name !== "README.md") {
                add(path.join(directory, entry.name));
            }
        }
    }

    return { artifacts, violations };
}

function main() {
    const json = process.argv.slice(2).includes("--json");
    const collected = collectArtifacts();
    const violations = [...collected.violations, ...validateArtifacts(collected.artifacts)];

    if (json) {
        console.log(JSON.stringify({ violations, count: violations.length }));
    } else if (violations.length > 0) {
        for (const { path: file, message } of violations) console.log(`${file}: ${message}`);
        console.log(`${violations.length} violation${violations.length === 1 ? "" : "s"}`);
    } else {
        console.log("Validation passed: 0 violations");
    }

    if (violations.length > 0) process.exitCode = 1;
}

if (import.meta.main) main();
