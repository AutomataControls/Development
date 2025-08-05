"use client"

import { useState, useEffect } from "react"
// Dynamic import for Tauri - will be null in web mode
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
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const savedConfig: BmsConfig | null = await invoke("get_bms_config", { boardId })
        if (savedConfig) {
          setConfig(savedConfig)
        }
      } else {
        console.log("Web mode: BMS config loading disabled")
      }
    } catch (error) {
      console.error("Failed to load BMS config:", error)
    }
  }

  const loadConnectionStatus = async () => {
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const status: BmsConnectionStatus = await invoke("get_bms_connection_status")
        setConnectionStatus(status)
      } else {
        console.log("Web mode: BMS connection status loading disabled")
      }
    } catch (error) {
      console.error("Failed to load connection status:", error)
    }
  }

  const queryRecentCommands = async () => {
    if (!config.enabled || !config.equipment_id || !config.location_id) return

    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const commands: BmsCommand[] = await invoke("query_bms_commands", {
          equipmentId: config.equipment_id,
          locationId: config.location_id,
        })
        setRecentCommands(commands.slice(0, 10)) // Show last 10 commands
      } else {
        console.log("Web mode: BMS command querying disabled")
      }
    } catch (error) {
      console.error("Failed to query BMS commands:", error)
    }
  }

  const testConnection = async () => {
    setIsTestingConnection(true)
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const result: string = await invoke("test_bms_connection", {
          equipmentId: config.equipment_id,
          locationId: config.location_id,
        })
        alert(`✅ ${result}`)
        loadConnectionStatus()
      } else {
        console.log("Web mode: BMS connection testing disabled")
        alert("BMS connection testing is only available in desktop mode")
      }
    } catch (error) {
      alert(`❌ Connection test failed: ${error}`)
    } finally {
      setIsTestingConnection(false)
    }
  }

  const saveConfig = async () => {
    setIsSaving(true)
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        await invoke("save_bms_config", { boardId, config })
        alert("BMS configuration saved successfully!")
      } else {
        console.log("Web mode: BMS config saving disabled")
        alert("BMS configuration saving is only available in desktop mode")
      }
    } catch (error) {
      alert(`Failed to save configuration: ${error}`)
    } finally {
      setIsSaving(false)
    }
  }

  const getCommandSourceIcon = () => {
    if (connectionStatus.command_source === "bms") {
      return <Cloud className="w-4 h-4 text-blue-500" />
    } else {
      return <Server className="w-4 h-4 text-gray-500" />
    }
  }

  const getConnectionIcon = () => {
    if (connectionStatus.connected) {
      return <Wifi className="w-4 h-4 text-green-500" />
    } else {
      return <WifiOff className="w-4 h-4 text-red-500" />
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Database className="w-5 h-5" />
            BMS Integration & Command System
            <div className="flex items-center gap-2 ml-auto">
              {getConnectionIcon()}
              <Badge variant={connectionStatus.connected ? "default" : "secondary"}>
                {connectionStatus.connected ? "Connected" : "Disconnected"}
              </Badge>
              {getCommandSourceIcon()}
              <Badge variant="outline" className="text-xs">
                {connectionStatus.command_source.toUpperCase()}
              </Badge>
            </div>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="flex items-center space-x-2">
                <Switch
                  checked={config.enabled}
                  onCheckedChange={(checked) => setConfig({ ...config, enabled: checked })}
                />
                <Label>Enable BMS Integration</Label>
              </div>
              {config.enabled && (
                <Badge variant="default" className="animate-pulse">
                  <Activity className="w-3 h-3 mr-1" />
                  Active
                </Badge>
              )}
            </div>
            <div className="flex gap-2">
              <Button onClick={testConnection} disabled={isTestingConnection || !config.enabled} variant="outline">
                <RefreshCw className={`w-4 h-4 mr-2 ${isTestingConnection ? "animate-spin" : ""}`} />
                Test Connection
              </Button>
              <Button onClick={saveConfig} disabled={isSaving}>
                <Settings className="w-4 h-4 mr-2" />
                Save Config
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Connection Status */}
      {config.enabled && (
        <Alert className={connectionStatus.connected ? "border-green-200" : "border-red-200"}>
          <div className="flex items-center gap-2">
            {connectionStatus.connected ? (
              <CheckCircle className="h-4 w-4 text-green-600" />
            ) : (
              <XCircle className="h-4 w-4 text-red-600" />
            )}
            <AlertDescription>
              <div className="flex items-center justify-between w-full">
                <div>
                  <strong>Status:</strong> {connectionStatus.connected ? "Connected to BMS" : "Using Local Logic"}
                  {connectionStatus.last_error && (
                    <div className="text-sm text-red-600 mt-1">Error: {connectionStatus.last_error}</div>
                  )}
                </div>
                <div className="text-sm text-gray-600">
                  {connectionStatus.last_successful_query && (
                    <div>Last Query: {new Date(connectionStatus.last_successful_query).toLocaleTimeString()}</div>
                  )}
                  {connectionStatus.retry_count > 0 && <div>Retries: {connectionStatus.retry_count}</div>}
                </div>
              </div>
            </AlertDescription>
          </div>
        </Alert>
      )}

      <Tabs defaultValue="configuration" className="space-y-6">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="configuration">Configuration</TabsTrigger>
          <TabsTrigger value="commands">Live Commands</TabsTrigger>
          <TabsTrigger value="monitoring">Monitoring</TabsTrigger>
        </TabsList>

        <TabsContent value="configuration" className="space-y-6">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Basic Configuration */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Settings className="w-5 h-5" />
                  Basic Configuration
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <Label>Location Name</Label>
                  <Input
                    value={config.location_name}
                    onChange={(e) => setConfig({ ...config, location_name: e.target.value })}
                    placeholder="FirstChurchOfGod"
                  />
                </div>
                <div>
                  <Label>System Name</Label>
                  <Input
                    value={config.system_name}
                    onChange={(e) => setConfig({ ...config, system_name: e.target.value })}
                    placeholder="AHU-001"
                  />
                </div>
                <div>
                  <Label>Location ID</Label>
                  <Input
                    value={config.location_id}
                    onChange={(e) => setConfig({ ...config, location_id: e.target.value })}
                    placeholder="9"
                  />
                </div>
                <div>
                  <Label>Equipment ID</Label>
                  <Input
                    value={config.equipment_id}
                    onChange={(e) => setConfig({ ...config, equipment_id: e.target.value })}
                    placeholder="WAg6mWpJneM2zLMDu11b"
                  />
                </div>
                <div>
                  <Label>Equipment Type</Label>
                  <Input
                    value={config.equipment_type}
                    onChange={(e) => setConfig({ ...config, equipment_type: e.target.value })}
                    placeholder="Air Handler"
                  />
                </div>
                <div>
                  <Label>Zone</Label>
                  <Input
                    value={config.zone}
                    onChange={(e) => setConfig({ ...config, zone: e.target.value })}
                    placeholder="Main Building"
                  />
                </div>
              </CardContent>
            </Card>

            {/* Server Configuration */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Cloud className="w-5 h-5" />
                  BMS Server Configuration
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <Label>BMS Server URL</Label>
                  <Input
                    value={config.bms_server_url}
                    onChange={(e) => setConfig({ ...config, bms_server_url: e.target.value })}
                    placeholder="http://143.198.162.31:8205/api/v3/query_sql"
                  />
                </div>
                <div>
                  <Label>InfluxDB URL</Label>
                  <Input
                    value={config.influx_url}
                    onChange={(e) => setConfig({ ...config, influx_url: e.target.value })}
                    placeholder="http://143.198.162.31:8205/api/v3/query_sql"
                  />
                </div>
                <div>
                  <Label>Command Query Interval (seconds)</Label>
                  <Input
                    type="number"
                    value={config.command_query_interval}
                    onChange={(e) => setConfig({ ...config, command_query_interval: Number.parseInt(e.target.value) })}
                    placeholder="30"
                  />
                </div>
                <div>
                  <Label>Data Update Interval (seconds)</Label>
                  <Input
                    type="number"
                    value={config.update_interval}
                    onChange={(e) => setConfig({ ...config, update_interval: Number.parseInt(e.target.value) })}
                    placeholder="30"
                  />
                </div>
                <div className="flex items-center space-x-2">
                  <Switch
                    checked={config.fallback_to_local}
                    onCheckedChange={(checked) => setConfig({ ...config, fallback_to_local: checked })}
                  />
                  <Label>Fallback to Local Logic</Label>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Query Template */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Database className="w-5 h-5" />
                InfluxDB Query Template
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="bg-gray-50 p-4 rounded-lg font-mono text-sm">
                <div className="text-gray-600 mb-2">// Generated Query Template:</div>
                <div className="text-blue-600">
                  {`SELECT * FROM "ProcessingEngineCommands"`}
                  <br />
                  {`WHERE equipment_id = '${config.equipment_id}'`}
                  <br />
                  {`AND location_id = '${config.location_id}'`}
                  <br />
                  {`AND time >= now() - INTERVAL '5 minutes'`}
                  <br />
                  {`ORDER BY time DESC`}
                  <br />
                  {`LIMIT 35`}
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="commands" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Activity className="w-5 h-5" />
                Recent BMS Commands
                <Button onClick={queryRecentCommands} variant="outline" size="sm" className="ml-auto bg-transparent">
                  <RefreshCw className="w-3 h-3 mr-1" />
                  Refresh
                </Button>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <ScrollArea className="h-96">
                {recentCommands.length === 0 ? (
                  <div className="text-center text-gray-500 py-8">
                    <Database className="w-12 h-12 mx-auto mb-4 opacity-50" />
                    <p>No recent commands</p>
                    <p className="text-sm">
                      {config.enabled ? "Waiting for BMS commands..." : "Enable BMS integration to see commands"}
                    </p>
                  </div>
                ) : (
                  <div className="space-y-3">
                    {recentCommands.map((command, index) => (
                      <div key={index} className="p-3 border rounded-lg">
                        <div className="flex items-center justify-between mb-2">
                          <div className="flex items-center gap-2">
                            <Badge variant="outline" className="text-xs">
                              {command.command_type}
                            </Badge>
                            <Badge variant="secondary" className="text-xs">
                              Priority: {command.priority}
                            </Badge>
                          </div>
                          <div className="text-xs text-gray-500">
                            {new Date(command.timestamp).toLocaleTimeString()}
                          </div>
                        </div>
                        <div className="text-sm">
                          <div className="text-gray-600">Equipment: {command.equipment_id}</div>
                          <div className="text-gray-600">Location: {command.location_id}</div>
                          <div className="font-mono text-xs bg-gray-50 p-2 rounded mt-2">
                            {JSON.stringify(command.command_data, null, 2)}
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </ScrollArea>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="monitoring" className="space-y-6">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Command Source Status */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Activity className="w-5 h-5" />
                  Command Source Status
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex items-center gap-2">
                      {getCommandSourceIcon()}
                      <span className="font-medium">Current Source</span>
                    </div>
                    <Badge variant={connectionStatus.command_source === "bms" ? "default" : "secondary"}>
                      {connectionStatus.command_source === "bms" ? "BMS Server" : "Local Logic Files"}
                    </Badge>
                  </div>

                  <div className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex items-center gap-2">
                      {getConnectionIcon()}
                      <span className="font-medium">Connection Status</span>
                    </div>
                    <Badge variant={connectionStatus.connected ? "default" : "destructive"}>
                      {connectionStatus.connected ? "Connected" : "Disconnected"}
                    </Badge>
                  </div>

                  {connectionStatus.last_successful_query && (
                    <div className="flex items-center justify-between p-3 border rounded-lg">
                      <div className="flex items-center gap-2">
                        <Clock className="w-4 h-4" />
                        <span className="font-medium">Last Query</span>
                      </div>
                      <span className="text-sm">
                        {new Date(connectionStatus.last_successful_query).toLocaleString()}
                      </span>
                    </div>
                  )}

                  {connectionStatus.retry_count > 0 && (
                    <div className="flex items-center justify-between p-3 border rounded-lg">
                      <div className="flex items-center gap-2">
                        <AlertTriangle className="w-4 h-4 text-yellow-500" />
                        <span className="font-medium">Retry Count</span>
                      </div>
                      <Badge variant="outline">{connectionStatus.retry_count}</Badge>
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>

            {/* Fallback Configuration */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Server className="w-5 h-5" />
                  Fallback Configuration
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <Alert>
                    <AlertTriangle className="h-4 w-4" />
                    <AlertDescription>
                      When BMS connection is lost, the system will {config.fallback_to_local ? "automatically" : "NOT"}{" "}
                      fall back to local logic files.
                    </AlertDescription>
                  </Alert>

                  <div className="flex items-center space-x-2">
                    <Switch
                      checked={config.fallback_to_local}
                      onCheckedChange={(checked) => setConfig({ ...config, fallback_to_local: checked })}
                    />
                    <Label>Enable Automatic Fallback</Label>
                  </div>

                  <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg">
                    <h4 className="font-semibold text-blue-800 mb-2">How It Works:</h4>
                    <ul className="text-sm text-blue-700 space-y-1">
                      <li>• System queries BMS server for commands every {config.command_query_interval}s</li>
                      <li>• If BMS is available, commands are executed from server</li>
                      <li>• If BMS fails and fallback is enabled, local logic files are used</li>
                      <li>• System automatically reconnects when BMS becomes available</li>
                    </ul>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}
