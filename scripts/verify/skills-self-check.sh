#!/usr/bin/env bash
# scripts/verify/skills-self-check.sh — meta-verifier for the
# committed `.skillshare/` skill set itself.
#
# Checks:
#   1. Every .skillshare/skills/*/*/SKILL.md has the required
#      frontmatter (name + description) and the six canonical
#      sections (Use when, Contract, Phase, Stop condition,
#      Boundary, Helper). The "Final Report" section is
#      recommended but not enforced.
#   2. Every scripts/<group>/*.sh is executable and has a
#      `#!/usr/bin/env bash` shebang.
#   3. Every `$skill-name` cross-reference in a SKILL.md body
#      resolves to an actual skill directory.
#   4. Every Helper block that names a `scripts/<path>` resolves
#      to a real file.
#
# Exit codes:
#   0 — every check passes
#   1 — at least one violation
#   2 — script error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../helpers/lib.sh"

cd "$(repo_root)"

violations=0

# Sections required of every committed SKILL.md. Skills come in
# three shapes; classification is done by inspecting the SKILL.md
# itself (presence of `## Phase` headings), not by directory path,
# so a future skill placed under any group cannot accidentally
# escape the right gate.
#
#   - orchestrator (has `## Phase`) — uses the full openclaw
#     grammar including Phase blocks plus a Helper section.
#   - verifier (`verify-*` name or no Phase block) — thin wrapper
#     around scripts/<group>/<name>.sh.
REQUIRED_COMMON=(
    "## Use when"
    "## Contract"
    "## Stop condition"
    "## Boundary"
)
REQUIRED_ORCHESTRATOR_ONLY=(
    "## Phase 1"
    "## Helper"
)

# Skills exempt from the openclaw shape. Currently none; kept as a
# named array so re-adding an exemption (e.g. a future hand-rolled
# manual-procedure skill) is a one-line change.
EXEMPT_SKILLS=()

mapfile -t SKILL_FILES < <(find .skillshare/skills -name SKILL.md -type f 2>/dev/null \
    | sort)
log_info "checking ${#SKILL_FILES[@]} SKILL.md files"

declare -A SKILL_NAMES=()

for file in "${SKILL_FILES[@]}"; do
    # Build the canonical skill name from the file path so cross-
    # references like `$adr-consistency-check` can be resolved.
    name="$(basename "$(dirname "$file")")"
    SKILL_NAMES["$name"]=1

    # 1a. Frontmatter — first non-blank line must be `---`.
    first_line=$(head -n1 "$file")
    if [[ "$first_line" != "---" ]]; then
        json_finding kind=missing-frontmatter file="$file" \
            detail="SKILL.md must start with --- frontmatter" >&2
        violations=$((violations + 1))
        continue
    fi
    if ! grep -qE '^name:[[:space:]]*[A-Za-z0-9_-]+' "$file"; then
        json_finding kind=missing-name file="$file" \
            detail="frontmatter missing name field" >&2
        violations=$((violations + 1))
    fi
    if ! grep -qE '^description:' "$file"; then
        json_finding kind=missing-description file="$file" \
            detail="frontmatter missing description field" >&2
        violations=$((violations + 1))
    fi

    # Exempt skills (predate the openclaw layout) skip all shape
    # checks but still must declare frontmatter.
    is_exempt=0
    for ex in "${EXEMPT_SKILLS[@]}"; do
        [[ "$name" == "$ex" ]] && is_exempt=1
    done
    if [[ "$is_exempt" -eq 1 ]]; then
        continue
    fi
    # Shape classification by SKILL.md content, not by path. A
    # skill is orchestrator-shaped iff it declares `## Phase`
    # blocks. Verifier-shape skills (`verify-*` or no Phase block)
    # are thin wrappers and opt out of the Phase / Helper checks.
    is_orchestrator=0
    if [[ "$name" != verify-* ]] && grep -qE '^## Phase' "$file"; then
        is_orchestrator=1
    fi

    # 1b. Required sections — common to every non-exempt skill.
    for section in "${REQUIRED_COMMON[@]}"; do
        if ! grep -qE "^${section}" "$file"; then
            json_finding kind=missing-section file="$file" section="$section" \
                detail="SKILL.md missing required section" >&2
            violations=$((violations + 1))
        fi
    done

    # 1c. Orchestrator-only sections.
    if [[ "$is_orchestrator" -eq 1 ]]; then
        for section in "${REQUIRED_ORCHESTRATOR_ONLY[@]}"; do
            # Phase 1 is matched as a prefix because the heading
            # carries a per-skill title (`## Phase 1: Gather
            # context`). Skills under `verify/` are exempt from
            # Phase blocks by shape.
            if ! grep -qE "^${section}" "$file"; then
                json_finding kind=missing-section file="$file" section="$section" \
                    detail="orchestrator SKILL.md missing Phase/Helper block" >&2
                violations=$((violations + 1))
            fi
        done
    fi
