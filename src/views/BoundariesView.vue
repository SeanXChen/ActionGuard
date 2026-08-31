<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useStore } from "../store";
import { useI18n } from "../i18n";
import type { DictKey } from "../i18n";
import { api } from "../api";

const { t } = useI18n();
const { state, setView } = useStore();

function back() { setView("dashboard"); }

interface BoundEntry {
  name: string; path: string; displayName: string; type: string;
  status: "enforced" | "observe-only" | "unsupported"; active: boolean;
}

type BoundClassDef = {
  cls: string;
  nameKey: DictKey;
  descKey: DictKey;
  enforced: boolean;
  observed: boolean;
  noteKey: DictKey;
};

const enforce = ref<{ boundaries: BoundEntry[] } | null>(null);
const loading = ref(false);

function inferDisplayName(path: string): { name: string; type: string } {
  const p = path.toLowerCase();
  if (p.includes("powershell") || p.includes("pwsh"))
    return { name: "Protected Shell (PowerShell)", type: "powershell" };
  if (p.includes("bash") || p.includes("zsh") || p.includes("wsl") || p.includes("git\\bin\\bash"))
    return { name: "Protected Shell (bash)", type: "bash" };
  if (p.includes("codebuddy") || p.includes("pretooluse") || p.includes("cursor") ||
      p.includes("claude") || p.includes("codex") || p.includes("manus") || p.includes("openclaw"))
    return { name: "Tool Hook", type: "hook" };
  if (p.includes("cmd") || p.includes("command"))
    return { name: "Protected Shell (CMD)", type: "cmd" };
  return { name: path, type: "shell" };
}

async function loadEnforcement() {
  loading.value = true;
  console.log("[BoundariesView] loadEnforcement called, loading:", loading.value);
  try {
    const paths = await api.getEnforcementPaths();
    console.log("[BoundariesView] getEnforcementPaths returned:", paths);
    enforce.value = {
      boundaries: paths.map((b) => {
        const dn = inferDisplayName(b.path);
        return {
          name: dn.name, path: b.path, displayName: dn.name, type: dn.type,
          active: true,
          status: b.block ? "enforced" : b.observe ? "observe-only" : "unsupported",
        } as BoundEntry;
      }),
    };
    console.log("[BoundariesView] enforce.value set:", enforce.value);
  } catch (e) {
    console.error("[BoundariesView] loadEnforcement error:", e);
    enforce.value = null;
  }
  finally { loading.value = false; }
}

onMounted(() => { if (state.session) void loadEnforcement(); });

const BOUNDARY_CLASSES: BoundClassDef[] = [
  { cls: "A", nameKey: "boundaries.class.A", descKey: "boundaries.class.A.desc",
    enforced: true, observed: true, noteKey: "boundaries.note.A" },
  { cls: "B", nameKey: "boundaries.class.B", descKey: "boundaries.class.B.desc",
    enforced: true, observed: true, noteKey: "boundaries.note.B" },
  { cls: "C", nameKey: "boundaries.class.C", descKey: "boundaries.class.C.desc",
    enforced: true, observed: true, noteKey: "boundaries.note.C" },
  { cls: "D", nameKey: "boundaries.class.D", descKey: "boundaries.class.D.desc",
    enforced: false, observed: false, noteKey: "boundaries.note.D" },
  { cls: "E", nameKey: "boundaries.class.E", descKey: "boundaries.class.E.desc",
    enforced: false, observed: false, noteKey: "boundaries.note.E" },
  { cls: "F", nameKey: "boundaries.class.F", descKey: "boundaries.class.F.desc",
    enforced: false, observed: false, noteKey: "boundaries.note.F" },
];

const currentPaths = computed(() => enforce.value?.boundaries ?? []);
</script>

