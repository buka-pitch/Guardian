import { useState } from "react";
import { useEvents } from "./hooks/useEvents";
import { Sidebar } from "./components/layout/Sidebar";

import { MonitoringDashboard } from "./components/dashboard/MonitoringDashboard";
import { EventsTable } from "./components/events/EventsTable";
import { SearchBar } from "./components/common/SearchBar";
import { Pagination } from "./components/common/Pagination";
import "./components/layout/Sidebar.css";
import "./components/events/EventsTable.css";
import "./components/common/SearchBar.css";
import "./components/common/Pagination.css";
import "./App.css";

function App() {
  const { events, query, setQuery, page, setPage, limit, total } = useEvents();
  const [currentView, setView] = useState("dashboard");

  return (
    <div className="app-container">
      <Sidebar currentView={currentView} setView={setView} />

      <main className="main-content">
        <header className="page-header">
          <h2>
            {currentView === "dashboard" ? "Dashboard Overview" : "Events Feed"}
          </h2>

          {currentView === "events" && (
            <div className="header-search">
              <SearchBar
                value={query}
                onChange={(q) => {
                  setQuery(q);
                  setPage(1);
                }}
              />
            </div>
          )}

          <div className="header-actions">
            <span className="live-badge">● Live</span>
          </div>
        </header>

        <div className="content-scroll">
          {currentView === "dashboard" && (
            <>
              <MonitoringDashboard />
              <div className="recent-events-section">
                <h3>Recent Activity</h3>
                <EventsTable events={events.slice(0, 10)} />
              </div>
            </>
          )}

          {currentView === "events" && (
            <div className="events-view">
              <EventsTable events={events} />
              <Pagination
                page={page}
                total={total}
                limit={limit}
                onPageChange={setPage}
              />
            </div>
          )}

          {currentView !== "dashboard" && currentView !== "events" && (
            <div className="placeholder-view">
              <h3>
                {currentView.charAt(0).toUpperCase() + currentView.slice(1)}
              </h3>
              <p>Module under development.</p>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}

export default App;
