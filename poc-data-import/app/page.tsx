"use client";

import { useMemo, useState } from "react";

const stages = [
  "Received",
  "Quality check",
  "Preparing",
  "Processing",
  "Catalogue review",
  "Published",
] as const;

type ImportState = "On track" | "Queued" | "Action needed" | "Ready to review" | "Published";

type ImportRecord = {
  id: string;
  name: string;
  customer: string;
  submitted: string;
  updated: string;
  stage: number;
  progress: number;
  items: number;
  status: ImportState;
  detail: string;
  accent: "amber" | "mint" | "cyan" | "coral";
};

const imports: ImportRecord[] = [
  {
    id: "IMP-2048",
    name: "Morning pastry collection",
    customer: "Northstar Bakehouse",
    submitted: "Today, 09:42",
    updated: "2 min ago",
    stage: 2,
    progress: 46,
    items: 42,
    status: "On track",
    detail: "Standardising names, weights, and image formats",
    accent: "amber",
  },
  {
    id: "IMP-2047",
    name: "Artisan bread range",
    customer: "Hearth & Field",
    submitted: "Today, 08:15",
    updated: "6 min ago",
    stage: 3,
    progress: 67,
    items: 86,
    status: "On track",
    detail: "Generating catalogue-ready product records",
    accent: "mint",
  },
  {
    id: "IMP-2044",
    name: "Seasonal tart photography",
    customer: "Mallow & Rye",
    submitted: "Yesterday, 16:20",
    updated: "18 min ago",
    stage: 4,
    progress: 84,
    items: 24,
    status: "Ready to review",
    detail: "Awaiting final catalogue approval",
    accent: "cyan",
  },
  {
    id: "IMP-2042",
    name: "Allergen & nutrition update",
    customer: "Northstar Bakehouse",
    submitted: "Yesterday, 13:08",
    updated: "31 min ago",
    stage: 1,
    progress: 28,
    items: 132,
    status: "Action needed",
    detail: "3 ingredient files need your attention",
    accent: "coral",
  },
  {
    id: "IMP-2049",
    name: "Summer café menu",
    customer: "Flour House Collective",
    submitted: "Today, 10:04",
    updated: "Just now",
    stage: 0,
    progress: 10,
    items: 38,
    status: "Queued",
    detail: "Files received securely and queued for checks",
    accent: "cyan",
  },
  {
    id: "IMP-2039",
    name: "Gluten-free collection",
    customer: "Golden Crumb Co.",
    submitted: "12 Jul, 14:32",
    updated: "12 Jul",
    stage: 5,
    progress: 100,
    items: 48,
    status: "Published",
    detail: "48 products are now live in the catalogue",
    accent: "mint",
  },
];

const stageSummary = [
  { label: "Received", count: 4, note: "Securely queued" },
  { label: "Quality check", count: 5, note: "Files & fields" },
  { label: "Preparing", count: 3, note: "Clean & standardise" },
  { label: "Processing", count: 3, note: "Create records" },
  { label: "Catalogue review", count: 2, note: "Ready for you" },
  { label: "Published", count: 41, note: "This month" },
];

const statusClass: Record<ImportState, string> = {
  "On track": "positive",
  Queued: "neutral",
  "Action needed": "attention",
  "Ready to review": "review",
  Published: "published",
};

function StageRail({ item }: { item: ImportRecord }) {
  return (
    <div className={`stage-rail accent-${item.accent}`} aria-label={`${item.progress}% complete`}>
      <div className="rail-line" aria-hidden="true">
        <span className="rail-fill" style={{ width: `${item.progress}%` }} />
      </div>
      <div className="rail-nodes" aria-hidden="true">
        {stages.map((stage, index) => (
          <span
            className={`rail-node ${index < item.stage ? "complete" : ""} ${index === item.stage ? "current" : ""}`}
            key={stage}
          />
        ))}
      </div>
      <div className="rail-labels" aria-hidden="true">
        {stages.map((stage) => (
          <span key={stage}>{stage}</span>
        ))}
      </div>
    </div>
  );
}