<template>
  <div class="view-shell">
    <div class="page-header">
      <button class="back-btn" @click="back">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="back-ico"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
        {{ t("page.back") }}
      </button>
      <div class="header-meta">
        <h1 class="page-title">{{ t("boundaries.title") }}</h1>
        <p class="page-desc">{{ t("boundaries.desc") }}</p>
      </div>
    </div>

    <!-- Boundary Class Reference Table -->
    <div class="card">
      <div class="section-header">
        <div>
          <h2 class="section-title">{{ t("boundaries.class.title") }}</h2>
          <p class="section-desc">{{ t("boundaries.class.desc") }}</p>
        </div>
      </div>

      <div class="tbl-wrap">
        <table class="bc-tbl">
          <thead>
            <tr>
              <th class="col-cls">{{ t("boundaries.col.class") }}</th>
              <th class="col-name">{{ t("boundaries.col.name") }}</th>
              <th class="col-desc">{{ t("boundaries.col.description") }}</th>
              <th class="col-status">{{ t("boundaries.col.enforce") }}</th>
              <th class="col-status">{{ t("boundaries.col.obs") }}</th>
              <th class="col-note">{{ t("boundaries.col.note") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="bc in BOUNDARY_CLASSES.filter(b => b.cls <= 'C')" :key="bc.cls">
              <td class="td-cls">
                <span class="cls-badge" :class="`cls-${bc.cls.toLowerCase()}`">{{ bc.cls }}</span>
              </td>
              <td class="td-name">
                <div class="bc-name">{{ t(bc.nameKey) }}</div>
              </td>
              <td class="td-desc">
                <span class="bc-desc">{{ t(bc.descKey) }}</span>
              </td>
              <td class="td-status">
                <span v-if="bc.enforced" class="status-chip enforced">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round" width="12" height="12"><polyline points="20 6 9 17 4 12"/></svg>
                  {{ t("boundaries.active.yes") }}
                </span>
              </td>
              <td class="td-status">
                <span v-if="bc.observed" class="status-chip observed">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round" width="12" height="12"><polyline points="20 6 9 17 4 12"/></svg>
                  {{ t("boundaries.active.yes") }}
                </span>
              </td>
              <td class="td-note">
                <span class="bc-note">{{ t(bc.noteKey) }}</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Current Execution Paths -->
    <div class="card">
      <div class="section-header">
        <div>
          <h2 class="section-title">{{ t("boundaries.current.title") }}</h2>
        </div>
        <div class="section-actions">
          <button class="btn btn-ghost btn-sm" :disabled="loading" @click="loadEnforcement">
            <svg v-if="!loading" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="13" height="13"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
            <span v-else class="spin-sm"></span>
            {{ t("home.advanced.diagnostics.run") }}
          </button>
          <button class="btn btn-ghost btn-sm" @click="setView('policies')">
            {{ t("sidebar.nav.policies") }}
          </button>
        </div>
      </div>

      <!-- No session -->
      <div v-if="!state.session" class="empty-state">
        <div class="e-ico-wrap">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" class="e-ico"><rect x="4" y="10" width="16" height="11" rx="2"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/><circle cx="12" cy="15.5" r="1.5"/></svg>
        </div>
        <div class="e-k">{{ t("empty.noBoundaries.k") }}</div>
        <p class="e-v">{{ t("empty.noBoundaries.v") }}</p>
      </div>

      <!-- Loading -->
      <div v-else-if="loading && currentPaths.length === 0" class="empty-state">
        <div class="e-ico-wrap">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" class="e-ico"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
        </div>
        <div class="e-k">Loading…</div>
      </div>

      <!-- No paths -->
      <div v-else-if="currentPaths.length === 0" class="empty-state">
        <div class="e-ico-wrap">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" class="e-ico"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><polyline points="9 12 11 14 15 10"/></svg>
        </div>
        <div class="e-k">{{ t("boundaries.current.empty") }}</div>
      </div>

      <!-- Path list -->
      <div v-else class="path-list">
        <div class="path-tbl-header">
          <div class="ph-col ph-path">{{ t("boundaries.current.path") }}</div>
          <div class="ph-col ph-type">Type</div>
          <div class="ph-col ph-status">{{ t("boundaries.current.status") }}</div>
          <div class="ph-col ph-active">Active</div>
        </div>
        <div v-for="(b, i) in currentPaths" :key="i" class="path-row">
          <div class="ph-col ph-path">
            <code class="path-code">{{ b.path }}</code>
          </div>
          <div class="ph-col ph-type">
            <span class="type-badge" :class="b.type">{{ b.type }}</span>
          </div>
          <div class="ph-col ph-status">
            <span class="status-chip" :class="b.status === 'enforced' ? 'enforced' : b.status === 'observe-only' ? 'observed' : 'unsupported'">
              <svg v-if="b.status === 'enforced'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round" width="11" height="11"><polyline points="20 6 9 17 4 12"/></svg>
              <svg v-else-if="b.status === 'observe-only'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="11" height="11"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
              <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="11" height="11"><circle cx="12" cy="12" r="10"/><line x1="4.93" y1="4.93" x2="19.07" y2="19.07"/></svg>
              {{ b.status === "enforced" ? t("boundaries.current.enforced") : b.status === "observe-only" ? t("boundaries.current.obs") : t("boundaries.current.unsupported") }}
            </span>
          </div>
          <div class="ph-col ph-active">
            <span v-if="b.active" class="active-badge">
              <span class="dot-active"></span>
              {{ t("boundaries.current.active") }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.view-shell {
  max-width: 1040px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 20px 24px 28px;
}
.page-header { display: flex; flex-direction: column; gap: 10px; padding: 0 2px; }
.back-btn {
  display: inline-flex; align-items: center; gap: 6px;
  background: transparent; border: 1px solid var(--border); padding: 7px 12px;
  border-radius: 8px; color: var(--text-dim); font-size: 12px; font-weight: 600;
  cursor: pointer; transition: all 0.15s; font-family: var(--sans); width: fit-content;
}
.back-btn:hover { background: rgba(255,255,255,0.03); color: var(--green); border-color: rgba(163,230,53,0.3); }
.back-ico { width: 13px; height: 13px; }
.header-meta { display: flex; flex-direction: column; gap: 4px; }
.page-title { font-size: 22px; font-weight: 800; letter-spacing: 0.2px; color: var(--text); margin: 0; }
.page-desc { color: var(--text-dim); font-size: 12.5px; line-height: 1.55; margin: 0; }

.card {
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: var(--radius); padding: 20px 22px;
}
.section-header {
  display: flex; align-items: flex-start; justify-content: space-between;
  gap: 14px; margin-bottom: 16px;
}
.section-title { font-size: 15px; font-weight: 700; color: var(--text); margin: 0 0 4px; }
.section-desc { font-size: 12px; color: var(--text-dim); margin: 0; line-height: 1.5; }
.section-actions { display: flex; gap: 8px; flex-shrink: 0; }

/* Boundary class table */
.tbl-wrap { overflow-x: auto; }
.bc-tbl { width: 100%; border-collapse: collapse; font-size: 12.5px; }
.bc-tbl thead th {
  text-align: left; padding: 10px 12px;
  background: rgba(255,255,255,0.015); color: var(--text-faint);
  font-weight: 600; font-size: 10.5px; letter-spacing: 0.5px;
  text-transform: uppercase; border-bottom: 1px solid var(--border-soft);
}
.bc-tbl tbody tr { border-bottom: 1px solid rgba(255,255,255,0.03); }
.bc-tbl tbody tr:last-child { border-bottom: none; }
.bc-tbl tbody tr:hover { background: rgba(255,255,255,0.02); }
.bc-tbl tbody td { padding: 13px 12px; vertical-align: top; }
.col-cls    { width: 60px; }
.col-name   { width: 180px; }
.col-status  { width: 130px; }
.col-note    { width: 200px; }

.cls-badge {
  display: inline-flex; align-items: center; justify-content: center;
  width: 28px; height: 28px; border-radius: 8px;
  font-size: 13px; font-weight: 800; font-family: var(--mono);
}
.cls-A { background: rgba(168,85,247,.15); color: #c084fc; border: 1px solid rgba(168,85,247,.3); }
.cls-B { background: rgba(59,130,246,.15); color: #60a5fa; border: 1px solid rgba(59,130,246,.3); }
.cls-C { background: rgba(34,197,94,.15); color: #4ade80; border: 1px solid rgba(34,197,94,.3); }
.cls-D, .cls-E, .cls-F { background: rgba(100,116,139,.1); color: #94a3b8; border: 1px solid rgba(100,116,139,.2); }

.bc-name { font-size: 13px; font-weight: 700; color: var(--text); }
.bc-desc { font-size: 12px; color: var(--text-dim); line-height: 1.5; }

.status-chip {
  display: inline-flex; align-items: center; gap: 5px;
  padding: 3px 9px 3px 7px; border-radius: 999px;
  font-size: 11px; font-weight: 700; white-space: nowrap;
}
.status-chip.enforced    { background: rgba(34,197,94,.12);  color: #4ade80; border: 1px solid rgba(34,197,94,.3); }
.status-chip.observed     { background: rgba(245,158,11,.12); color: var(--amber); border: 1px solid rgba(245,158,11,.28); }
.status-chip.unsupported  { background: rgba(100,116,139,.1); color: #94a3b8; border: 1px solid rgba(100,116,139,.2); }
.status-chip.future      { background: rgba(100,116,139,.08); color: #94a3b8; border: 1px dashed rgba(100,116,139,.3); }

.bc-note { font-size: 11.5px; color: var(--text-faint); }
.future-row td { opacity: 0.7; }

/* Empty state */
.empty-state {
  padding: 40px 16px; display: flex; flex-direction: column;
  align-items: center; gap: 8px; text-align: center;
}
.e-ico-wrap {
  width: 52px; height: 52px; border-radius: 13px;
  background: rgba(163,230,53,.07); color: var(--green);
  display: grid; place-items: center; margin-bottom: 4px;
}
.e-ico { width: 24px; height: 24px; }
.e-k { font-size: 13px; font-weight: 700; color: var(--text); }
.e-v { font-size: 12px; color: var(--text-dim); line-height: 1.5; margin: 0; max-width: 380px; }

/* Path table */
.path-tbl-header {
  display: grid;
  grid-template-columns: 1fr 100px 160px 80px;
  gap: 10px; padding: 8px 12px;
  border-bottom: 1px solid var(--border-soft); margin-bottom: 6px;
}
.ph-col {
  font-size: 10.5px; font-weight: 700;
  text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-faint);
}
.path-row {
  display: grid;
  grid-template-columns: 1fr 100px 160px 80px;
  gap: 10px; align-items: center;
  padding: 10px 12px; border-radius: 9px; transition: background 0.12s;
}
.path-row:hover { background: rgba(255,255,255,0.025); }
.path-code {
  font-family: var(--mono); font-size: 12px; color: var(--text);
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.type-badge {
  display: inline-flex; align-items: center; padding: 2px 8px;
  border-radius: 6px; font-size: 11px; font-weight: 700; text-transform: capitalize;
}
.type-badge.hook      { background: rgba(168,85,247,.12); color: #c084fc; }
.type-badge.powershell { background: rgba(34,197,94,.12); color: #4ade80; }
.type-badge.bash      { background: rgba(59,130,246,.12); color: #60a5fa; }
.type-badge.cmd       { background: rgba(245,158,11,.12); color: var(--amber); }
.type-badge.shell    { background: rgba(100,116,139,.1); color: #94a3b8; }

.active-badge {
  display: inline-flex; align-items: center; gap: 5px;
  font-size: 11px; font-weight: 700; color: var(--green-check);
}
.dot-active {
  width: 7px; height: 7px; border-radius: 50%;
  background: var(--green-check); animation: pulse-dot 1.6s infinite;
}
@keyframes pulse-dot { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }

.spin-sm {
  width: 12px; height: 12px;
  border: 2px solid rgba(255,255,255,0.2); border-top-color: currentColor;
  border-radius: 50%; animation: spin 0.7s linear infinite; display: inline-block;
}
@keyframes spin { to { transform: rotate(360deg); } }
.btn-sm { padding: 6px 12px; font-size: 12px; }
</style>
