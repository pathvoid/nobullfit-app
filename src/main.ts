import { invoke } from "@tauri-apps/api/core";

// App detection
export function isApp(): boolean {
  return navigator.userAgent.includes("NBFAPP");
}

// File import functionality for app
export async function importCsvFile(): Promise<void> {
  if (!isApp()) {
    alert("App features unavailable in browser.");
    return;
  }
  
  try {
    // Open file picker dialog
    const filePath: string | null = await invoke("pick_csv_file");
    
    if (!filePath) {
      // User cancelled file selection
      return;
    }
    
    // Read the selected file
    const content: string = await invoke("read_text", { 
      path: filePath, 
      max_bytes: 2_000_000 
    });
    
    // Validate the CSV content
    const validation: any = await invoke("validate_csv", { content });
    
    if (validation.is_valid) {
      alert(`SUCCESS: ${validation.message}`);
      console.log("CSV validation successful:", validation);
      
      // TODO: In the future, we'll parse and import the data here
      // For now, just show the validation result
    } else {
      alert(`ERROR: Invalid CSV - ${validation.message}`);
      console.error("CSV validation failed:", validation);
    }
    
  } catch (error) {
    console.error("Import failed:", error);
    alert("Import failed: " + error);
  }
}

// Legacy function for backward compatibility
export async function pickAndImportCSV(): Promise<void> {
  return importCsvFile();
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
  importCsvFile,
  pickAndImportCSV
};
