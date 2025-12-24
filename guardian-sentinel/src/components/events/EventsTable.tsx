import React from "react";
import { LogEvent } from "../../types";
import { getSeverityColor, formatDate } from "../../utils/formatting";

interface EventsTableProps {
  events: LogEvent[];
}

export const EventsTable: React.FC<EventsTableProps> = ({ events }) => {
  return (
    <div className="events-table-container">
      <table>
        <thead>
          <tr>
            <th>Severity</th>
            <th>Type</th>
            <th>Message/Details</th>
            <th>Host</th>
            <th>Time</th>
            <th>Rule</th>
          </tr>
        </thead>
        <tbody>
          {events.map((event) => (
            <tr key={event.id}>
              <td>
                <span
                  className="severity-badge"
                  style={{
                    backgroundColor: getSeverityColor(event.severity),
                    color: "#fff",
                  }}
                >
                  {event.severity}
                </span>
              </td>
              <td style={{ textTransform: "capitalize" }}>
                {(event.type || "Unknown").replace("_", " ")}
              </td>
              <td className="details-cell">
                <EventDetails event={event} />
              </td>
              <td>{event.hostname}</td>
              <td className="timestamp-cell">{formatDate(event.timestamp)}</td>
              <td>
                {event.rule_triggered && (
                  <span className="rule-badge" title={event.rule_name}>
                    🚨 Triggered
                  </span>
                )}
              </td>
            </tr>
          ))}
          {events.length === 0 && (
            <tr>
              <td
                colSpan={6}
                style={{
                  textAlign: "center",
                  padding: "2rem",
                  color: "#71717a",
                }}
              >
                No events found
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
};

const EventDetails: React.FC<{ event: LogEvent }> = ({ event }) => {
  if (event.type === "file_integrity") {
    return (
      <span className="event-detail">
        <span className="op">{event.operation}</span> on{" "}
        <span className="path">{event.path}</span>
      </span>
    );
  }

  if (event.type === "process_monitor") {
    return (
      <span className="event-detail">
        Process <span className="highlight">{event.name}</span> (PID:{" "}
        {event.pid}) using <span className="warn">{event.cpu_usage}% CPU</span>
      </span>
    );
  }

  if (event.type === "network_socket") {
    return (
      <span className="event-detail">
        {event.protocol.toUpperCase()} {event.state}:{" "}
        <span className="highlight">{event.local_addr}</span> →{" "}
        {event.remote_addr || "unknown"}
      </span>
    );
  }

  if (event.type === "system_log") {
    return (
      <span className="event-detail">
        [{event.source}] {event.message}
      </span>
    );
  }

  return <span className="raw-json">Unknown event type</span>;
};
