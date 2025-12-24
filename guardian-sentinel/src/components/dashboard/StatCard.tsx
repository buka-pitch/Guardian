import React from "react";
import { getSeverityColor } from "../../utils/formatting";

interface StatCardProps {
  title: string;
  value: number | string;
  severity?: string;
}

export const StatCard: React.FC<StatCardProps> = ({
  title,
  value,
  severity,
}) => {
  return (
    <div className="stat-card">
      <h3>{title}</h3>
      <p
        className="stat-value"
        style={severity ? { color: getSeverityColor(severity) } : {}}
      >
        {value}
      </p>
    </div>
  );
};
