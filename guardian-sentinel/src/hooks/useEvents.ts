import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { LogEvent, AppStats } from "../types";
import { EventService } from "../services/eventService";

export const useEvents = () => {
  const [events, setEvents] = useState<LogEvent[]>([]);
  const [stats, setStats] = useState<AppStats | null>(null);
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(1);
  const [limit] = useState(50);
  const [total, setTotal] = useState(0); // Approximate from stats

  useEffect(() => {
    loadData();

    // Listen for real-time events
    const unlisten = listen<LogEvent>("log-event", (event) => {
      // Only prepend if we are on the first page and not searching
      if (page === 1 && !query) {
        setEvents((prev) => [event.payload, ...prev].slice(0, limit));
      }
      loadStats();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [page, query, limit]); // Reload when these change

  const loadData = async () => {
    await Promise.all([loadEvents(), loadStats()]);
  };

  const loadEvents = async () => {
    try {
      const offset = (page - 1) * limit;
      let data: LogEvent[] = [];

      if (query) {
        data = await EventService.searchEvents(query, undefined, limit, offset);
      } else {
        // Use search with empty query to support pagination if getRecentEvents doesn't
        // Or assume getRecentEvents supports limit, but not offset?
        // Let's use searchEvents with empty query for consistent pagination
        data = await EventService.searchEvents("", undefined, limit, offset);
      }
      setEvents(data);
    } catch (error) {
      console.error("Failed to load events:", error);
    }
  };

  const loadStats = async () => {
    try {
      const data = await EventService.getStats();
      setStats(data);
      if (data && data.total) {
        setTotal(data.total);
      }
    } catch (error) {
      console.error("Failed to load stats:", error);
    }
  };

  return {
    events,
    stats,
    loadEvents,
    loadStats,
    query,
    setQuery,
    page,
    setPage,
    limit,
    total,
  };
};
