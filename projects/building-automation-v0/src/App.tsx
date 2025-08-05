"use client"

import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/tauri"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts"
import { Gauge, AlertTriangle, CheckCircle, Settings, Activity } from "lucide-react"

interface SensorReading {
  channel: number
  voltage: number
  pressure: number
  temperature: number
  timestamp: string
  transducer_range: {
    min_psi: number
    max_psi: number
    model: string
  }
}

interface DiagnosticResult {
  timestamp: string
  refrigerant: string
  system_type: string
  superheat: number
  subcooling: number
  fault_condition: string
  confidence: string
  recommendations: string[]
}

interface SystemConfiguration {
  refrigerant_type: string
  system_type: string
  evaporator_temp: number
  condenser_temp: number
  suction_pressure_channel: number
  discharge_pressure_channel: number
  suction_temp_channel: number
  liquid_temp_channel: number
}

export default function App() {
  const [sensorReadings, setSensorReadings] = useState<SensorReading[]>([])
  const [diagnosticResult, setDiagnosticResult] = useState<DiagnosticResult | null>(null)
  const [systemConfig, setSystemConfig] = useState<SystemConfiguration>({
    refrigerant_type: "R-410A",
    system_type: "TXV",
    evaporator_temp: 40,
    condenser_temp: 105,
    suction_pressure_channel: 1,
    discharge_pressure_channel: 2,
    suction_temp_channel: 3,
    liquid_temp_channel: 4,
  })
  const [isMonitoring, setIsMonitoring] = useState(false)
  const [historicalData, setHistoricalData] = useState<any[]>([])

  useEffect(() => {
    let interval: NodeJS.Timeout
    if (isMonitoring) {
      interval = setInterval(async () => {
        await readSensors()
        await runDiagnostics()
      }, 5000) // Read every 5 seconds
    }
    return () => {
      if (interval) clearInterval(interval)
    }
  }, [isMonitoring, systemConfig])

  const readSensors = async () => {
    try {
      const readings: SensorReading[] = await invoke("read_pressure_sensors")
      setSensorReadings(readings)
    } catch (error) {
      console.error("Failed to read sensors:", error)
    }
  }

  const runDiagnostics = async () => {
    try {
      const result: DiagnosticResult = await invoke("calculate_superheat_subcooling", {
        config: systemConfig,
      })
      setDiagnosticResult(result)

      // Add to historical data
      setHistoricalData((prev) => [
        ...prev.slice(-50),
        {
          timestamp: new Date(result.timestamp).toLocaleTimeString(),
          superheat: result.superheat,
          subcooling: result.subcooling,
        },
      ])
    } catch (error) {
      console.error("Failed to run diagnostics:", error)
    }
  }

  const calibrateTransducer = async (channel: number, knownPressure: number) => {
    try {
      const result: string = await invoke("calibrate_transducer", {
        channel,
        knownPressure,
        pressureRange: {
          min_psi: 0,
          max_psi: 500,
          model: "P499VAP-105C",
        },
      })
      alert(result)
    } catch (error) {
      console.error("Calibration failed:", error)
    }
  }

  const getSeverityColor = (condition: string) => {
    if (condition === "Normal Operation") return "bg-green-500"
    if (condition.includes("Low Refrigerant") || condition.includes("Overcharge")) return "bg-red-500"
    if (condition.includes("High") || condition.includes("Restriction")) return "bg-yellow-500"
    return "bg-gray-500"
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 p-4">
      <div className="max-w-7xl mx-auto">
        <div className="mb-8">
          <h1 className="text-4xl font-bold text-gray-900 mb-2">🍓 HVAC Diagnostic System</h1>
          <p className="text-gray-600">Professional HVAC/Refrigeration diagnostics using P499 pressure transducers</p>
        </div>

        <Tabs defaultValue="monitoring" className="space-y-6">
          <TabsList className="grid w-full grid-cols-4">
            <TabsTrigger value="monitoring">Live Monitoring</TabsTrigger>
            <TabsTrigger value="diagnostics">Diagnostics</TabsTrigger>
            <TabsTrigger value="configuration">Configuration</TabsTrigger>
            <TabsTrigger value="calibration">Calibration</TabsTrigger>
          </TabsList>

          <TabsContent value="monitoring" className="space-y-6">
            <div className="flex gap-4 mb-6">
              <Button
                onClick={() => setIsMonitoring(!isMonitoring)}
                variant={isMonitoring ? "destructive" : "default"}
                className="flex items-center gap-2"
              >
                <Activity className="w-4 h-4" />
                {isMonitoring ? "Stop Monitoring" : "Start Monitoring"}
              </Button>
              <Button onClick={readSensors} variant="outline">
                <Gauge className="w-4 h-4 mr-2" />
                Read Sensors Now
              </Button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              {sensorReadings.map((reading) => (
                <Card key={reading.channel} className="relative">
                  <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium">Channel {reading.channel}</CardTitle>
                    <Badge variant="outline" className="text-xs">
                      {reading.transducer_range.model}
                    </Badge>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <span className="text-sm text-gray-600">Pressure:</span>
                        <span className="font-bold text-lg text-blue-600">{reading.pressure.toFixed(1)} psi</span>
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-sm text-gray-600">Voltage:</span>
                        <span className="text-sm">{reading.voltage.toFixed(2)} V</span>
                      </div>
                      <div className="text-xs text-gray-500">
                        Range: {reading.transducer_range.min_psi}-{reading.transducer_range.max_psi} psi
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>

            {historicalData.length > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle>Superheat & Subcooling Trends</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="h-64">
                    <ResponsiveContainer width="100%" height="100%">
                      <LineChart data={historicalData}>
                        <CartesianGrid strokeDasharray="3 3" />
                        <XAxis dataKey="timestamp" />
                        <YAxis />
                        <Tooltip />
                        <Line type="monotone" dataKey="superheat" stroke="#8884d8" name="Superheat (°F)" />
                        <Line type="monotone" dataKey="subcooling" stroke="#82ca9d" name="Subcooling (°F)" />
                      </LineChart>
                    </ResponsiveContainer>
                  </div>
                </CardContent>
              </Card>
            )}
          </TabsContent>

          <TabsContent value="diagnostics" className="space-y-6">
            {diagnosticResult && (
              <div className="space-y-4">
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      {diagnosticResult.fault_condition === "Normal Operation" ? (
                        <CheckCircle className="w-5 h-5 text-green-500" />
                      ) : (
                        <AlertTriangle className="w-5 h-5 text-yellow-500" />
                      )}
                      System Diagnosis
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
                      <div className="text-center">
                        <div className="text-2xl font-bold text-blue-600">
                          {diagnosticResult.superheat.toFixed(1)}°F
                        </div>
                        <div className="text-sm text-gray-600">Superheat</div>
                      </div>
                      <div className="text-center">
                        <div className="text-2xl font-bold text-green-600">
                          {diagnosticResult.subcooling.toFixed(1)}°F
                        </div>
                        <div className="text-sm text-gray-600">Subcooling</div>
                      </div>
                      <div className="text-center">
                        <Badge className={`${getSeverityColor(diagnosticResult.fault_condition)} text-white`}>
                          {diagnosticResult.confidence} Confidence
                        </Badge>
                      </div>
                    </div>

                    <Alert
                      className={
                        diagnosticResult.fault_condition === "Normal Operation"
                          ? "border-green-200"
                          : "border-yellow-200"
                      }
                    >
                      <AlertDescription>
                        <strong>Condition:</strong> {diagnosticResult.fault_condition}
                      </AlertDescription>
                    </Alert>

                    {diagnosticResult.recommendations.length > 0 && (
                      <div className="mt-4">
                        <h4 className="font-semibold mb-2">Recommendations:</h4>
                        <ul className="list-disc list-inside space-y-1">
                          {diagnosticResult.recommendations.map((rec, index) => (
                            <li key={index} className="text-sm text-gray-700">
                              {rec}
                            </li>
                          ))}
                        </ul>
                      </div>
                    )}
                  </CardContent>
                </Card>
              </div>
            )}
          </TabsContent>

          <TabsContent value="configuration" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Settings className="w-5 h-5" />
                  System Configuration
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium mb-2">Refrigerant Type</label>
                    <Select
                      value={systemConfig.refrigerant_type}
                      onValueChange={(value) => setSystemConfig({ ...systemConfig, refrigerant_type: value })}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="R-410A">R-410A (Current Standard)</SelectItem>
                        <SelectItem value="R-454B">R-454B (A2L Next-Gen)</SelectItem>
                        <SelectItem value="R-32">R-32 (A2L Single Component)</SelectItem>
                        <SelectItem value="R-22">R-22 (Legacy HCFC)</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  <div>
                    <label className="block text-sm font-medium mb-2">System Type</label>
                    <Select
                      value={systemConfig.system_type}
                      onValueChange={(value) => setSystemConfig({ ...systemConfig, system_type: value })}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="TXV">TXV (Thermostatic Expansion Valve)</SelectItem>
                        <SelectItem value="Fixed_Orifice">Fixed Orifice/Piston</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  <div>
                    <label className="block text-sm font-medium mb-2">Evaporator Temperature (°F)</label>
                    <Input
                      type="number"
                      value={systemConfig.evaporator_temp}
                      onChange={(e) =>
                        setSystemConfig({ ...systemConfig, evaporator_temp: Number.parseFloat(e.target.value) })
                      }
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-medium mb-2">Condenser Temperature (°F)</label>
                    <Input
                      type="number"
                      value={systemConfig.condenser_temp}
                      onChange={(e) =>
                        setSystemConfig({ ...systemConfig, condenser_temp: Number.parseFloat(e.target.value) })
                      }
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-medium mb-2">Suction Pressure Channel</label>
                    <Select
                      value={systemConfig.suction_pressure_channel.toString()}
                      onValueChange={(value) =>
                        setSystemConfig({ ...systemConfig, suction_pressure_channel: Number.parseInt(value) })
                      }
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {[1, 2, 3, 4, 5, 6, 7, 8].map((ch) => (
                          <SelectItem key={ch} value={ch.toString()}>
                            Channel {ch}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>

                  <div>
                    <label className="block text-sm font-medium mb-2">Discharge Pressure Channel</label>
                    <Select
                      value={systemConfig.discharge_pressure_channel.toString()}
                      onValueChange={(value) =>
                        setSystemConfig({ ...systemConfig, discharge_pressure_channel: Number.parseInt(value) })
                      }
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {[1, 2, 3, 4, 5, 6, 7, 8].map((ch) => (
                          <SelectItem key={ch} value={ch.toString()}>
                            Channel {ch}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="calibration" className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>Transducer Calibration</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <Alert>
                    <AlertDescription>
                      Connect a calibrated pressure gauge to verify transducer accuracy. P499 transducers have ±1%
                      accuracy specification.
                    </AlertDescription>
                  </Alert>

                  <div className="grid grid-cols-1 md:grid-cols-8 gap-2">
                    {[1, 2, 3, 4, 5, 6, 7, 8].map((channel) => (
                      <Button
                        key={channel}
                        variant="outline"
                        onClick={() => {
                          const pressure = prompt(`Enter known pressure for Channel ${channel} (psi):`)
                          if (pressure) {
                            calibrateTransducer(channel, Number.parseFloat(pressure))
                          }
                        }}
                        className="text-xs"
                      >
                        Cal Ch {channel}
                      </Button>
                    ))}
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
