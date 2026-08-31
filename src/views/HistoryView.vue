<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useStore } from "../store";
import { useI18n } from "../i18n";
import type { DictKey } from "../i18n";
import RiskBadge from "../components/RiskBadge.vue";
import type { LedgerEntry, Decision } from "../types";

const { state, refreshLedger, setView } = useStore();
const { t, tf, lang } = useI18n();

function back() { setView("dashboard"); }
onMounted(() => refreshLedger());

type ActivityTab = "all" | "allowed" | "asked" | "blocked";
type TabDef = { key: ActivityTab; labelKey: DictKey };
const TABS: TabDef[] = [
  { key: "all",     labelKey: "activity.tabs.all" },
  { key: "allowed", labelKey: "activity.tabs.allowed" },
  { key: "asked",   labelKey: "activity.tabs.asked" },
  { key: "blocked", labelKey: "activity.tabs.blocked" },
];
const activeTab = ref<ActivityTab>("all");
const search = ref("");
const filterOpen = ref(false);

const PAGE_SIZE = 31;
const page = ref(1);

const filtered = computed(() => {
  let items = state.ledger.slice();
  if (activeTab.value === "allowed") items = items.filter((x) => x.decision === "allow" as Decision);
  if (activeTab.value === "blocked") items = items.filter((x) => x.decision === "deny" as Decision);
  if (activeTab.value === "asked")   items = items.filter((x) => x.decision === "ask" as Decision);
  const q = search.value.trim().toLowerCase();
  if (q) items = items.filter((x) =>
    (x.target ?? "").toLowerCase().includes(q) ||
    (x.category ?? "").toLowerCase().includes(q));
  return items;
});

const paged = computed(() => {
  const start = (page.value - 1) * PAGE_SIZE;
  return filtered.value.slice(start, start + PAGE_SIZE);
});
const pages = computed(() => Math.max(1, Math.ceil(filtered.value.length / PAGE_SIZE)));

function goPage(n: number) {
  if (n < 1 || n > pages.value) return;
  page.value = n;
}
function pageWindow(): (number | "…")[] {
  const p = pages.value, c = page.value;
  if (p <= 7) return Array.from({ length: p }, (_, i) => i + 1);
  const out: (number | "…")[] = [1];
  if (c > 3) out.push("…");
  for (let i = Math.max(2, c - 1); i <= Math.min(p - 1, c + 1); i++) out.push(i);
  if (c < p - 2) out.push("…");
  out.push(p);
  return out;
}

function fmtTime(ts: string): string {
  try {
    const d = new Date(ts.replace(" ", "T"));
    return d.toLocaleTimeString(lang.value === "zh" ? "zh-CN" : undefined, {
      hour: "2-digit", minute: "2-digit", second: "2-digit", hour12: false,
    });
  } catch { return ts; }
}
function fmtDate(ts: string): string {
  try {
    const d = new Date(ts.replace(" ", "T"));
    return d.toLocaleDateString(lang.value === "zh" ? "zh-CN" : undefined, {
      month: "short", day: "numeric",
    });
  } catch { return ""; }
}
function resultClass(e: Readonly<LedgerEntry>): string {
  if (e.decision === "allow") return "res-allow";
  if (e.decision === "deny")  return "res-block";
  return "res-ask";
}
function resultKey(e: Readonly<LedgerEntry>): DictKey {
  if (e.decision === "allow") return "decision.allow";
  if (e.decision === "deny")  return "decision.deny";
  return "decision.ask";
}
function sourceLabel(e: Readonly<LedgerEntry>): string {
  return (e as any).agent || (e as any).source || "Terminal";
}
function rowMenuOpen(id: string): boolean { return rowMenu.value === id; }
const rowMenu = ref<string | null>(null);
function toggleRowMenu(e: Event, id: string) {
  e.stopPropagation();
  rowMenu.value = rowMenu.value === id ? null : id;
}
</script>

