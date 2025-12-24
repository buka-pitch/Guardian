import { invoke } from "@tauri-apps/api/core";
import { LogEvent } from "../types";

export const EventService = {
  async getRecentEvents(limit: number = 100): Promise<LogEvent[]> {
    return invoke<LogEvent[]>("get_recent_events", { limit });
  },

  async getStats(): Promise<any> {
    return invoke("get_event_stats");
  },

  async searchEvents(
    query: string,
    severity?: string,
    limit: number = 100,
    offset: number = 0
  ): Promise<LogEvent[]> {
    return invoke("search_events", {
      query,
      severity: severity || null,
      limit,
      offset,
    });
  },
};
