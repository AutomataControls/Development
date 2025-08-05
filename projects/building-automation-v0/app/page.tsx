"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Switch } from "@/components/ui/switch"
import { Slider } from "@/components/ui/slider"
import { Progress } from "@/components/ui/progress"
import { ScrollArea } from "@/components/ui/scroll-area"
import {
  Cpu,
  Zap,
  Activity,
  Settings,
  Database,
  Wifi,
  Power,
  Gauge,
  Eye,
  RefreshCw,
  Cloud,
  Server,
  FileCode,
  Download,
} from "lucide-react"

interface ChannelConfig {
  id: string
  name: string
  description: string
  sensor_type: string
  input_type?: string // For universal inputs: voltage, current, resistance, digital
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

// Mock data for preview - Updated to show proper stacking
const mockBoards = [
  {
    board_type: "SM-I-002 Building Automation",
    stack_level: 0,
    firmware_version: "2.1.3",
    status: "Connected",
    capabilities: {
      universal_inputs: 8, // All 8 are universal
      analog_outputs: 4,
      digital_inputs: 0, // These are now part of universal inputs
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
    stack_level: 0, // Same stack 0, different board type
    firmware_version: "1.2.1",
    status: "Connected",
    capabilities: {
      universal_inputs: 16, // All 16 are universal
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
    stack_level: 0, // Same stack 0, different board type
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
    stack_level: 1, // Stack 1 of 8-relay type
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

const mockExecutionHistory = [
  {
    logic_id: "fcog-ahu-001",
    timestamp: new Date(Date.now() - 30000).toISOString(),
    execution_time_ms: 45,
    success: true,
    outputs: {
      heating_valve_position: 85.2,
      cooling_valve_position: 0.0,
      fan_enabled: true,
      fan_vfd_speed: 32.5,
      outdoor_damper_position: 100,
      supply_air_temp_setpoint: 58.5,
      unit_enable: true,
      is_occupied: true,
    },
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

  // Mock board configuration data - Updated for universal inputs
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
    sm16univin_16_universal_input_0: {
      board_id: "sm16univin_16_universal_input_0",
      board_name: "Zone Sensors - 16 Input",
      location: "Building Wide",
      universal_inputs: Array.from({ length: 16 }, (_, i) => ({
        id: `ui_${i + 1}`,
        name: i < 8 ? `Zone ${i + 1} Temperature` : `Zone ${i - 7} Humidity`,
        description: i < 8 ? `Zone ${i + 1} space temperature sensor` : `Zone ${i - 7} humidity sensor`,
        sensor_type: i < 8 ? "temperature" : "humidity",
        input_type: "voltage",
        scaling_min: i < 8 ? 65 : 0,
        scaling_max: i < 8 ? 85 : 100,
        units: i < 8 ? "°F" : "%RH",
        enabled: true,
        calibration_offset: 0.0,
      })),
      analog_outputs: [],
      relay_outputs: [],
      triac_outputs: [],
    },
    sm16relind_16_relay_0: {
      board_id: "sm16relind_16_relay_0",
      board_name: "Zone Control Relays",
      location: "Electrical Room",
      universal_inputs: [],
      analog_outputs: [],
      relay_outputs: Array.from({ length: 16 }, (_, i) => ({
        id: `ro_${i + 1}`,
        name: i < 8 ? `Zone ${i + 1} Heat` : `Zone ${i - 7} Cool`,
        description: i < 8 ? `Zone ${i + 1} heating relay` : `Zone ${i - 7} cooling relay`,
        sensor_type: i < 8 ? "heating" : "cooling",
        scaling_min: 0,
        scaling_max: 1,
        units: "",
        enabled: true,
        calibration_offset: 0.0,
      })),
      triac_outputs: [],
    },
    sm8relind_8_relay_1: {
      board_id: "sm8relind_8_relay_1",
      board_name: "Equipment Control Relays",
      location: "Mechanical Room A",
      universal_inputs: [],
      analog_outputs: [],
      relay_outputs: Array.from({ length: 8 }, (_, i) => {
        const names = [
          "Boiler #1",
          "Boiler #2",
          "Chiller #1",
          "Chiller #2",
          "Cooling Tower #1",
          "Cooling Tower #2",
          "Exhaust Fan #1",
          "Exhaust Fan #2",
        ]
        return {
          id: `ro_${i + 1}`,
          name: names[i],
          description: `${names[i]} control relay`,
          sensor_type: "equipment",
          scaling_min: 0,
          scaling_max: 1,
          units: "",
          enabled: true,
          calibration_offset: 0.0,
        }
      }),
      triac_outputs: [],
    },
  }

  // Initialize mock configs
  useEffect(() => {
    setBoardConfigs(mockBoardConfigs)
  }, [])

  const getBoardIcon = (boardType: string) => {
    if (boardType.includes("Building Automation")) return <Cpu className="w-5 h-5" />
    if (boardType.includes("Relay")) return <Power className="w-5 h-5" />
    if (boardType.includes("Universal Input")) return <Eye className="w-5 h-5" />
    if (boardType.includes("Analog Output")) return <Zap className="w-5 h-5" />
    return <Settings className="w-5 h-5" />
  }

  const getBoardId = (board: any) => {
    return `${board.board_type.toLowerCase().replace(/[^a-z0-9]/g, "_")}_${board.stack_level}`
  }

  const currentBoard = mockBoards.find((b) => getBoardId(b) === selectedBoard)
  const selectedLogicFile = mockLogicFiles.find((lf) => lf.id === selectedLogic)

  const executeLogic = () => {
    setIsExecuting(true)
    setTimeout(() => {
      setIsExecuting(false)
      alert("Logic executed successfully! Outputs applied to board.")
    }, 2000)
  }

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
        // Scale based on input type
        if (channelConfig.input_type === "current") {
          // 4-20mA scaling
          scaledValue =
            ((rawValue - 4) / 16) * (channelConfig.scaling_max - channelConfig.scaling_min) +
            channelConfig.scaling_min +
            channelConfig.calibration_offset
        } else {
          // 0-10V scaling
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
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-blue-50 p-4">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-4xl font-bold text-gray-900 mb-2 flex items-center gap-3">
                🍓 Building Automation Control Center
                <Badge variant="outline" className="text-sm">
                  <Wifi className="w-3 h-3 mr-1" />
                  {mockBoards.length} Boards
                </Badge>
              </h1>
              <p className="text-gray-600">Professional I/O control with universal input configuration</p>
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
        <Card className="mb-6 bg-gradient-to-r from-green-50 to-blue-50 border-green-200">
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
                  <Zap className="w-4 h-4" />
                  <span className="text-sm">24V: OK</span>
                </div>
                <div className="flex items-center gap-2">
                  <FileCode className="w-4 h-4" />
                  <span className="text-sm">Logic: {mockLogicFiles.filter((lf) => lf.is_active).length} Active</span>
                </div>
              </div>
              <Badge variant="default" className="animate-pulse">
                <Activity className="w-3 h-3 mr-1" />
                Auto-Executing
              </Badge>
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

        <Tabs defaultValue="board-config" className="space-y-6">
          <TabsList className="grid w-full grid-cols-6">
            <TabsTrigger value="io-control">I/O Control</TabsTrigger>
            <TabsTrigger value="monitoring">Live Monitoring</TabsTrigger>
            <TabsTrigger value="board-config">Board Config</TabsTrigger>
            <TabsTrigger value="logic-engine">Logic Engine</TabsTrigger>
            <TabsTrigger value="bms-config">BMS Integration</TabsTrigger>
            <TabsTrigger value="processing">Processing Config</TabsTrigger>
          </TabsList>

          <TabsContent value="board-config" className="space-y-6">
            {/* Board Configuration Header */}
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
                {currentBoard && boardConfigs[selectedBoard] && (
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div>
                      <Label>Board Name</Label>
                      <Input
                        value={boardConfigs[selectedBoard].board_name}
                        onChange={(e) =>
                          setBoardConfigs((prev) => ({
                            ...prev,
                            [selectedBoard]: {
                              ...prev[selectedBoard],
                              board_name: e.target.value,
                            },
                          }))
                        }
                        placeholder="Enter board name"
                      />
                    </div>
                    <div>
                      <Label>Location</Label>
                      <Input
                        value={boardConfigs[selectedBoard].location}
                        onChange={(e) =>
                          setBoardConfigs((prev) => ({
                            ...prev,
                            [selectedBoard]: {
                              ...prev[selectedBoard],
                              location: e.target.value,
                            },
                          }))
                        }
                        placeholder="Enter location"
                      />
                    </div>
                    <div className="flex items-end">
                      <Button className="w-full">
                        <Download className="w-4 h-4 mr-2" />
                        Export Config
                      </Button>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>

            {currentBoard && boardConfigs[selectedBoard] && (
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {/* Universal Inputs Configuration */}
                {currentBoard.capabilities.universal_inputs > 0 && (
                  <Card className="lg:col-span-2">
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2">
                        <Gauge className="w-5 h-5" />
                        Universal Inputs Configuration ({currentBoard.capabilities.universal_inputs} channels)
                        <Badge variant="secondary" className="ml-2">
                          Voltage | Current | Resistance | Digital
                        </Badge>
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <ScrollArea className="h-96">
                        <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
                          {boardConfigs[selectedBoard].universal_inputs.map((channel, index) => (
                            <div key={channel.id} className="p-3 border rounded-lg hover:bg-gray-50 transition-colors">
                              <div className="flex items-center justify-between mb-2">
                                <div className="flex items-center gap-2">
                                  <Badge variant="outline" className="text-xs">
                                    UI{index + 1}
                                  </Badge>
                                  <span className="font-medium text-sm">{channel.name}</span>
                                  <Badge
                                    variant={channel.input_type === "digital" ? "default" : "secondary"}
                                    className="text-xs"
                                  >
                                    {channel.input_type?.toUpperCase()}
                                  </Badge>
                                  <Switch
                                    checked={channel.enabled}
                                    onCheckedChange={(checked) =>
                                      setBoardConfigs((prev) => ({
                                        ...prev,
                                        [selectedBoard]: {
                                          ...prev[selectedBoard],
                                          universal_inputs: prev[selectedBoard].universal_inputs.map((ch, i) =>
                                            i === index ? { ...ch, enabled: checked } : ch,
                                          ),
                                        },
                                      }))
                                    }
                                    size="sm"
                                  />
                                </div>
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => {
                                    setEditingChannel(channel)
                                    setEditingChannelType("universal_input")
                                    setConfigDialogOpen(true)
                                  }}
                                >
                                  <Settings className="w-3 h-3" />
                                </Button>
                              </div>
                              <div className="text-xs text-gray-600 space-y-1">
                                <div>
                                  Type: {channel.sensor_type} ({channel.input_type})
                                </div>
                                <div>
                                  Range: {channel.scaling_min} - {channel.scaling_max} {channel.units}
                                </div>
                                <div>
                                  Current: {mockIoState.universal_inputs[index]?.toFixed(2)}V →{" "}
                                  {getScaledValue("universal_input", index, mockIoState.universal_inputs[index] || 0)}
                                </div>
                                {channel.description && <div className="italic">{channel.description}</div>}
                              </div>
                            </div>
                          ))}
                        </div>
                      </ScrollArea>
                    </CardContent>
                  </Card>
                )}

                {/* Analog Outputs Configuration */}
                {currentBoard.capabilities.analog_outputs > 0 && (
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2">
                        <Zap className="w-5 h-5" />
                        Analog Outputs Configuration
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <ScrollArea className="h-96">
                        <div className="space-y-3">
                          {boardConfigs[selectedBoard].analog_outputs.map((channel, index) => (
                            <div key={channel.id} className="p-3 border rounded-lg hover:bg-gray-50 transition-colors">
                              <div className="flex items-center justify-between mb-2">
                                <div className="flex items-center gap-2">
                                  <Badge variant="outline" className="text-xs">
                                    AO{index + 1}
                                  </Badge>
                                  <span className="font-medium text-sm">{channel.name}</span>
                                  <Switch
                                    checked={channel.enabled}
                                    onCheckedChange={(checked) =>
                                      setBoardConfigs((prev) => ({
                                        ...prev,
                                        [selectedBoard]: {
                                          ...prev[selectedBoard],
                                          analog_outputs: prev[selectedBoard].analog_outputs.map((ch, i) =>
                                            i === index ? { ...ch, enabled: checked } : ch,
                                          ),
                                        },
                                      }))
                                    }
                                    size="sm"
                                  />
                                </div>
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => {
                                    setEditingChannel(channel)
                                    setEditingChannelType("analog_output")
                                    setConfigDialogOpen(true)
                                  }}
                                >
                                  <Settings className="w-3 h-3" />
                                </Button>
                              </div>
                              <div className="text-xs text-gray-600 space-y-1">
                                <div>Type: {channel.sensor_type}</div>
                                <div>
                                  Range: {channel.scaling_min} - {channel.scaling_max} {channel.units}
                                </div>
                                <div>
                                  Current: {mockIoState.analog_outputs[index]?.toFixed(2)}V →{" "}
                                  {(
                                    ((mockIoState.analog_outputs[index] || 0) *
                                      (channel.scaling_max - channel.scaling_min)) /
                                      10 +
                                    channel.scaling_min
                                  ).toFixed(1)}{" "}
                                  {channel.units}
                                </div>
                                {channel.description && <div className="italic">{channel.description}</div>}
                              </div>
                            </div>
                          ))}
                        </div>
                      </ScrollArea>
                    </CardContent>
                  </Card>
                )}

                {/* Relay Outputs Configuration */}
                {currentBoard.capabilities.relays > 0 && (
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2">
                        <Power className="w-5 h-5" />
                        Relay Outputs Configuration ({currentBoard.capabilities.relays} relays)
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <ScrollArea className="h-96">
                        <div className="space-y-3">
                          {boardConfigs[selectedBoard].relay_outputs.map((channel, index) => (
                            <div key={channel.id} className="p-3 border rounded-lg hover:bg-gray-50 transition-colors">
                              <div className="flex items-center justify-between mb-2">
                                <div className="flex items-center gap-2">
                                  <Badge variant="outline" className="text-xs">
                                    R{index + 1}
                                  </Badge>
                                  <span className="font-medium text-sm">{channel.name}</span>
                                  <Switch
                                    checked={channel.enabled}
                                    onCheckedChange={(checked) =>
                                      setBoardConfigs((prev) => ({
                                        ...prev,
                                        [selectedBoard]: {
                                          ...prev[selectedBoard],
                                          relay_outputs: prev[selectedBoard].relay_outputs.map((ch, i) =>
                                            i === index ? { ...ch, enabled: checked } : ch,
                                          ),
                                        },
                                      }))
                                    }
                                    size="sm"
                                  />
                                </div>
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => {
                                    setEditingChannel(channel)
                                    setEditingChannelType("relay_output")
                                    setConfigDialogOpen(true)
                                  }}
                                >
                                  <Settings className="w-3 h-3" />
                                </Button>
                              </div>
                              <div className="text-xs text-gray-600 space-y-1">
                                <div>Type: {channel.sensor_type}</div>
                                <div>Current: OFF</div>
                                {channel.description && <div className="italic">{channel.description}</div>}
                              </div>
                            </div>
                          ))}
                        </div>
                      </ScrollArea>
                    </CardContent>
                  </Card>
                )}

                {/* TRIAC Outputs Configuration */}
                {currentBoard.capabilities.triacs > 0 && (
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2">
                        <Power className="w-5 h-5" />
                        TRIAC Outputs Configuration
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-3">
                        {boardConfigs[selectedBoard].triac_outputs.map((channel, index) => (
                          <div key={channel.id} className="p-3 border rounded-lg hover:bg-gray-50 transition-colors">
                            <div className="flex items-center justify-between mb-2">
                              <div className="flex items-center gap-2">
                                <Badge variant="outline" className="text-xs">
                                  TR{index + 1}
                                </Badge>
                                <span className="font-medium text-sm">{channel.name}</span>
                                <Switch
                                  checked={channel.enabled}
                                  onCheckedChange={(checked) =>
                                    setBoardConfigs((prev) => ({
                                      ...prev,
                                      [selectedBoard]: {
                                        ...prev[selectedBoard],
                                        triac_outputs: prev[selectedBoard].triac_outputs.map((ch, i) =>
                                          i === index ? { ...ch, enabled: checked } : ch,
                                        ),
                                      },
                                    }))
                                  }
                                  size="sm"
                                />
                              </div>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => {
                                  setEditingChannel(channel)
                                  setEditingChannelType("triac_output")
                                  setConfigDialogOpen(true)
                                }}
                              >
                                <Settings className="w-3 h-3" />
                              </Button>
                            </div>
                            <div className="text-xs text-gray-600 space-y-1">
                              <div>Type: {channel.sensor_type}</div>
                              <div>Current: {mockIoState.triac_states[index] ? "ON" : "OFF"}</div>
                              {channel.description && <div className="italic">{channel.description}</div>}
                            </div>
                          </div>
                        ))}
                      </div>
                    </CardContent>
                  </Card>
                )}
              </div>
            )}

            {/* Channel Configuration Dialog */}
            <Dialog open={configDialogOpen} onOpenChange={setConfigDialogOpen}>
              <DialogContent className="max-w-2xl">
                <DialogHeader>
                  <DialogTitle>Configure {editingChannel?.name || "Channel"}</DialogTitle>
                </DialogHeader>
                {editingChannel && (
                  <div className="grid grid-cols-2 gap-4 py-4">
                    <div>
                      <Label>Channel Name</Label>
                      <Input
                        value={editingChannel.name}
                        onChange={(e) => setEditingChannel({ ...editingChannel, name: e.target.value })}
                        placeholder="Enter channel name"
                      />
                    </div>
                    <div>
                      <Label>Sensor Type</Label>
                      <Select
                        value={editingChannel.sensor_type}
                        onValueChange={(value) => setEditingChannel({ ...editingChannel, sensor_type: value })}
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="temperature">Temperature</SelectItem>
                          <SelectItem value="pressure">Pressure</SelectItem>
                          <SelectItem value="humidity">Humidity</SelectItem>
                          <SelectItem value="flow">Flow</SelectItem>
                          <SelectItem value="co2">CO2</SelectItem>
                          <SelectItem value="valve_position">Valve Position</SelectItem>
                          <SelectItem value="damper_position">Damper Position</SelectItem>
                          <SelectItem value="vfd_speed">VFD Speed</SelectItem>
                          <SelectItem value="motor_starter">Motor Starter</SelectItem>
                          <SelectItem value="pump">Pump</SelectItem>
                          <SelectItem value="heating">Heating</SelectItem>
                          <SelectItem value="cooling">Cooling</SelectItem>
                          <SelectItem value="equipment">Equipment</SelectItem>
                          <SelectItem value="status">Status</SelectItem>
                          <SelectItem value="alarm">Alarm</SelectItem>
                          <SelectItem value="auxiliary">Auxiliary</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    {editingChannelType === "universal_input" && (
                      <div>
                        <Label>Input Type</Label>
                        <Select
                          value={editingChannel.input_type || "voltage"}
                          onValueChange={(value) => setEditingChannel({ ...editingChannel, input_type: value })}
                        >
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="voltage">0-10V</SelectItem>
                            <SelectItem value="current">4-20mA</SelectItem>
                            <SelectItem value="resistance">Resistance (10K Thermistor)</SelectItem>
                            <SelectItem value="digital">Digital (Dry Contact)</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    )}
                    <div>
                      <Label>Minimum Scale</Label>
                      <Input
                        type="number"
                        value={editingChannel.scaling_min}
                        onChange={(e) =>
                          setEditingChannel({
                            ...editingChannel,
                            scaling_min: Number.parseFloat(e.target.value),
                          })
                        }
                      />
                    </div>
                    <div>
                      <Label>Maximum Scale</Label>
                      <Input
                        type="number"
                        value={editingChannel.scaling_max}
                        onChange={(e) =>
                          setEditingChannel({
                            ...editingChannel,
                            scaling_max: Number.parseFloat(e.target.value),
                          })
                        }
                      />
                    </div>
                    <div>
                      <Label>Units</Label>
                      <Input
                        value={editingChannel.units}
                        onChange={(e) => setEditingChannel({ ...editingChannel, units: e.target.value })}
                        placeholder="°F, %, psi, etc."
                      />
                    </div>
                    <div>
                      <Label>Calibration Offset</Label>
                      <Input
                        type="number"
                        step="0.1"
                        value={editingChannel.calibration_offset}
                        onChange={(e) =>
                          setEditingChannel({
                            ...editingChannel,
                            calibration_offset: Number.parseFloat(e.target.value),
                          })
                        }
                      />
                    </div>
                    {editingChannelType === "universal_input" && editingChannel.input_type !== "digital" && (
                      <>
                        <div>
                          <Label>High Alarm</Label>
                          <Input
                            type="number"
                            value={editingChannel.alarm_high || ""}
                            onChange={(e) =>
                              setEditingChannel({
                                ...editingChannel,
                                alarm_high: e.target.value ? Number.parseFloat(e.target.value) : undefined,
                              })
                            }
                            placeholder="Optional"
                          />
                        </div>
                        <div>
                          <Label>Low Alarm</Label>
                          <Input
                            type="number"
                            value={editingChannel.alarm_low || ""}
                            onChange={(e) =>
                              setEditingChannel({
                                ...editingChannel,
                                alarm_low: e.target.value ? Number.parseFloat(e.target.value) : undefined,
                              })
                            }
                            placeholder="Optional"
                          />
                        </div>
                      </>
                    )}
                    <div className="col-span-2">
                      <Label>Description</Label>
                      <Input
                        value={editingChannel.description}
                        onChange={(e) => setEditingChannel({ ...editingChannel, description: e.target.value })}
                        placeholder="Enter channel description"
                      />
                    </div>
                  </div>
                )}
                <div className="flex justify-end gap-2">
                  <Button variant="outline" onClick={() => setConfigDialogOpen(false)}>
                    Cancel
                  </Button>
                  <Button
                    onClick={() => {
                      if (editingChannel) {
                        setBoardConfigs((prev) => {
                          const newConfig = { ...prev[selectedBoard] }
                          const channelType = editingChannelType.replace("_", "_") + "s"
                          const channelIndex = newConfig[channelType as keyof BoardConfig].findIndex(
                            (ch: any) => ch.id === editingChannel.id,
                          )
                          if (channelIndex >= 0) {
                            ;(newConfig[channelType as keyof BoardConfig] as any)[channelIndex] = editingChannel
                          }
                          return { ...prev, [selectedBoard]: newConfig }
                        })
                      }
                      setConfigDialogOpen(false)
                      setEditingChannel(null)
                    }}
                  >
                    Save Configuration
                  </Button>
                </div>
              </DialogContent>
            </Dialog>
          </TabsContent>

          <TabsContent value="monitoring" className="space-y-6">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* Universal Inputs */}
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
                      .slice(0, currentBoard?.capabilities.universal_inputs || 8)
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
                              <div className="text-lg font-bold text-blue-600">{value.toFixed(2)}V</div>
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
              {/* Analog Outputs */}
              {currentBoard?.capabilities.analog_outputs > 0 && (
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <Zap className="w-5 h-5" />
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

              {/* Triacs */}
              {currentBoard?.capabilities.triacs > 0 && (
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <Zap className="w-5 h-5" />
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

          <TabsContent value="logic-engine" className="space-y-6">
            {/* Logic Engine content remains the same */}
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
                <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
                  <h4 className="font-semibold text-blue-800 mb-2">Logic Engine Ready</h4>
                  <p className="text-sm text-blue-700">
                    Universal inputs configured and ready for logic execution. Load your JavaScript control files to
                    begin automated control.
                  </p>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="bms-config" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Database className="w-5 h-5" />
                  BMS Integration Configuration
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div className="flex items-center space-x-2">
                    <Switch checked={true} />
                    <Label>Enable BMS Integration</Label>
                  </div>

                  <Button className="w-full">
                    <Cloud className="w-4 h-4 mr-2" />
                    Configure BMS Settings
                  </Button>

                  <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
                    <h4 className="font-semibold text-green-800 mb-2">BMS Integration Active</h4>
                    <p className="text-sm text-green-700">Data will be sent to InfluxDB every 30 seconds</p>
                    <div className="mt-2 text-xs text-green-600">
                      Location: FirstChurchOfGod | System: AHU-001 | Equipment: WAg6mWpJneM2zLMDu11b
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
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
        </Tabs>
      </div>
    </div>
  )
}
