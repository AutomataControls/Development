"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Switch } from "@/components/ui/switch"
import { Slider } from "@/components/ui/slider"
import { Progress } from "@/components/ui/progress"
import { Cpu, Activity, Settings, Wifi, Power, Gauge, Eye, RefreshCw, Cloud, Server, FileCode, Wrench } from "lucide-react"
import { WiEarthquake } from "react-icons/wi"
import Image from 'next/image'

import FirmwareManager from "@/components/firmware-manager"
import BmsIntegration from "@/components/bms-integration"
import MaintenanceMode from "@/components/maintenance-mode"
import MetricsVisualization from "@/components/metrics-visualization"

interface ChannelConfig {
  id: string
  name: string
  description: string
  sensor_type: string
  input_type?: string
  scaling_min: number
  scaling_max: number
  units: string
  enabled: boolean
  alarm_high?: number
  alarm_low?: number
  calibration_offset: number
}

interface BoardConfig {
  board_id: string
  board_name: string
  location: string
  universal_inputs: ChannelConfig[]
  analog_outputs: ChannelConfig[]
  relay_outputs: ChannelConfig[]
  triac_outputs: ChannelConfig[]
}

const mockBoards = [
  {
    board_type: "SM-I-002 Building Automation",
    stack_level: 0,
    firmware_version: "2.1.3",
    status: "Connected",
    capabilities: {
      universal_inputs: 8,
      analog_outputs: 4,
      digital_inputs: 0,
      digital_outputs: 0,
      relays: 0,
      triacs: 4,
      has_rtc: true,
      has_watchdog: true,
      has_1wire: true,
    },
  },
  {
    board_type: "SM16univin 16 Universal Input",
    stack_level: 0,
    firmware_version: "1.2.1",
    status: "Connected",
    capabilities: {
      universal_inputs: 16,
      analog_outputs: 0,
      digital_inputs: 0,
      digital_outputs: 0,
      relays: 0,
      triacs: 0,
      has_rtc: true,
      has_watchdog: true,
      has_1wire: false,
    },
  },
  {
    board_type: "SM16relind 16-Relay",
    stack_level: 0,
    firmware_version: "1.0.2",
    status: "Connected",
    capabilities: {
      universal_inputs: 0,
      analog_outputs: 0,
      digital_inputs: 0,
      digital_outputs: 0,
      relays: 16,
      triacs: 0,
      has_rtc: false,
      has_watchdog: true,
      has_1wire: false,
    },
  },
  {
    board_type: "SM8relind 8-Relay",
    stack_level: 1,
    firmware_version: "1.1.0",
    status: "Connected",
    capabilities: {
      universal_inputs: 0,
      analog_outputs: 0,
      digital_inputs: 0,
      digital_outputs: 0,
      relays: 8,
      triacs: 0,
      has_rtc: false,
      has_watchdog: false,
      has_1wire: false,
    },
  },
]

const mockIoState = {
  board_id: "sm_i_002_building_automation_0",
  universal_inputs: [2.35, 7.12, 6.85, 0.85, 3.21, 4.67, 1.23, 8.9],
  analog_outputs: [3.2, 6.8, 0.0, 4.5],
  relay_states: [],
  triac_states: [true, false, true, false],
  timestamp: new Date().toISOString(),
}

const mockLogicFiles = [
  {
    id: "fcog-ahu-001",
    name: "FirstChurchOfGod AHU Control",
    file_path: "/home/pi/logic/firstchurchofgod/air-handler.js",
    equipment_type: "Air Handler",
    location_id: "9",
    equipment_id: "WAg6mWpJneM2zLMDu11b",
    description: "Supply air temperature control with OAR, static pressure control, and unoccupied cycling",
    last_modified: new Date().toISOString(),
    is_active: true,
    execution_interval: 30,
    last_execution: new Date(Date.now() - 45000).toISOString(),
    execution_count: 1247,
    last_error: null,
  },
]

