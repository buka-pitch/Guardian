export const getSeverityColor = (severity: string): string => {
  switch (severity.toUpperCase()) {
    case "CRITICAL":
      return "#dc2626"; // Red-600
    case "HIGH":
      return "#ea580c"; // Orange-600
    case "MEDIUM":
      return "#ca8a04"; // Yellow-600
    case "LOW":
      return "#16a34a"; // Green-600
    case "INFO":
    default:
      return "#3b82f6"; // Blue-500
  }
};

export const formatDate = (timestamp: string): string => {
  return new Date(timestamp).toLocaleString();
};

export const formatJSON = (data: any): string => {
  return JSON.stringify(data, null, 2);
};
