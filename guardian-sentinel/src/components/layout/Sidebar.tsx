import React from "react";

interface SidebarProps {
  currentView: string;
  setView: (view: string) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({ currentView, setView }) => {
  const menuItems = [
    { id: "dashboard", label: "Dashboard", icon: "📊" },
    { id: "events", label: "Events Feed", icon: "📋" },
    { id: "rules", label: "Rules Engine", icon: "🛡️" },
    { id: "settings", label: "Settings", icon: "⚙️" },
  ];

  return (
    <aside className="sidebar">
      <div className="logo-container">
        <div className="logo-icon">🛡️</div>
        <div className="logo-text">
          <h1>Guardian</h1>
          <span>SIEM SENTINEL</span>
        </div>
      </div>

      <nav className="nav-menu">
        {menuItems.map((item) => (
          <button
            key={item.id}
            className={`nav-item ${currentView === item.id ? "active" : ""}`}
            onClick={() => setView(item.id)}
          >
            <span className="icon">{item.icon}</span>
            <span className="label">{item.label}</span>
          </button>
        ))}
      </nav>

      <div className="sidebar-footer">
        <div className="system-status">
          <span className="status-dot"></span>
          System Online
        </div>
        <div className="version">v0.1.0</div>
      </div>
    </aside>
  );
};