done

# 2. Scripts must be executable + have a bash shebang.
mapfile -t SCRIPT_FILES < <(find scripts -name '*.sh' -type f 2>/dev/null | sort)
log_info "checking ${#SCRIPT_FILES[@]} scripts/*.sh files"
for script in "${SCRIPT_FILES[@]}"; do
    if [[ ! -x "$script" ]]; then
        json_finding kind=not-executable file="$script" \
            detail="chmod +x missing" >&2
        violations=$((violations + 1))
    fi
    if ! head -n1 "$script" | grep -qE '^#!/usr/bin/env (bash|sh)'; then
        json_finding kind=missing-shebang file="$script" \
            detail="first line must be a bash/sh shebang" >&2
        violations=$((violations + 1))
    fi
done

# 3. Cross-reference resolution. `$skill-name` strings in SKILL.md
# bodies AND in AGENTS.md's trigger map should resolve to an actual
# skill in SKILL_NAMES. The trigger map is the public-facing front
# door for the skill set, so a typo or rename there is exactly the
# class of drift the meta-verifier exists to catch.
log_info "checking \$skill-name cross-references"
XREF_FILES=("${SKILL_FILES[@]}")
[[ -f AGENTS.md ]] && XREF_FILES+=("AGENTS.md")
for file in "${XREF_FILES[@]}"; do
    while IFS= read -r ref; do
        # Strip the leading `$`. Skip common false positives like
        # `$1`, `$_`, `$skill-name` (literal placeholder), `$ARGS`,
        # and lowercase shell-variable names that would otherwise
        # produce noise (require at least one hyphen — every real
        # Limn skill name is kebab-cased).
        name="${ref#\$}"
        case "$name" in
            [0-9]*|_*|skill-name|ARGS|*\(*|*\.*|*\}*|*\)*) continue ;;
        esac
        # Real skill IDs always contain a hyphen (e.g.
        # `roundtrip-check`). Bare lowercase tokens like `$file`,
        # `$dir`, `$repo_root` are shell variables — skip them.
        case "$name" in
            *-*) ;;
            *) continue ;;
        esac
        if [[ -z "${SKILL_NAMES[$name]:-}" ]]; then
            json_finding kind=broken-skill-ref file="$file" ref="$ref" \
                detail="\$skill-name does not match any directory under .skillshare/skills/" >&2
            violations=$((violations + 1))
        fi
    done < <(grep -oE '\$[a-z][a-z0-9-]+' "$file" 2>/dev/null \
             | sort -u || true)
done

# 4. Helper script path resolution — scoped to the `## Helper`
# block. The previous implementation grepped the whole file, which
# also accepted unrelated path mentions in prose and gave a free
# pass to Helper blocks that said "no script" while a real script
# existed. Now we extract the section from `^## Helper` to the next
# `^## ` heading and validate inside that range only.
log_info "checking Helper script path references"
for file in "${SKILL_FILES[@]}"; do
    helper_body=$(awk '
        /^## Helper/   { in_helper = 1; next }
        in_helper && /^## / { in_helper = 0 }
        in_helper { print }
    ' "$file")
    while IFS= read -r path; do
        [[ -z "$path" ]] && continue
        clean="${path//\`/}"
        if [[ ! -e "$clean" ]]; then
            json_finding kind=missing-helper-script file="$file" \
                path="$clean" \
                detail="Helper block names a script that does not exist" >&2
            violations=$((violations + 1))
        fi
    done < <(grep -oE 'scripts/[a-z]+/[a-z0-9_-]+\.sh' <<<"$helper_body" \
             | sort -u || true)
done

if [[ "$violations" -eq 0 ]]; then
    log_ok "skills-self-check clean: ${#SKILL_FILES[@]} skills, ${#SCRIPT_FILES[@]} scripts verified"
    exit 0
fi

log_error "skills-self-check failed: $violations violation(s)"
exit 1
