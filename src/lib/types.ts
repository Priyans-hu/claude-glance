// Domain types for claude-glance.
// Kept minimal at v0; extended as the watcher and parser land.

export type SessionStatus = "working" | "waiting" | "plan" | "idle";

export interface Session {
  /** UUID, derived from the JSONL filename. */
  id: string;
  /** Working directory of the Claude Code session. */
  cwd: string;
  /** Project name — last segment of cwd, with worktree suffix if applicable. */
  project: string;
  /** Git branch at last hook event. */
  branch: string | null;
  /** First user prompt (truncated) or auto-generated summary. */
  title: string;
  status: SessionStatus;
  /** Tool currently executing, if any. */
  currentTool: string | null;
  /** Permission mode at last event: yolo / auto-edit / plan / read-only. */
  permissionMode: string | null;
  /** Number of active subagents spawned by this session. */
  subagentCount: number;
  /** ISO timestamp of last JSONL event. */
  lastActivity: string;
  /** Aggregate tokens for the session (input + output). */
  tokens: number;
}
