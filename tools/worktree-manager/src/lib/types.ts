export interface WorktreeInfo {
  path: string;
  branch: string;
  commit: string;
  is_bare: boolean;
}

export interface SessionInfo {
  worktree_path: string;
  branch: string;
  state: "Working" | "Question" | "Idle" | "Merged";
  last_activity: string;
  question_text: string | null;
}

export interface ActivityEntry {
  timestamp: string;
  activity_type: "ToolUse" | "Message" | "Error" | "Notification" | "Start" | "Stop";
  summary: string;
  details: string | null;
}

export interface GitStatus {
  modified: string[];
  added: string[];
  deleted: string[];
  untracked: string[];
}
