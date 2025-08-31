import { invoke } from "@tauri-apps/api/core";

// App detection
export function isApp(): boolean {
  return navigator.userAgent.includes("NBFAPP");
}

// File import functionality for app
export async function pickAndImportCSV(): Promise<void> {
  if (!isApp()) {
    alert("App features unavailable in browser.");
    return;
  }
  
  try {
    const text: string = await invoke("read_text", { 
      path: "/tmp/example.csv", 
      max_bytes: 2_000_000 
    });
    
    // Parse CSV and normalize data
    const rows = parseCsv(text);
    const entries = normalizeRows(rows);
    
    // Send to Phoenix backend
    await fetch("/api/import", {
      method: "POST",
      headers: { 
        "Content-Type": "application/json", 
        "x-csrf-token": getCsrf() 
      },
      body: JSON.stringify({ entries })
    });
    
    console.log("Import completed successfully");
  } catch (error) {
    console.error("Import failed:", error);
    alert("Import failed: " + error);
  }
}

// Helper functions
function parseCsv(text: string): string[][] {
  return text.split('\n')
    .map(line => line.split(',').map(cell => cell.trim()))
    .filter(row => row.length > 0 && row.some(cell => cell.length > 0));
}

function normalizeRows(rows: string[][]): any[] {
  // This would be implemented based on your specific CSV format
  // For now, return a basic structure
  return rows.slice(1).map(row => ({
    date: row[0] || "",
    metric: row[1] || "",
    value: parseFloat(row[2]) || 0,
    unit: row[3] || null
  }));
}

function getCsrf(): string {
  // Get CSRF token from meta tag or cookie
  const meta = document.querySelector('meta[name="csrf-token"]');
  return meta ? meta.getAttribute('content') || '' : '';
}

// Make functions available globally for the embedded website
(window as any).NBFAPP = {
  isApp,
  pickAndImportCSV
};
