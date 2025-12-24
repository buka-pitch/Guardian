import { useState, useEffect } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  BarChart,
  Bar,
} from "recharts";
import { Activity, Cpu, HardDrive, Shield } from "lucide-react";
import "./MonitoringDashboard.css";
import { listen } from "@tauri-apps/api/event";
import { LogEvent } from "../../types";

interface SystemStats {
  timestamp: string;
  cpu: number;
  memory: number;
}

interface SeverityStats {
  name: string;
  count: number;
}

export const MonitoringDashboard = () => {
  const [systemData, setSystemData] = useState<SystemStats[]>([]);
  const [severityData, setSeverityData] = useState<SeverityStats[]>([
    { name: "Low", count: 0 },
    { name: "Medium", count: 0 },
    { name: "High", count: 0 },
    { name: "Critical", count: 0 },
  ]);
  const [currentCpu, setCurrentCpu] = useState(0);
  const [currentMem, setCurrentMem] = useState(0);

  useEffect(() => {
    // Listen for realtime events
    const unlisten = listen<LogEvent>("realtime-event", (event) => {
      const payload = event.payload;

      // Check if it's a system monitor event
      if (payload.type === "process_monitor" && payload.name === "system") {
        const timestamp = new Date().toLocaleTimeString();
        // @ts-ignore
        const cpu = Math.round(payload.cpu_usage || 0);
        // @ts-ignore
        const memory = Math.round((payload.memory_usage || 0) / 1024 / 1024); // MB

        setCurrentCpu(cpu);
        setCurrentMem(memory);

        setSystemData((prev) => {
          const newData = [...prev, { timestamp, cpu, memory }];
          if (newData.length > 20) return newData.slice(newData.length - 20);
          return newData;
        });
      } else {
         // Update severity counts for other events
         const severity = payload.severity;
         if (severity && severity !== "INFO") {
             setSeverityData(prev => prev.map(item => {
                 if (item.name.toUpperCase() === severity) {
                     return { ...item, count: item.count + 1 };
                 }
                 return item;
             }));
         }
      }
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  return (
    <div className="dashboard-grid">
      {/* Stats Cards */}
      <div className="stat-card">
        <div className="stat-header">
          <Cpu className="icon cpu" />
          <span>CPU Usage</span>
        </div>
        <div className="stat-value">{currentCpu}%</div>
      </div>
      <div className="stat-card">
        <div className="stat-header">
          <Activity className="icon memory" />
          <span>Memory Usage</span>
        </div>
        <div className="stat-value">{currentMem} MB</div>
      </div>
      <div className="stat-card">
        <div className="stat-header">
          <HardDrive className="icon disk" />
          <span>Events processed</span>
        </div>
        <div className="stat-value">{systemData.length}</div>
      </div>
      <div className="stat-card">
        <div className="stat-header">
          <Shield className="icon security" />
          <span>Threat Level</span>
        </div>
        <div className="stat-value text-green">Low</div>
      </div>

      {/* Main Charts */}
      <div className="chart-container wide">
        <h3>System Performance</h3>
        <div className="chart-wrapper">
          <ResponsiveContainer width="100%" height={300}>
            <AreaChart data={systemData}>
              <defs>
                <linearGradient id="colorCpu" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#8884d8" stopOpacity={0.8} />
                  <stop offset="95%" stopColor="#8884d8" stopOpacity={0} />
                </linearGradient>
                <linearGradient id="colorMem" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#82ca9d" stopOpacity={0.8} />
                  <stop offset="95%" stopColor="#82ca9d" stopOpacity={0} />
                </linearGradient>
              </defs>
              <XAxis dataKey="timestamp" />
              <YAxis />
              <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#333" />
              <Tooltip 
                contentStyle={{ backgroundColor: "#1f2937", border: "none" }}
                itemStyle={{ color: "#fff" }}
              />
              <Area
                type="monotone"
                dataKey="cpu"
                stroke="#8884d8"
                fillOpacity={1}
                fill="url(#colorCpu)"
                name="CPU %"
              />
              <Area
                type="monotone"
                dataKey="memory"
                stroke="#82ca9d"
                fillOpacity={1}
                fill="url(#colorMem)"
                name="Memory (MB)"
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div className="chart-container">
        <h3>Event Severity Distribution</h3>
        <div className="chart-wrapper">
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={severityData}>
              <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#333" />
              <XAxis dataKey="name" />
              <YAxis />
              <Tooltip
                cursor={{ fill: "transparent" }}
                contentStyle={{ backgroundColor: "#1f2937", border: "none" }}
              />
              <Bar dataKey="count" fill="#f59e0b" radius={[4, 4, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
};
