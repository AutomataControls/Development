"use client"

import { useState, useEffect } from "react"
import { apiClient, isWebMode } from "@/lib/api-client"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Badge } from "@/components/ui/badge"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { ScrollArea } from "@/components/ui/scroll-area"
import {
  Database,
  Cloud,
  CheckCircle,
  XCircle,
  AlertTriangle,
  RefreshCw,
  Settings,
  Activity,
  Clock,
  Server,
  Wifi,
  WifiOff,
} from "lucide-react"

interface BmsConfig {
  enabled: boolean
  location_name: string
  system_name: string
  location_id: string
  equipment_id: string
  equipment_type: string
  zone: string
  influx_url: string
  update_interval: number
  field_mappings: Record<string, string>
  bms_server_url: string
  command_query_interval: number
  fallback_to_local: boolean
}

interface BmsConnectionStatus {
  connected: boolean
  last_successful_query?: string
  last_error?: string
  command_source: string
  retry_count: number
}

interface BmsCommand {
  equipment_id: string
  location_id: string
  command_type: string
  command_data: any
  timestamp: string
  priority: number
}

interface BmsIntegrationProps {
  boardId: string
}

export default function BmsIntegration({ boardId }: BmsIntegrationProps) {
  const [config, setConfig] = useState<BmsConfig>({
    enabled: false,
    location_name: "FirstChurchOfGod",
    system_name: "AHU-001",
    location_id: "9",
    equipment_id: "WAg6mWpJneM2zLMDu11b",
    equipment_type: "Air Handler",
    zone: "Main Building",
    influx_url: "http://143.198.162.31:8205/api/v3/query_sql",
    update_interval: 30,
    field_mappings: {},
    bms_server_url: "http://143.198.162.31:8205/api/v3/query_sql",
    command_query_interval: 30,
    fallback_to_local: true,
  })

  const [connectionStatus, setConnectionStatus] = useState<BmsConnectionStatus>({
    connected: false,
    command_source: "local",
    retry_count: 0,
  })

  const [recentCommands, setRecentCommands] = useState<BmsCommand[]>([])
  const [isTestingConnection, setIsTestingConnection] = useState(false)
  const [isSaving, setIsSaving] = useState(false)

  useEffect(() => {
    loadBmsConfig()
    loadConnectionStatus()
  }, [boardId])

  useEffect(() => {
    let interval: NodeJS.Timeout
    if (config.enabled && connectionStatus.connected) {
      interval = setInterval(() => {
        loadConnectionStatus()
        queryRecentCommands()
      }, config.command_query_interval * 1000)
    }
    return () => {
      if (interval) clearInterval(interval)
    }
  }, [config.enabled, config.command_query_interval, connectionStatus.connected])

  const loadBmsConfig = async () => {
    try {
      const savedConfig = await apiClient.getBmsConfig()
      if (savedConfig && Object.keys(savedConfig).length > 0) {
        setConfig({ ...config, ...savedConfig })
      }
    } catch (error) {
      console.error("Failed to load BMS config:", error)
    }
  }

  const loadConnectionStatus = async () => {
    try {
      // For now, just set a mock status in web mode
      if (isWebMode) {
        setConnectionStatus({
          connected: true,
          command_source: "local",
          retry_count: 0,
          last_successful_query: new Date().toISOString()
        })
      }
    } catch (error) {
      console.error("Failed to load connection status:", error)
    }
  }

  const queryRecentCommands = async () => {
    if (!config.enabled || !config.equipment_id || !config.location_id) return

    try {
      // In web mode, we'll skip command queries for now
      if (!isWebMode) {
        // Implement command query when backend supports it
      }
    } catch (error) {
      console.error("Failed to query commands:", error)
    }
  }

  const saveBmsConfig = async () => {
    setIsSaving(true)
    try {
      await apiClient.saveBmsConfig(config)
      // Show success message
    } catch (error) {
      console.error("Failed to save BMS config:", error)
    } finally {
      setIsSaving(false)
    }
  }

  const testConnection = async () => {
    setIsTestingConnection(true)
    try {
      // Implement connection test
      const response = await fetch(config.influx_url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query: 'SELECT 1'
        })
      })
      
      setConnectionStatus({
        ...connectionStatus,
        connected: response.ok,
        last_successful_query: response.ok ? new Date().toISOString() : undefined,
        last_error: response.ok ? undefined : `HTTP ${response.status}`
      })
    } catch (error: any) {
      setConnectionStatus({
        ...connectionStatus,
        connected: false,
        last_error: error.message
      })
    } finally {
      setIsTestingConnection(false)
    }
  }

  const updateFieldMapping = (field: string, bmsField: string) => {
    setConfig({
      ...config,
      field_mappings: {
        ...config.field_mappings,
        [field]: bmsField,
      },
    })
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Database className="h-5 w-5" />
            BMS Integration
          </div>
          <div className="flex items-center gap-2">
            {connectionStatus.connected ? (
              <Badge variant="default" className="flex items-center gap-1">
                <Wifi className="h-3 w-3" />
                Connected
              </Badge>
            ) : (
              <Badge variant="destructive" className="flex items-center gap-1">
                <WifiOff className="h-3 w-3" />
                Disconnected
              </Badge>
            )}
            <Badge variant={config.enabled ? "default" : "secondary"}>
              {config.enabled ? "Enabled" : "Disabled"}
            </Badge>
          </div>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="config" className="w-full">
          <TabsList className="grid w-full grid-cols-4">
            <TabsTrigger value="config">Configuration</TabsTrigger>
            <TabsTrigger value="mapping">Field Mapping</TabsTrigger>
            <TabsTrigger value="commands">Commands</TabsTrigger>
            <TabsTrigger value="status">Status</TabsTrigger>
          </TabsList>

          <TabsContent value="config" className="space-y-4">
            <div className="flex items-center justify-between">
              <Label htmlFor="bms-enabled">Enable BMS Integration</Label>
              <Switch
                id="bms-enabled"
                checked={config.enabled}
                onCheckedChange={(checked) =>
                  setConfig({ ...config, enabled: checked })
                }
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="location-name">Location Name</Label>
                <Input
                  id="location-name"
                  value={config.location_name}
                  onChange={(e) =>
                    setConfig({ ...config, location_name: e.target.value })
                  }
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="system-name">System Name</Label>
                <Input
                  id="system-name"
                  value={config.system_name}
                  onChange={(e) =>
                    setConfig({ ...config, system_name: e.target.value })
                  }
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="location-id">Location ID</Label>
                <Input
                  id="location-id"
                  value={config.location_id}
                  onChange={(e) =>
                    setConfig({ ...config, location_id: e.target.value })
                  }
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="equipment-id">Equipment ID</Label>
                <Input
                  id="equipment-id"
                  value={config.equipment_id}
                  onChange={(e) =>
                    setConfig({ ...config, equipment_id: e.target.value })
                  }
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="equipment-type">Equipment Type</Label>
                <Input
                  id="equipment-type"
                  value={config.equipment_type}
                  onChange={(e) =>
                    setConfig({ ...config, equipment_type: e.target.value })
                  }
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="zone">Zone</Label>
                <Input
                  id="zone"
                  value={config.zone}
                  onChange={(e) => setConfig({ ...config, zone: e.target.value })}
                />
              </div>

              <div className="space-y-2 col-span-2">
                <Label htmlFor="bms-url">BMS Server URL</Label>
                <Input
                  id="bms-url"
                  value={config.bms_server_url}
                  onChange={(e) =>
                    setConfig({ ...config, bms_server_url: e.target.value })
                  }
                  placeholder="http://server:port/api/v3/query_sql"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="update-interval">Update Interval (seconds)</Label>
                <Input
                  id="update-interval"
                  type="number"
                  value={config.update_interval}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      update_interval: parseInt(e.target.value),
                    })
                  }
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="command-interval">
                  Command Query Interval (seconds)
                </Label>
                <Input
                  id="command-interval"
                  type="number"
                  value={config.command_query_interval}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      command_query_interval: parseInt(e.target.value),
                    })
                  }
                />
              </div>
            </div>

            <div className="flex items-center justify-between">
              <Label htmlFor="fallback-local">Fallback to Local Control</Label>
              <Switch
                id="fallback-local"
                checked={config.fallback_to_local}
                onCheckedChange={(checked) =>
                  setConfig({ ...config, fallback_to_local: checked })
                }
              />
            </div>

            <div className="flex gap-2">
              <Button onClick={testConnection} disabled={isTestingConnection}>
                {isTestingConnection ? (
                  <RefreshCw className="h-4 w-4 animate-spin mr-2" />
                ) : (
                  <Activity className="h-4 w-4 mr-2" />
                )}
                Test Connection
              </Button>
              <Button onClick={saveBmsConfig} disabled={isSaving}>
                {isSaving ? (
                  <RefreshCw className="h-4 w-4 animate-spin mr-2" />
                ) : (
                  <Settings className="h-4 w-4 mr-2" />
                )}
                Save Configuration
              </Button>
            </div>
          </TabsContent>

          <TabsContent value="mapping" className="space-y-4">
            <Alert>
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>
                Map local sensor names to BMS field names for data synchronization.
              </AlertDescription>
            </Alert>

            <div className="space-y-4">
              {Object.entries(config.field_mappings).map(([field, bmsField]) => (
                <div key={field} className="grid grid-cols-2 gap-4">
                  <Input value={field} disabled />
                  <Input
                    value={bmsField}
                    onChange={(e) => updateFieldMapping(field, e.target.value)}
                    placeholder="BMS field name"
                  />
                </div>
              ))}
            </div>

            <Button
              variant="outline"
              onClick={() => updateFieldMapping("", "")}
              className="w-full"
            >
              Add Field Mapping
            </Button>
          </TabsContent>

          <TabsContent value="commands" className="space-y-4">
            <div className="flex items-center justify-between mb-4">
              <h4 className="text-sm font-medium">Recent Commands</h4>
              <Badge variant="outline">
                Source: {connectionStatus.command_source}
              </Badge>
            </div>

            <ScrollArea className="h-[300px] rounded-md border p-4">
              {recentCommands.length === 0 ? (
                <div className="text-center text-muted-foreground py-8">
                  No commands received
                </div>
              ) : (
                <div className="space-y-2">
                  {recentCommands.map((cmd, idx) => (
                    <Card key={idx} className="p-3">
                      <div className="flex items-center justify-between">
                        <div>
                          <p className="text-sm font-medium">{cmd.command_type}</p>
                          <p className="text-xs text-muted-foreground">
                            {new Date(cmd.timestamp).toLocaleString()}
                          </p>
                        </div>
                        <Badge
                          variant={cmd.priority > 5 ? "destructive" : "default"}
                        >
                          Priority: {cmd.priority}
                        </Badge>
                      </div>
                      <pre className="text-xs mt-2 p-2 bg-muted rounded">
                        {JSON.stringify(cmd.command_data, null, 2)}
                      </pre>
                    </Card>
                  ))}
                </div>
              )}
            </ScrollArea>
          </TabsContent>

          <TabsContent value="status" className="space-y-4">
            <div className="space-y-4">
              <div className="flex items-center justify-between p-4 border rounded-lg">
                <div className="flex items-center gap-2">
                  <Server className="h-5 w-5" />
                  <span>Connection Status</span>
                </div>
                {connectionStatus.connected ? (
                  <CheckCircle className="h-5 w-5 text-green-500" />
                ) : (
                  <XCircle className="h-5 w-5 text-red-500" />
                )}
              </div>

              {connectionStatus.last_successful_query && (
                <div className="p-4 border rounded-lg">
                  <div className="flex items-center gap-2 mb-2">
                    <Clock className="h-4 w-4" />
                    <span className="text-sm font-medium">Last Successful Query</span>
                  </div>
                  <p className="text-sm text-muted-foreground">
                    {new Date(connectionStatus.last_successful_query).toLocaleString()}
                  </p>
                </div>
              )}

              {connectionStatus.last_error && (
                <Alert variant="destructive">
                  <AlertTriangle className="h-4 w-4" />
                  <AlertDescription>{connectionStatus.last_error}</AlertDescription>
                </Alert>
              )}

              <div className="p-4 border rounded-lg">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Retry Count</span>
                  <Badge variant="outline">{connectionStatus.retry_count}</Badge>
                </div>
              </div>
            </div>
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  )
}