// API client that works in both Tauri and web modes

const isWebMode = typeof window !== 'undefined' && !window.__TAURI__;
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

export interface Board {
  type: string;
  address: number;
  name: string;
}

export interface SystemInfo {
  platform: string;
  arch: string;
  uptime: number;
  memory: any;
  hardwareInterfaces: any;
}

class ApiClient {
  private async fetchApi(endpoint: string, options?: RequestInit) {
    if (!isWebMode && window.__TAURI__) {
      // In Tauri mode, use invoke
      const command = endpoint.replace('/api/', '').replace(/\//g, '_');
      return await window.__TAURI__.invoke(command, options?.body ? JSON.parse(options.body as string) : {});
    } else {
      // In web mode, use fetch
      const response = await fetch(`${API_BASE_URL}${endpoint}`, {
        ...options,
        headers: {
          'Content-Type': 'application/json',
          ...options?.headers,
        },
      });
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      return await response.json();
    }
  }

  // System
  async getSystemInfo(): Promise<SystemInfo> {
    return this.fetchApi('/api/system-info');
  }

  // Boards
  async getBoards(): Promise<Board[]> {
    return this.fetchApi('/api/boards');
  }

  // Inputs
  async readInput(board: number, channel: number): Promise<{ value: number }> {
    return this.fetchApi(`/api/inputs/${board}/${channel}`);
  }

  // Outputs
  async writeOutput(board: number, channel: number, value: number): Promise<{ success: boolean }> {
    return this.fetchApi(`/api/outputs/${board}/${channel}`, {
      method: 'POST',
      body: JSON.stringify({ value }),
    });
  }

  // Relays
  async setRelay(board: number, relay: number, state: boolean): Promise<{ success: boolean }> {
    return this.fetchApi(`/api/relays/${board}/${relay}`, {
      method: 'POST',
      body: JSON.stringify({ state }),
    });
  }

  // Metrics
  async getMetrics(name: string, hours: number = 24): Promise<any[]> {
    return this.fetchApi(`/api/metrics/${name}?hours=${hours}`);
  }

  async saveMetric(name: string, value: number): Promise<{ success: boolean }> {
    return this.fetchApi('/api/metrics', {
      method: 'POST',
      body: JSON.stringify({ name, value }),
    });
  }

  // Points
  async getPoints(): Promise<any> {
    return this.fetchApi('/api/points');
  }

  async savePoints(points: any): Promise<{ success: boolean }> {
    return this.fetchApi('/api/points', {
      method: 'POST',
      body: JSON.stringify(points),
    });
  }

  // Logic files
  async getLogicFiles(): Promise<any[]> {
    return this.fetchApi('/api/logic-files');
  }

  // BMS
  async getBmsConfig(): Promise<any> {
    return this.fetchApi('/api/bms/config');
  }

  async saveBmsConfig(config: any): Promise<{ success: boolean }> {
    return this.fetchApi('/api/bms/config', {
      method: 'POST',
      body: JSON.stringify(config),
    });
  }
}

// Export singleton instance
export const apiClient = new ApiClient();

// Export web mode flag
export { isWebMode };