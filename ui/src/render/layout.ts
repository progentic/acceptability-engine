import { setHtml } from "../dom";

export function renderLayout(): void {
  setHtml(
    "app",
    `
      <main class="app-shell">
        <header class="topbar">
          <div>
            <p class="eyebrow">Acceptability Engine</p>
            <h1>Evidence Control Plane</h1>
          </div>
          <div class="api-control">
            <label for="api-base">API</label>
            <input id="api-base" value="/api" autocomplete="off" />
            <button id="connect-api" class="icon-button" title="Reconnect">↻</button>
          </div>
        </header>

        <section class="metrics-row" id="metrics-row"></section>

        <section class="workspace-grid">
          <aside class="run-column">
            <div class="panel-head">
              <h2>Runs</h2>
              <select id="status-filter" aria-label="Status filter">
                <option value="ALL">All</option>
                <option value="QUEUED">Queued</option>
                <option value="RUNNING">Running</option>
                <option value="PENDING_HUMAN_REVIEW">Human review</option>
                <option value="APPROVED">Approved</option>
                <option value="REJECTED">Rejected</option>
                <option value="FAILED_INTERNAL">Failed internal</option>
              </select>
            </div>
            <div id="run-list" class="run-list"></div>
          </aside>

          <section class="detail-column">
            <div id="run-detail"></div>
          </section>

          <aside class="side-column">
            <section class="panel">
              <div class="panel-head">
                <h2>Submit</h2>
              </div>
              <form id="submit-form" class="submit-form">
                <input id="contract-id" placeholder="contract id" required />
                <input id="repo-url" placeholder="repo url" required />
                <input id="base-sha" placeholder="base sha" required />
                <input id="scopes" placeholder="scopes, comma-separated" required />
                <label class="check-row">
                  <input id="human-review" type="checkbox" />
                  <span>Requires human review</span>
                </label>
                <button type="submit">Submit run</button>
              </form>
            </section>

            <section class="panel">
              <div class="panel-head">
                <h2>Review Queue</h2>
              </div>
              <div id="review-queue" class="review-queue"></div>
            </section>
          </aside>
        </section>

        <footer class="status-line">
          <span id="connection-state">Idle</span>
          <span id="last-updated">Never updated</span>
        </footer>
      </main>
    `,
  );
}