function ImportCard({ item }: { item: ImportRecord }) {
  return (
    <article className="import-card">
      <div className="import-topline">
        <div className="import-identity">
          <div className={`file-mark accent-${item.accent}`} aria-hidden="true">
            <span />
          </div>
          <div>
            <div className="id-row">
              <span>{item.id}</span>
              <span className="dot-separator" />
              <span>{item.items} items</span>
            </div>
            <h3>{item.name}</h3>
            <p>{item.customer}</p>
          </div>
        </div>
        <div className="import-state">
          <span className={`status-pill ${statusClass[item.status]}`}>
            <i aria-hidden="true" />
            {item.status}
          </span>
          <button className="more-button" aria-label={`More options for ${item.name}`} type="button">
            •••
          </button>
        </div>
      </div>

      <StageRail item={item} />

      <div className="import-footer">
        <p>
          <span className="pulse-dot" aria-hidden="true" />
          {item.detail}
        </p>
        <div className="import-time">
          <span>Submitted {item.submitted}</span>
          <span>Updated {item.updated}</span>
        </div>
      </div>
    </article>
  );
}

export default function Home() {
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState("Active");
  const [showUpload, setShowUpload] = useState(false);
  const [toast, setToast] = useState(false);

  const visibleImports = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    return imports.filter((item) => {
      const matchesQuery =
        !normalized ||
        item.name.toLowerCase().includes(normalized) ||
        item.customer.toLowerCase().includes(normalized) ||
        item.id.toLowerCase().includes(normalized);
      const matchesFilter =
        filter === "All" ||
        (filter === "Active" && item.status !== "Published") ||
        (filter === "Needs attention" && item.status === "Action needed") ||
        (filter === "Completed" && item.status === "Published");
      return matchesQuery && matchesFilter;
    });
  }, [filter, query]);

  const notify = () => {
    setShowUpload(false);
    setToast(true);
    window.setTimeout(() => setToast(false), 3200);
  };

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <a className="brand" href="#top" aria-label="Knead dashboard home">
          <span className="brand-mark" aria-hidden="true">
            <i />
            <i />
            <i />
          </span>
          <span>Knead</span>
        </a>

        <nav className="main-nav" aria-label="Primary navigation">
          <a className="nav-item active" href="#top">
            <span className="nav-icon grid-icon" aria-hidden="true"><i /><i /><i /><i /></span>
            Overview
          </a>
          <a className="nav-item" href="#imports">
            <span className="nav-icon transfer-icon" aria-hidden="true">⇅</span>
            Imports
            <span className="nav-count">18</span>
          </a>
          <a className="nav-item" href="#pipeline">
            <span className="nav-icon layers-icon" aria-hidden="true">◇</span>
            Catalogue
          </a>
          <a className="nav-item" href="#activity">
            <span className="nav-icon pulse-icon" aria-hidden="true">⌁</span>
            Activity
          </a>
        </nav>

        <div className="sidebar-bottom">
          <div className="support-card">
            <span className="support-icon" aria-hidden="true">?</span>
            <div>
              <strong>Need a hand?</strong>
              <p>We usually reply in 5 min</p>
            </div>
            <button type="button" aria-label="Contact support">↗</button>
          </div>
          <a className="nav-item" href="#settings">
            <span className="nav-icon" aria-hidden="true">⚙</span>
            Settings
          </a>
          <div className="user-card">
            <span className="avatar">KM</span>
            <div>
              <strong>Kate Miller</strong>
              <span>Northstar Bakehouse</span>
            </div>
            <button type="button" aria-label="Open account menu">⌄</button>
          </div>
        </div>
      </aside>

      <main id="top" className="main-content">
        <header className="topbar">
          <div>
            <div className="eyebrow">
              <span className="live-dot" aria-hidden="true" />
              Catalogue intake
            </div>
            <h1>Good morning, Kate.</h1>
            <p>Your catalogue imports are moving along nicely.</p>
          </div>
          <div className="topbar-actions">
            <button className="icon-button" type="button" aria-label="Notifications">
              <span className="bell-icon" aria-hidden="true">♢</span>
              <i />
            </button>
            <button className="primary-button" type="button" onClick={() => setShowUpload(true)}>
              <span aria-hidden="true">＋</span>
              New import
            </button>
          </div>
        </header>

        <section className="metric-grid" aria-label="Import overview">
          <article className="metric-card featured">
            <div className="metric-heading">
              <span className="metric-icon warm" aria-hidden="true">⇅</span>
              <span>Active imports</span>
            </div>
            <div className="metric-value-row">
              <strong>18</strong>
              <span className="trend up">↗ 12%</span>
            </div>
            <p>Across 6 catalogue stages</p>
            <div className="spark-bars" aria-hidden="true">
              {[28, 38, 32, 52, 46, 64, 58, 78, 74, 88, 82, 94].map((height, index) => (
                <i key={index} style={{ height: `${height}%` }} />
              ))}
            </div>
          </article>

          <article className="metric-card">
            <div className="metric-heading">
              <span className="metric-icon mint" aria-hidden="true">✓</span>
              <span>On time</span>
            </div>
            <div className="metric-value-row">
              <strong>94<span>%</span></strong>
              <span className="trend up">↗ 3%</span>
            </div>
            <p>Within expected turnaround</p>
            <div className="mini-progress mint" aria-hidden="true"><i /></div>
          </article>

          <article className="metric-card">
            <div className="metric-heading">
              <span className="metric-icon blue" aria-hidden="true">□</span>
              <span>Items processed</span>
            </div>
            <div className="metric-value-row">
              <strong>1,284</strong>
              <span className="trend up">↗ 8%</span>
            </div>
            <p>Catalogue items this month</p>
            <div className="mini-progress blue" aria-hidden="true"><i /></div>
          </article>

          <article className="metric-card attention-card">
            <div className="metric-heading">
              <span className="metric-icon coral" aria-hidden="true">!</span>
              <span>Needs attention</span>
            </div>
            <div className="metric-value-row">
              <strong>3</strong>
              <span className="trend steady">No change</span>
            </div>
            <p>Waiting for customer input</p>
            <button type="button" onClick={() => setFilter("Needs attention")}>Review now <span>→</span></button>
          </article>
        </section>

        <section id="pipeline" className="pipeline-card">
          <div className="section-heading">
            <div>
              <span className="section-kicker">Live workflow</span>
              <h2>Your catalogue pipeline</h2>
              <p>Every file, from hand-off to published product.</p>
            </div>
            <div className="freshness"><span /> Updated just now</div>
          </div>
          <div className="pipeline-flow">
            {stageSummary.map((stage, index) => (
              <div className={`pipeline-stage stage-${index}`} key={stage.label}>
                <div className="stage-orb">
                  <span>{stage.count}</span>
                  {index < stageSummary.length - 1 && <i className="stage-connector" />}
                </div>
                <strong>{stage.label}</strong>
                <small>{stage.note}</small>
              </div>
            ))}
          </div>
        </section>

        <section id="imports" className="imports-section">
          <div className="imports-heading">
            <div>
              <span className="section-kicker">Current work</span>
              <h2>Recent imports</h2>
            </div>
            <div className="imports-tools">
              <label className="search-box">
                <span aria-hidden="true">⌕</span>
                <span className="sr-only">Search imports</span>
                <input
                  value={query}
                  onChange={(event) => setQuery(event.target.value)}
                  placeholder="Search imports"
                  type="search"
                />
              </label>
              <div className="filter-tabs" aria-label="Filter imports">
                {["Active", "Needs attention", "Completed", "All"].map((option) => (
                  <button
                    className={filter === option ? "active" : ""}
                    key={option}
                    onClick={() => setFilter(option)}
                    type="button"
                  >
                    {option}
                  </button>
                ))}
              </div>
            </div>
          </div>

          <div className="import-list">
            {visibleImports.map((item) => <ImportCard item={item} key={item.id} />)}
            {visibleImports.length === 0 && (
              <div className="empty-state">
                <span aria-hidden="true">⌕</span>
                <h3>No imports found</h3>
                <p>Try a different search or filter.</p>
              </div>
            )}
          </div>
        </section>

        <footer id="activity" className="page-footer">
          <span><i /> All systems operational</span>
          <p>Last synchronised just now · Customer view</p>
        </footer>
      </main>

      {showUpload && (
        <div className="modal-backdrop" role="presentation" onMouseDown={() => setShowUpload(false)}>
          <section
            className="upload-modal"
            role="dialog"
            aria-modal="true"
            aria-labelledby="upload-title"
            onMouseDown={(event) => event.stopPropagation()}
          >
            <button className="modal-close" type="button" aria-label="Close" onClick={() => setShowUpload(false)}>×</button>
            <span className="modal-kicker">New catalogue import</span>
            <h2 id="upload-title">Hand us the ingredients.</h2>
            <p>Upload product data, images, or supporting documents. We’ll keep everything together through processing.</p>
            <button className="drop-zone" type="button" onClick={notify}>
              <span className="drop-icon" aria-hidden="true">↑</span>
              <strong>Drop files here or choose from your device</strong>
              <small>CSV, XLSX, ZIP, PDF or images · up to 2 GB</small>
            </button>
            <div className="modal-security"><span aria-hidden="true">◇</span> Files are encrypted in transit and at rest.</div>
          </section>
        </div>
      )}

      <div className={`toast ${toast ? "visible" : ""}`} role="status" aria-live="polite">
        <span aria-hidden="true">✓</span>
        <div><strong>POC import created</strong><small>Your new batch is now in Received.</small></div>
      </div>
    </div>
  );
}
