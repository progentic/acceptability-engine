import type {
  AttemptGateDetail,
  AttemptSummary,
  ContractSubmission,
  EvidenceBundleSummary,
  ReviewDecisionRequest,
  ReviewDecisionResponse,
  RunProgressEvent,
  RunDetail,
  RunListItem,
  RunStatus,
  RunStatusSummary,
  SubmitResponse,
} from "./models";

export class ApiClient {
  constructor(private readonly baseUrl: string) {}

  async listRuns(status?: RunStatus): Promise<RunListItem[]> {
    const query = new URLSearchParams({ limit: "100", offset: "0" });
    if (status) {
      query.set("status", status);
    }
    return this.getJson<RunListItem[]>(`/runs?${query.toString()}`);
  }

  async submitContract(contract: ContractSubmission): Promise<SubmitResponse> {
    return this.postJson<SubmitResponse>("/runs", contract);
  }

  async getRun(runId: number): Promise<RunStatusSummary> {
    return this.getJson<RunStatusSummary>(`/runs/${runId}`);
  }

  async listAttempts(runId: number): Promise<AttemptSummary[]> {
    return this.getJson<AttemptSummary[]>(`/runs/${runId}/attempts`);
  }

  async listAttemptGates(attemptId: number): Promise<AttemptGateDetail[]> {
    return this.getJson<AttemptGateDetail[]>(`/attempts/${attemptId}/gates`);
  }

  async listEvidence(runId: number): Promise<EvidenceBundleSummary[]> {
    return this.getJson<EvidenceBundleSummary[]>(`/runs/${runId}/evidence`);
  }

  async approveReview(
    runId: number,
    request: ReviewDecisionRequest,
  ): Promise<ReviewDecisionResponse> {
    return this.postJson<ReviewDecisionResponse>(
      `/runs/${runId}/review/approve`,
      request,
    );
  }

  async rejectReview(
    runId: number,
    request: ReviewDecisionRequest,
  ): Promise<ReviewDecisionResponse> {
    return this.postJson<ReviewDecisionResponse>(
      `/runs/${runId}/review/reject`,
      request,
    );
  }

  async getRunDetail(runId: number): Promise<RunDetail> {
    const summary = await this.getRun(runId);
    const attempts = await this.listAttempts(runId);
    const gates = await this.listLatestAttemptGates(attempts);
    const evidence = await this.listEvidence(runId);
    return { summary, attempts, gates, evidence };
  }

  openRunProgress(runId: number, after?: number): WebSocket {
    const query = new URLSearchParams();
    if (after !== undefined) {
      query.set("after", String(after));
    }
    const suffix = query.toString() ? `?${query.toString()}` : "";
    return new WebSocket(this.websocketUrl(`/runs/${runId}/progress${suffix}`));
  }

  private async listLatestAttemptGates(
    attempts: AttemptSummary[],
  ): Promise<AttemptGateDetail[]> {
    const latestAttempt = attempts.at(-1);
    if (!latestAttempt) {
      return [];
    }
    return this.listAttemptGates(latestAttempt.attempt_id);
  }

  private async getJson<T>(path: string): Promise<T> {
    const response = await fetch(this.url(path));
    return readJson<T>(response);
  }

  private async postJson<T>(path: string, payload: unknown): Promise<T> {
    const response = await fetch(this.url(path), {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    return readJson<T>(response);
  }

  private url(path: string): string {
    return `${this.baseUrl}${path}`;
  }

  private websocketUrl(path: string): string {
    const url = new URL(this.url(path), window.location.href);
    url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
    return url.toString();
  }
}

export function parseRunProgressEvent(payload: string): RunProgressEvent {
  return JSON.parse(payload) as RunProgressEvent;
}

async function readJson<T>(response: Response): Promise<T> {
  if (response.ok) {
    return response.json() as Promise<T>;
  }
  throw new Error(await response.text());
}