export default function BuildingAutomationPreview() {
  const [selectedBoard, setSelectedBoard] = useState("sm_i_002_building_automation_0")
  const [selectedLogic, setSelectedLogic] = useState("fcog-ahu-001")
  const [isMonitoring, setIsMonitoring] = useState(true)
  const [autoExecuteEnabled, setAutoExecuteEnabled] = useState(true)
  const [isExecuting, setIsExecuting] = useState(false)
  const [uploadDialogOpen, setUploadDialogOpen] = useState(false)
  const [logicFilePath, setLogicFilePath] = useState("")
  const [boardConfigs, setBoardConfigs] = useState<Record<string, BoardConfig>>({})
  const [configDialogOpen, setConfigDialogOpen] = useState(false)
  const [editingChannel, setEditingChannel] = useState<ChannelConfig | null>(null)
  const [editingChannelType, setEditingChannelType] = useState<string>("")
  const [maintenanceMode, setMaintenanceMode] = useState(false)

  // Store metrics when monitoring is active
  const storeMetrics = async () => {
    if (isMonitoring && boardConfigs[selectedBoard]) {
      try {
        // Check if we're in Tauri environment
        if (typeof window !== 'undefined' && (window as any).__TAURI__) {
          const { invoke } = await import("@tauri-apps/api/tauri");
          await invoke("store_board_metrics", {
            boardId: selectedBoard,
            boardConfig: boardConfigs[selectedBoard]
          });
        } else {
          console.log("Tauri not available - metrics storage disabled in web mode");
        }
      } catch (error) {
        console.error("Failed to store metrics:", error)
      }
    }
  }

  // Store metrics every 30 seconds when monitoring
  useEffect(() => {
    if (isMonitoring) {
      const interval = setInterval(storeMetrics, 30000)
      return () => clearInterval(interval)
    }
  }, [isMonitoring, selectedBoard])

  const mockBoardConfigs: Record<string, BoardConfig> = {
    sm_i_002_building_automation_0: {
      board_id: "sm_i_002_building_automation_0",
      board_name: "FirstChurchOfGod AHU-001 BAS",
      location: "Mechanical Room A",
      universal_inputs: [
        {
          id: "ui_1",
          name: "Supply Air Temperature",
          description: "AHU supply air temperature sensor (10K thermistor)",
          sensor_type: "temperature",
          input_type: "resistance",
          scaling_min: 0,
          scaling_max: 100,
          units: "°F",
          enabled: true,
          alarm_high: 85,
          alarm_low: 45,
          calibration_offset: 0.0,
        },
        {
          id: "ui_2",
          name: "Return Air Temperature",
          description: "Return air temperature from space (0-10V)",
          sensor_type: "temperature",
          input_type: "voltage",
          scaling_min: 0,
          scaling_max: 100,
          units: "°F",
          enabled: true,
          alarm_high: 80,
          alarm_low: 65,
          calibration_offset: -1.2,
        },
        {
          id: "ui_3",
          name: "Mixed Air Temperature",
          description: "Mixed air temperature after dampers (4-20mA)",
          sensor_type: "temperature",
          input_type: "current",
          scaling_min: 0,
          scaling_max: 100,
          units: "°F",
          enabled: true,
          calibration_offset: 0.5,
        },
        {
          id: "ui_4",
          name: "Outdoor Air Temperature",
          description: "Outside air temperature sensor (0-10V)",
          sensor_type: "temperature",
          input_type: "voltage",
          scaling_min: -20,
          scaling_max: 120,
          units: "°F",
          enabled: true,
          calibration_offset: 0.0,
        },
        {
          id: "ui_5",
          name: "Supply Static Pressure",
          description: "Duct static pressure sensor (4-20mA)",
          sensor_type: "pressure",
          input_type: "current",
          scaling_min: 0,
          scaling_max: 4,
          units: '"WC',
          enabled: true,
          alarm_high: 3.5,
          alarm_low: 0.5,
          calibration_offset: 0.0,
        },
        {
          id: "ui_6",
          name: "Filter Differential Pressure",
          description: "Pressure drop across filters (0-10V)",
          sensor_type: "pressure",
          input_type: "voltage",
          scaling_min: 0,
          scaling_max: 2,
          units: '"WC',
          enabled: true,
          alarm_high: 1.5,
          calibration_offset: 0.0,
        },
        {
          id: "ui_7",
          name: "Occupancy Status",
          description: "Building occupancy digital input",
          sensor_type: "status",
          input_type: "digital",
          scaling_min: 0,
          scaling_max: 1,
          units: "",
          enabled: true,
          calibration_offset: 0.0,
        },
        {
          id: "ui_8",
          name: "Freeze Protection",
          description: "Freeze stat alarm input",
          sensor_type: "alarm",
          input_type: "digital",
          scaling_min: 0,
          scaling_max: 1,
          units: "",
          enabled: true,
          calibration_offset: 0.0,
        },
      ],
      analog_outputs: [
        {
          id: "ao_1",
          name: "Heating Valve",
          description: "Hot water heating coil valve (0-10V)",
          sensor_type: "valve_position",
          scaling_min: 0,
          scaling_max: 100,
          units: "%",
          enabled: true,
          calibration_offset: 0.0,
        },
        {
          id: "ao_2",
          name: "Cooling Valve",
          description: "Chilled water cooling coil valve (0-10V)",
          sensor_type: "valve_position",
          scaling_min: 0,
          scaling_max: 100,
          units: "%",
          enabled: true,
          calibration_offset: 0.0,
        },
        {
          id: "ao_3",
          name: "Outdoor Air Damper",
          description: "Outside air damper actuator (0-10V)",
          sensor_type: "damper_position",
          scaling_min: 0,
          scaling_max: 100,
          units: "%",
          enabled: true,
          calibration_offset: 0.0,
        },
        {
          id: "ao_4",
          name: "Supply Fan VFD",
          description: "Supply fan variable frequency drive (0-10V)",
          sensor_type: "vfd_speed",
          scaling_min: 0,
          scaling_max: 100,
          units: "%",
          enabled: true,
          alarm_high: 95,
          calibration_offset: 0.0,
        },
      ],
      relay_outputs: [],
      triac_outputs: [
        {
          id: "to_1",
          name: "Supply Fan Starter",
          description: "Supply fan motor starter (24VAC)",
          sensor_type: "motor_starter",
          scaling_min: 0,
          scaling_max: 1,
          units: "",
          enabled: true,
          calibration_offset: 0.0,
        },
        {
          id: "to_2",
          name: "Chilled Water Pump",
          description: "CHW circulation pump starter (24VAC)",
          sensor_type: "pump",
          scaling_min: 0,
          scaling_max: 1,
          units: "",
          enabled: true,
          calibration_offset: 0.0,
        },
        {
          id: "to_3",
          name: "Hot Water Pump",
          description: "HW circulation pump starter (24VAC)",
          sensor_type: "pump",
          scaling_min: 0,
          scaling_max: 1,
          units: "",
          enabled: true,
          calibration_offset: 0.0,
        },
        {
          id: "to_4",
          name: "Auxiliary Equipment",
          description: "Auxiliary equipment control (24VAC)",
          sensor_type: "auxiliary",
          scaling_min: 0,
          scaling_max: 1,
          units: "",
          enabled: true,
          calibration_offset: 0.0,
        },
      ],
    },
  }

  useEffect(() => {
    setBoardConfigs(mockBoardConfigs)
  }, [])

  const getBoardIcon = (boardType: string) => {
    if (boardType.includes("Building Automation")) return <Cpu className="w-5 h-5" />
    if (boardType.includes("Relay")) return <Power className="w-5 h-5" />
    if (boardType.includes("Universal Input")) return <Eye className="w-5 h-5" />
    if (boardType.includes("Analog Output")) return <WiEarthquake className="w-5 h-5" />
    return <Settings className="w-5 h-5" />
  }

  const getBoardId = (board: any) => {
    return `${board.board_type.toLowerCase().replace(/[^a-z0-9]/g, "_")}_${board.stack_level}`
  }

  const currentBoard = mockBoards.find((b) => getBoardId(b) === selectedBoard)

  const getChannelName = (type: string, index: number) => {
    const config = boardConfigs[selectedBoard]
    if (!config) return `Channel ${index + 1}`

    if (type === "universal_input" && config.universal_inputs[index]) {
      return config.universal_inputs[index].name
    }
    return `Channel ${index + 1}`
  }

  const getScaledValue = (type: string, index: number, rawValue: number) => {
    const config = boardConfigs[selectedBoard]
    if (!config) return rawValue.toFixed(2)

    let channelConfig
    if (type === "universal_input" && config.universal_inputs[index]) {
      channelConfig = config.universal_inputs[index]
    }

    if (channelConfig) {
      let scaledValue
      if (channelConfig.input_type === "digital") {
        return rawValue > 5 ? "ON" : "OFF"
      } else {
        if (channelConfig.input_type === "current") {
          scaledValue =
            ((rawValue - 4) / 16) * (channelConfig.scaling_max - channelConfig.scaling_min) +
            channelConfig.scaling_min +
            channelConfig.calibration_offset
        } else {
          scaledValue =
            (rawValue * (channelConfig.scaling_max - channelConfig.scaling_min)) / 10 +
            channelConfig.scaling_min +
            channelConfig.calibration_offset
        }
        return `${scaledValue.toFixed(1)} ${channelConfig.units}`
      }
    }

    return rawValue.toFixed(2)
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 p-4">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-4xl font-bold text-gray-900 mb-2 flex items-center gap-3">
                <Image src="/automata-nexus-logo.png" alt="Automata Nexus" width={48} height={48} className="rounded" />
                Automata Nexus Control Center
                <Badge variant="outline" className="text-sm">
                  <Wifi className="w-3 h-3 mr-1" />
                  {mockBoards.length} Boards
                </Badge>
              </h1>
              <p className="text-gray-600">Professional I/O control with BMS command integration</p>
            </div>
            <div className="flex gap-2">
              <Button variant="outline">
                <RefreshCw className="w-4 h-4 mr-2" />
                Scan Boards
              </Button>
              <Button variant={isMonitoring ? "destructive" : "default"}>
                <Activity className="w-4 h-4 mr-2" />
                {isMonitoring ? "Stop Monitoring" : "Start Monitoring"}
              </Button>
            </div>
          </div>
        </div>

        {/* System Status Bar */}
        <Card className="mb-6 bg-gradient-to-r from-green-50 to-gray-50 border-green-200">
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-6">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 bg-green-500 rounded-full animate-pulse"></div>
                  <span className="font-medium">System Active</span>
                </div>
                <div className="flex items-center gap-2">
                  <Cpu className="w-4 h-4" />
                  <span className="text-sm">CPU: 42.1°C</span>
                </div>
                <div className="flex items-center gap-2">
                  <WiEarthquake className="w-4 h-4" />
                  <span className="text-sm">24V: OK</span>
                </div>
                <div className="flex items-center gap-2">
                  <Cloud className="w-4 h-4" />
                  <span className="text-sm">BMS: Connected</span>
                </div>
                <div className="flex items-center gap-2">
                  <FileCode className="w-4 h-4" />
                  <span className="text-sm">Logic: {mockLogicFiles.filter((lf) => lf.is_active).length} Active</span>
                </div>
              </div>
              {maintenanceMode ? (
                <Badge variant="destructive" className="animate-pulse">
                  <Wrench className="w-3 h-3 mr-1" />
                  MAINTENANCE MODE ACTIVE
                </Badge>
              ) : (
                <Badge variant="default" className="animate-pulse">
                  <Activity className="w-3 h-3 mr-1" />
                  BMS Commands Active
                </Badge>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Board Selection */}
        <Card className="mb-6">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Settings className="w-5 h-5" />
              Board Selection & Status
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label>Select Board</Label>
                <Select value={selectedBoard} onValueChange={setSelectedBoard}>
                  <SelectTrigger>
                    <SelectValue placeholder="Choose a board" />
                  </SelectTrigger>
                  <SelectContent>
                    {mockBoards.map((board) => {
                      const boardId = getBoardId(board)
                      return (
                        <SelectItem key={boardId} value={boardId}>
                          <div className="flex items-center gap-2">
                            {getBoardIcon(board.board_type)}
                            <div>
                              <div>{board.board_type}</div>
                              <div className="text-xs text-gray-500">Stack {board.stack_level}</div>
                            </div>
                          </div>
                        </SelectItem>
                      )
                    })}
                  </SelectContent>
                </Select>
              </div>
              {currentBoard && (
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-600">Status:</span>
                    <Badge variant="default">{currentBoard.status}</Badge>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-600">Firmware:</span>
                    <span className="text-sm font-medium">{currentBoard.firmware_version}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-600">Stack Level:</span>
                    <span className="text-sm font-medium">
                      {currentBoard.stack_level} (of {currentBoard.board_type})
                    </span>
                  </div>
                </div>
              )}
            </div>
          </CardContent>
        </Card>

        <Tabs defaultValue="bms-integration" className="space-y-6">
          <TabsList className="grid w-full grid-cols-9">
            <TabsTrigger value="io-control">I/O Control</TabsTrigger>
            <TabsTrigger value="monitoring">Live Monitoring</TabsTrigger>
            <TabsTrigger value="board-config">Board Config</TabsTrigger>
            <TabsTrigger value="logic-engine">Logic Engine</TabsTrigger>
            <TabsTrigger value="firmware">Firmware Updates</TabsTrigger>
            <TabsTrigger value="bms-integration">BMS Integration</TabsTrigger>
            <TabsTrigger value="processing">Processing Config</TabsTrigger>
            <TabsTrigger value="metrics">Metrics & Trends</TabsTrigger>
            <TabsTrigger value="maintenance" className={maintenanceMode ? "text-orange-600 font-bold" : ""}>
              Maintenance
            </TabsTrigger>
          </TabsList>

          <TabsContent value="bms-integration" className="space-y-6">
            <BmsIntegration boardId={selectedBoard} />
          </TabsContent>

          <TabsContent value="monitoring" className="space-y-6">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <Card className="lg:col-span-2">
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Gauge className="w-5 h-5" />
                    Universal Inputs - Live Monitoring
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
                    {mockIoState.universal_inputs
                      .slice(0, currentBoard?.capabilities?.universal_inputs || 8)
                      .map((value, i) => {
                        const config = boardConfigs[selectedBoard]?.universal_inputs[i]
                        return (
                          <div key={i} className="p-3 border rounded-lg">
                            <div className="flex items-center justify-between mb-2">
                              <span className="text-sm font-medium">{getChannelName("universal_input", i)}</span>
                              {config && (
                                <Badge
                                  variant={config.input_type === "digital" ? "default" : "secondary"}
                                  className="text-xs"
                                >
                                  {config.input_type?.toUpperCase()}
                                </Badge>
                              )}
                            </div>
                            <div className="text-right">
                              <div className="text-lg font-bold text-green-600">{value.toFixed(2)}V</div>
                              <div className="text-xs text-gray-500">{getScaledValue("universal_input", i, value)}</div>
                            </div>
                            {config?.input_type !== "digital" && <Progress value={value * 10} className="h-2 mt-2" />}
                          </div>
                        )
                      })}
                  </div>
                </CardContent>
              </Card>
            </div>
          </TabsContent>

          <TabsContent value="io-control" className="space-y-6">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {currentBoard?.capabilities?.analog_outputs && currentBoard.capabilities.analog_outputs > 0 && (
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <WiEarthquake className="w-5 h-5" />
                      Analog Outputs (0-10V)
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-4">
                      {mockIoState.analog_outputs.map((value, i) => (
                        <div key={i} className="space-y-2">
                          <div className="flex items-center justify-between">
                            <Label>{boardConfigs[selectedBoard]?.analog_outputs[i]?.name || `Channel ${i + 1}`}</Label>
                            <span className="text-sm font-medium">{value.toFixed(2)}V</span>
                          </div>
                          <Slider value={[value]} max={10} min={0} step={0.1} className="w-full" />
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              )}

              {currentBoard?.capabilities?.triacs && currentBoard.capabilities.triacs > 0 && (
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <WiEarthquake className="w-5 h-5" />
                      TRIAC Outputs (24VAC)
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="grid grid-cols-2 gap-4">
                      {mockIoState.triac_states.map((state, i) => (
                        <div key={i} className="flex items-center justify-between p-3 border rounded-lg">
                          <Label>{boardConfigs[selectedBoard]?.triac_outputs[i]?.name || `TRIAC ${i + 1}`}</Label>
                          <Switch checked={state} />
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              )}
            </div>
          </TabsContent>

          <TabsContent value="board-config" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Settings className="w-5 h-5" />
                  Board Configuration
                  {currentBoard && (
                    <Badge variant="outline" className="ml-auto">
                      {currentBoard.board_type} - Stack {currentBoard.stack_level}
                    </Badge>
                  )}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="p-4 bg-gray-50 border border-gray-200 rounded-lg">
                  <h4 className="font-semibold text-gray-800 mb-2">Board Configuration Ready</h4>
                  <p className="text-sm text-gray-700">
                    Universal inputs configured and ready for BMS command integration. Configure channel names, scaling,
                    and sensor types.
                  </p>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="logic-engine" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <FileCode className="w-5 h-5" />
                  Logic Engine
                  <Badge variant="outline" className="ml-auto">
                    {mockLogicFiles.length} Files
                  </Badge>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
                  <h4 className="font-semibold text-green-800 mb-2">BMS Command Integration Active</h4>
                  <p className="text-sm text-green-700">
                    System is querying BMS server for commands every 30 seconds. Local logic files serve as fallback
                    when BMS is unavailable.
                  </p>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="firmware" className="space-y-6">
            <FirmwareManager
              boards={mockBoards.map((board) => ({
                board_type: board.board_type,
                stack_level: board.stack_level,
                firmware_version: board.firmware_version,
                status: board.status,
                repo_name: getBoardId(board),
              }))}
            />
          </TabsContent>

          <TabsContent value="processing" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Server className="w-5 h-5" />
                  Processing Configuration
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center space-x-2">
                    <Switch checked={false} />
                    <Label>Enable Processing Integration</Label>
                  </div>

                  <Button className="w-full">
                    <Server className="w-4 h-4 mr-2" />
                    Configure Processing Settings
                  </Button>

                  <div className="p-4 bg-gray-50 border border-gray-200 rounded-lg">
                    <h4 className="font-semibold text-gray-800 mb-2">Processing Integration Disabled</h4>
                    <p className="text-sm text-gray-700">
                      Enable to send data to validation proxy for enhanced processing
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="metrics" className="space-y-6">
            <MetricsVisualization 
              boardId={selectedBoard} 
              boardConfig={boardConfigs[selectedBoard]} 
            />
          </TabsContent>

          <TabsContent value="maintenance" className="space-y-6">
            <MaintenanceMode onStatusChange={setMaintenanceMode} />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  )
}