<template>
  <div class="view-shell">
    <div class="page-header">
      <button class="back-btn" @click="back">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="back-ico"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
        {{ t("page.back") }}
      </button>
      <div class="header-meta">
        <h1 class="page-title">{{ t("activity.title") }}</h1>
        <p class="page-desc">{{ t("activity.desc") }}</p>
      </div>
    </div>

    <div class="card activity-card">
      <div class="card-head">
        <div class="tabs">
          <button
            v-for="tb in TABS"
            :key="tb.key"
            class="tab"
            :class="{ active: activeTab === tb.key }"
            @click="activeTab = tb.key; page = 1"
          >
            <span class="tab-dot" :class="tb.key" />
            {{ t(tb.labelKey) }}
          </button>
        </div>
        <div class="tools-row">
          <div class="tool-wrap" :class="{ open: filterOpen }">
            <button class="btn btn-ghost btn-sm" @click="filterOpen = !filterOpen">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14"><polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/></svg>
              {{ t("activity.filter") }}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="10" height="10"><polyline points="6 9 12 15 18 9"/></svg>
            </button>
            <div v-if="filterOpen" class="filter-dd">
            </div>
          </div>
          <div class="search-wrap">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="s-ico"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
            <input v-model="search" class="search-input" :placeholder="t('activity.search')" />
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="filtered.length === 0" class="empty-state">
        <div class="e-ico-wrap">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" class="e-ico"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
        </div>
        <div class="e-k">{{ t("empty.noActivity.k") }}</div>
        <p class="e-v">{{ t("empty.noActivity.v") }}</p>
      </div>

      <div v-else class="tbl-wrap">
        <table class="act-tbl">
          <thead>
            <tr>
              <th class="col-menu">{{ t("activity.col.menu") }}</th>
              <th class="col-time">{{ t("activity.col.time") }}</th>
              <th class="col-act">{{ t("activity.col.action") }}</th>
              <th class="col-src">{{ t("activity.col.source") }}</th>
              <th class="col-risk">{{ t("activity.col.risk") }}</th>
              <th class="col-res">{{ t("activity.col.result") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(e, idx) in paged" :key="(e as any).id || idx" class="row" @click="rowMenu = null">
              <td class="col-menu">
                <button class="btn-mini-menu" :class="{open: rowMenuOpen(String((e as any).id || idx))}" @click.stop="toggleRowMenu($event, String((e as any).id || idx))">
                  <svg viewBox="0 0 24 24" fill="currentColor" width="13" height="13"><circle cx="5" cy="12" r="1.8"/><circle cx="12" cy="12" r="1.8"/><circle cx="19" cy="12" r="1.8"/></svg>
                </button>
                <div v-if="rowMenuOpen(String((e as any).id || idx))" class="row-dd" @click.stop">
                </div>
              </td>
              <td class="col-time">
                <span class="t-time">{{ fmtTime(e.timestamp) }}</span>
                <span class="t-date">{{ fmtDate(e.timestamp) }}</span>
              </td>
              <td class="col-act">
                <div class="act-main">
                  <RiskBadge :level="e.risk" size="md" class="act-rb" />
                  <span class="act-target mono">{{ e.target || "—" }}</span>
                </div>
                <div class="act-cat">{{ e.category }}</div>
              </td>
              <td class="col-src">
                <span class="src-pill">{{ sourceLabel(e) }}</span>
              </td>
              <td class="col-risk">
                <RiskBadge :level="e.risk" size="md" />
              </td>
              <td class="col-res">
                <span class="res" :class="resultClass(e)">
                  <svg v-if="e.decision === 'allow'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" width="11" height="11"><polyline points="20 6 9 17 4 12"/></svg>
                  <svg v-else-if="e.decision === 'deny'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" width="11" height="11"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" width="11" height="11"><circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
                  {{ t(resultKey(e)) }}
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="filtered.length > 0" class="foot">
        <div class="foot-k">{{ tf("activity.showing", { shown: String(paged.length), total: String(filtered.length) }) }}</div>
        <div class="pager">
          <button class="pg-btn" :disabled="page === 1" @click="goPage(page - 1)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" width="13" height="13"><polyline points="15 18 9 12 15 6"/></svg>
          </button>
          <template v-for="(p, i) in pageWindow()" :key="i">
            <span v-if="p === '…'" class="pg-ell">…</span>
            <button v-else class="pg-btn" :class="{active: p === page}" @click="goPage(p as number)">{{ p }}</button>
          </template>
          <button class="pg-btn" :disabled="page === pages" @click="goPage(page + 1)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" width="13" height="13"><polyline points="9 18 15 12 9 6"/></svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.view-shell {
  max-width: 1280px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 20px 24px 28px;
}
.page-header { display: flex; flex-direction: column; gap: 10px; }
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

.activity-card { padding: 14px 16px 16px; }
.card-head {
  display: flex; align-items: center; justify-content: space-between;
  gap: 16px; flex-wrap: wrap; padding: 4px 2px 12px; margin-bottom: 6px;
  border-bottom: 1px solid var(--border-soft);
}
.tabs {
  display: inline-flex; align-items: center;
  background: rgba(255,255,255,0.025); border: 1px solid var(--border);
  padding: 4px; border-radius: 10px; gap: 2px;
}
.tab {
  background: transparent; border: none; padding: 6px 12px 6px 10px;
  border-radius: 7px; font-size: 12px; font-weight: 600; color: var(--text-dim);
  cursor: pointer; font-family: var(--sans); transition: all 0.15s;
  display: inline-flex; align-items: center; gap: 7px;
}
.tab:hover { color: var(--text); }
.tab.active { background: var(--bg-card); color: var(--text); box-shadow: 0 0 0 1px var(--border); }
.tab-dot { width: 7px; height: 7px; border-radius: 50%; background: #94a3b8; }
.tab-dot.allowed { background: var(--green); }
.tab-dot.blocked { background: #ef4444; }
.tab-dot.asked   { background: #eab308; }
.tab-dot.all     { background: #cbd5e1; }

.tools-row { display: inline-flex; align-items: center; gap: 10px; }
.tool-wrap { position: relative; }
.btn-sm { padding: 6px 11px; font-size: 12px; }
.filter-dd {
  position: absolute; z-index: 900; right: 0; top: calc(100% + 6px);
  min-width: 180px; background: var(--bg-card); border: 1px solid var(--border);
  border-radius: 10px; padding: 10px 12px; box-shadow: 0 12px 30px rgba(0,0,0,0.45);
  font-size: 12px; color: var(--text-dim);
}
.ph-dd { font-size: 12px; }
.search-wrap { position: relative; }
.s-ico { position: absolute; left: 10px; top: 50%; transform: translateY(-50%); width: 13px; height: 13px; color: var(--text-faint); }
.search-input {
  width: 240px; background: rgba(255,255,255,0.025); border: 1px solid var(--border);
  color: var(--text); font-size: 12px; padding: 7px 11px 7px 30px; border-radius: 8px;
  font-family: var(--sans); transition: border-color 0.15s;
}
.search-input:focus { outline: none; border-color: rgba(163,230,53,0.45); }
.search-input::placeholder { color: var(--text-faint); }

/* Empty */
.empty-state {
  padding: 48px 20px; display: flex; flex-direction: column;
  align-items: center; gap: 10px; text-align: center;
}
.e-ico-wrap {
  width: 56px; height: 56px; border-radius: 14px;
  background: rgba(163,230,53,.07); color: var(--green);
  display: grid; place-items: center; margin-bottom: 4px;
}
.e-ico { width: 26px; height: 26px; }
.e-k { font-size: 14px; font-weight: 700; color: var(--text); }
.e-v { font-size: 12px; color: var(--text-dim); line-height: 1.5; margin: 0; max-width: 400px; }

/* Table */
.tbl-wrap { overflow-x: auto; margin-top: 8px; }
.act-tbl { width: 100%; border-collapse: collapse; font-size: 12.5px; }
.act-tbl thead th {
  position: sticky; top: 0; text-align: left;
  background: rgba(7,9,13,0.96); color: var(--text-faint);
  font-weight: 600; font-size: 10.5px; letter-spacing: 0.5px;
  text-transform: uppercase; padding: 10px 12px; border-bottom: 1px solid var(--border-soft);
}
.act-tbl tbody td { padding: 11px 12px; border-bottom: 1px solid var(--border-soft); vertical-align: top; }
.act-tbl tbody tr.row:hover { background: rgba(255,255,255,0.02); cursor: default; }
.act-tbl tbody tr.row:last-child td { border-bottom: none; }
.col-menu { width: 38px; padding-left: 10px !important; }
.col-time { width: 110px; white-space: nowrap; }
.col-src  { width: 140px; }
.col-risk { width: 108px; }
.col-res  { width: 130px; }

.btn-mini-menu {
  background: transparent; border: none; padding: 4px; border-radius: 6px;
  color: var(--text-faint); cursor: pointer; display: grid; place-items: center; transition: all 0.12s;
}
.btn-mini-menu:hover, .btn-mini-menu.open { background: rgba(255,255,255,0.05); color: var(--text); }
.row-dd {
  position: absolute; z-index: 50; margin-top: 4px; left: -8px;
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: 8px; padding: 6px; min-width: 150px;
  box-shadow: 0 10px 24px rgba(0,0,0,0.45);
}
.dd-item { padding: 6px 9px; border-radius: 5px; font-size: 12px; color: var(--text-faint); }
.t-time { display: block; color: var(--text); font-family: var(--mono); font-weight: 600; font-size: 12px; }
.t-date { display: block; color: var(--text-faint); font-size: 10.5px; margin-top: 2px; }
.act-main { display: flex; align-items: center; gap: 8px; }
.act-rb { flex-shrink: 0; }
.act-target { color: var(--text); font-size: 12.5px; word-break: break-all; font-weight: 500; }
.act-cat { margin-top: 3px; color: var(--text-faint); font-size: 10.5px; text-transform: uppercase; letter-spacing: 0.4px; font-family: var(--mono); }
.src-pill {
  display: inline-flex; align-items: center; padding: 3px 8px;
  background: rgba(148,163,184,0.08); border: 1px solid var(--border);
  border-radius: 6px; color: var(--text-dim); font-size: 11px; font-weight: 600;
}
.res {
  display: inline-flex; align-items: center; gap: 6px; padding: 3px 9px 3px 7px;
  border-radius: 7px; font-size: 11.5px; font-weight: 700;
  letter-spacing: 0.2px; font-family: var(--mono);
}
.res.res-allow { background: rgba(163,230,53,.10); color: var(--green); border: 1px solid rgba(163,230,53,.28); }
.res.res-block { background: rgba(239,68,68,.10);  color: #f87171; border: 1px solid rgba(239,68,68,.28); }
.res.res-ask   { background: rgba(234,179,8,.10);  color: #eab308; border: 1px solid rgba(234,179,8,.28); }

/* Footer pager */
.foot {
  display: flex; align-items: center; justify-content: space-between;
  gap: 12px; padding: 12px 6px 2px; color: var(--text-faint); font-size: 12px;
}
.pager { display: inline-flex; align-items: center; gap: 4px; }
.pg-btn {
  min-width: 28px; height: 28px; background: transparent; border: 1px solid var(--border);
  color: var(--text-dim); border-radius: 7px; padding: 0 8px; font-size: 12px; font-weight: 600;
  cursor: pointer; font-family: var(--sans); transition: all 0.12s;
  display: inline-flex; align-items: center; justify-content: center;
}
.pg-btn:hover:not(:disabled) { color: var(--text); border-color: rgba(163,230,53,0.3); background: rgba(163,230,53,0.04); }
.pg-btn:disabled { opacity: 0.35; cursor: not-allowed; }
.pg-btn.active {
  color: #0d1604; background: linear-gradient(180deg, var(--green), var(--green-soft));
  border-color: rgba(163,230,53,0.55); font-weight: 800;
}
.pg-ell { color: var(--text-faint); padding: 0 4px; font-size: 12px; }
.mono { font-family: var(--mono); }
</style>
