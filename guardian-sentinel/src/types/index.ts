export type EventType =
  | { type: "file_integrity"; path: string; operation: string; hash?: string }
  | {
      type: "process_monitor";
      pid: number;
      name: string;
      cpu_usage: number;
      memory_usage: number;
    }
  | {
      type: "network_socket";
      local_addr: string;
      remote_addr?: string;
      protocol: string;
      state: string;
    }
  | { type: "system_log"; source: string; level: string; message: string };

export type LogEvent = {
  id: string;
  timestamp: string;
  severity: string;
  hostname: string;
  tags: string[];
  rule_triggered: boolean;
  rule_name?: string;
} & EventType;

export interface AppStats {
  total: number;
  by_severity: Record<string, number>;
  rules_triggered: number;
}
