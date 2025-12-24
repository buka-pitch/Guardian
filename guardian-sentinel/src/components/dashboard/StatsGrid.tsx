import React from "react";
import { AppStats } from "../../types";
import { StatCard } from "./StatCard";

interface StatsGridProps {
  stats: AppStats | null;
}

export const StatsGrid: React.FC<StatsGridProps> = ({ stats }) => {
  if (!stats) return null;

  return (
    <div className="stats">
      <StatCard title="Total Events" value={stats.total} />
      <StatCard title="Rules Triggered" value={stats.rules_triggered} />

      {stats.by_severity &&
        Object.entries(stats.by_severity).map(([severity, count]) => (
          <StatCard
            key={severity}
            title={severity}
            value={count}
            severity={severity}
          />
        ))}
    </div>
  );
};
