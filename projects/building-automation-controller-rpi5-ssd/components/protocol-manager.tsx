"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { useToast } from "@/components/ui/use-toast"
import { Loader2, Wifi, Usb, Plus, Trash2, Search, Download, Upload, RefreshCw } from "lucide-react"
import { invoke } from "@tauri-apps/api/tauri"

interface ProtocolConfig {
  protocol_type: "BacnetIp" | "BacnetMstp" | "ModbusTcp" | "ModbusRtu"
  connection: SerialConnection | NetworkConnection
  timeout_ms: number
  retry_count: number
  enabled: boolean
}

interface SerialConnection {
  Serial: {
    port: string
    baud_rate: number
    data_bits: number
    stop_bits: number
    parity: string
  }
}

interface NetworkConnection {
  Network: {
    ip_address: string
    port: number
    interface?: string
  }
}

interface PointValue {
  Bool?: boolean
  Int?: number
  Float?: number
  String?: string
}

export function ProtocolManager() {
  const [protocols, setProtocols] = useState<Record<string, ProtocolConfig>>({})
  const [availablePorts, setAvailablePorts] = useState<string[]>([])
  const [loading, setLoading] = useState(false)
  const [discovering, setDiscovering] = useState(false)
  const [selectedProtocol, setSelectedProtocol] = useState<string>("")
  const [devices, setDevices] = useState<Record<string, string[]>>({})
  const { toast } = useToast()

  // Form states for adding new protocol
  const [newProtocolName, setNewProtocolName] = useState("")
  const [newProtocolType, setNewProtocolType] = useState<ProtocolConfig["protocol_type"]>("ModbusRtu")
  const [connectionType, setConnectionType] = useState<"serial" | "network">("serial")
  const [serialPort, setSerialPort] = useState("")
  const [baudRate, setBaudRate] = useState(9600)
  const [ipAddress, setIpAddress] = useState("")
  const [networkPort, setNetworkPort] = useState(502)

  useEffect(() => {
    initializeProtocols()
    loadAvailablePorts()
  }, [])

  const initializeProtocols = async () => {
    try {
      await invoke("initialize_protocols")
      await loadProtocols()
    } catch (error) {
      toast({
        title: "Error",
        description: `Failed to initialize protocols: ${error}`,
        variant: "destructive",
      })
    }
  }

  const loadAvailablePorts = async () => {
    try {
      const ports = await invoke<string[]>("get_available_serial_ports")
      setAvailablePorts(ports)
    } catch (error) {
      toast({
        title: "Error",
        description: `Failed to load serial ports: ${error}`,
        variant: "destructive",
      })
    }
  }

  const loadProtocols = async () => {
    try {
      const protocolList = await invoke<string[]>("get_all_protocols")
      // Load each protocol's configuration
      const configs: Record<string, ProtocolConfig> = {}
      for (const name of protocolList) {
        // This would need a get_protocol_config command
        configs[name] = {} as ProtocolConfig
      }
      setProtocols(configs)
    } catch (error) {
      console.error("Failed to load protocols:", error)
    }
  }

  const addProtocol = async () => {
    if (!newProtocolName) {
      toast({
        title: "Error",
        description: "Please enter a protocol name",
        variant: "destructive",
      })
      return
    }

    setLoading(true)
    try {
      const config: ProtocolConfig = {
        protocol_type: newProtocolType,
        connection: connectionType === "serial" ? {
          Serial: {
            port: serialPort,
            baud_rate: baudRate,
            data_bits: 8,
            stop_bits: 1,
            parity: newProtocolType === "ModbusRtu" ? "even" : "none"
          }
        } : {
          Network: {
            ip_address: ipAddress,
            port: networkPort,
            interface: undefined
          }
        },
        timeout_ms: 3000,
        retry_count: 3,
        enabled: true
      }

      await invoke("add_protocol", { name: newProtocolName, config })
      
      toast({
        title: "Success",
        description: `Protocol ${newProtocolName} added successfully`,
      })

      // Reset form
      setNewProtocolName("")
      setSerialPort("")
      setIpAddress("")
      
      // Reload protocols
      await loadProtocols()
    } catch (error) {
      toast({
        title: "Error",
        description: `Failed to add protocol: ${error}`,
        variant: "destructive",
      })
    } finally {
      setLoading(false)
    }
  }

  const removeProtocol = async (name: string) => {
    try {
      await invoke("remove_protocol", { name })
      toast({
        title: "Success",
        description: `Protocol ${name} removed`,
      })
      await loadProtocols()
    } catch (error) {
      toast({
        title: "Error",
        description: `Failed to remove protocol: ${error}`,
        variant: "destructive",
      })
    }
  }

  const discoverDevices = async (protocolName: string) => {
    setDiscovering(true)
    try {
      const discovered = await invoke<string[]>("discover_protocol_devices", { protocol: protocolName })
      setDevices(prev => ({ ...prev, [protocolName]: discovered }))
      toast({
        title: "Discovery Complete",
        description: `Found ${discovered.length} devices`,
      })
    } catch (error) {
      toast({
        title: "Error",
        description: `Discovery failed: ${error}`,
        variant: "destructive",
      })
    } finally {
      setDiscovering(false)
    }
  }

  const testReadPoint = async (protocol: string, device: string, point: string) => {
    try {
      const value = await invoke<PointValue>("read_protocol_point", {
        protocol,
        deviceId: device,
        point
      })
      
      let displayValue = "Unknown"
      if ("Bool" in value) displayValue = value.Bool ? "True" : "False"
      else if ("Int" in value) displayValue = value.Int!.toString()
      else if ("Float" in value) displayValue = value.Float!.toFixed(2)
      else if ("String" in value) displayValue = value.String!
      
      toast({
        title: "Read Success",
        description: `${point} = ${displayValue}`,
      })
    } catch (error) {
      toast({
        title: "Error",
        description: `Failed to read point: ${error}`,
        variant: "destructive",
      })
    }
  }

  const isNetworkProtocol = (type: ProtocolConfig["protocol_type"]) => {
    return type === "BacnetIp" || type === "ModbusTcp"
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Protocol Manager</CardTitle>
          <CardDescription>
            Configure BACnet and Modbus protocols for integration with BMS systems
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Tabs defaultValue="protocols">
            <TabsList className="grid w-full grid-cols-3">
              <TabsTrigger value="protocols">Configured Protocols</TabsTrigger>
              <TabsTrigger value="add">Add Protocol</TabsTrigger>
              <TabsTrigger value="devices">Discovered Devices</TabsTrigger>
            </TabsList>

            <TabsContent value="protocols" className="space-y-4">
              <div className="flex justify-between items-center">
                <h3 className="text-lg font-semibold">Active Protocols</h3>
                <Button size="sm" variant="outline" onClick={loadProtocols}>
                  <RefreshCw className="h-4 w-4 mr-2" />
                  Refresh
                </Button>
              </div>

              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>Type</TableHead>
                    <TableHead>Connection</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {Object.entries(protocols).map(([name, config]) => (
                    <TableRow key={name}>
                      <TableCell className="font-medium">{name}</TableCell>
                      <TableCell>
                        <Badge variant="outline">
                          {config.protocol_type}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        {"Serial" in config.connection ? (
                          <div className="flex items-center gap-2">
                            <Usb className="h-4 w-4" />
                            <span className="text-sm">
                              {config.connection.Serial.port} @ {config.connection.Serial.baud_rate}
                            </span>
                          </div>
                        ) : (
                          <div className="flex items-center gap-2">
                            <Wifi className="h-4 w-4" />
                            <span className="text-sm">
                              {config.connection.Network.ip_address}:{config.connection.Network.port}
                            </span>
                          </div>
                        )}
                      </TableCell>
                      <TableCell>
                        <Badge variant={config.enabled ? "default" : "secondary"}>
                          {config.enabled ? "Enabled" : "Disabled"}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <div className="flex gap-2">
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => discoverDevices(name)}
                            disabled={discovering}
                          >
                            {discovering ? (
                              <Loader2 className="h-4 w-4 animate-spin" />
                            ) : (
                              <Search className="h-4 w-4" />
                            )}
                          </Button>
                          <Button
                            size="sm"
                            variant="destructive"
                            onClick={() => removeProtocol(name)}
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TabsContent>

            <TabsContent value="add" className="space-y-4">
              <div className="grid gap-4">
                <div>
                  <Label htmlFor="protocol-name">Protocol Name</Label>
                  <Input
                    id="protocol-name"
                    placeholder="e.g., Building A HVAC"
                    value={newProtocolName}
                    onChange={(e) => setNewProtocolName(e.target.value)}
                  />
                </div>

                <div>
                  <Label htmlFor="protocol-type">Protocol Type</Label>
                  <Select
                    value={newProtocolType}
                    onValueChange={(value) => {
                      setNewProtocolType(value as ProtocolConfig["protocol_type"])
                      setConnectionType(isNetworkProtocol(value as ProtocolConfig["protocol_type"]) ? "network" : "serial")
                    }}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="BacnetIp">BACnet IP</SelectItem>
                      <SelectItem value="BacnetMstp">BACnet MS/TP (RS485)</SelectItem>
                      <SelectItem value="ModbusTcp">Modbus TCP</SelectItem>
                      <SelectItem value="ModbusRtu">Modbus RTU (RS485)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {connectionType === "serial" ? (
                  <>
                    <div>
                      <Label htmlFor="serial-port">Serial Port</Label>
                      <div className="flex gap-2">
                        <Select value={serialPort} onValueChange={setSerialPort}>
                          <SelectTrigger>
                            <SelectValue placeholder="Select a port" />
                          </SelectTrigger>
                          <SelectContent>
                            {availablePorts.map((port) => (
                              <SelectItem key={port} value={port}>
                                {port}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                        <Button
                          size="icon"
                          variant="outline"
                          onClick={loadAvailablePorts}
                        >
                          <RefreshCw className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>

                    <div>
                      <Label htmlFor="baud-rate">Baud Rate</Label>
                      <Select
                        value={baudRate.toString()}
                        onValueChange={(value) => setBaudRate(parseInt(value))}
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="9600">9600</SelectItem>
                          <SelectItem value="19200">19200</SelectItem>
                          <SelectItem value="38400">38400</SelectItem>
                          <SelectItem value="57600">57600</SelectItem>
                          <SelectItem value="76800">76800</SelectItem>
                          <SelectItem value="115200">115200</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </>
                ) : (
                  <>
                    <div>
                      <Label htmlFor="ip-address">IP Address</Label>
                      <Input
                        id="ip-address"
                        placeholder="192.168.1.100"
                        value={ipAddress}
                        onChange={(e) => setIpAddress(e.target.value)}
                      />
                    </div>

                    <div>
                      <Label htmlFor="network-port">Port</Label>
                      <Input
                        id="network-port"
                        type="number"
                        placeholder={newProtocolType === "BacnetIp" ? "47808" : "502"}
                        value={networkPort}
                        onChange={(e) => setNetworkPort(parseInt(e.target.value))}
                      />
                    </div>
                  </>
                )}

                <Button
                  onClick={addProtocol}
                  disabled={loading || !newProtocolName || (connectionType === "serial" ? !serialPort : !ipAddress)}
                >
                  {loading ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      Adding...
                    </>
                  ) : (
                    <>
                      <Plus className="mr-2 h-4 w-4" />
                      Add Protocol
                    </>
                  )}
                </Button>
              </div>
            </TabsContent>

            <TabsContent value="devices" className="space-y-4">
              <div className="space-y-4">
                {Object.entries(devices).map(([protocol, deviceList]) => (
                  <Card key={protocol}>
                    <CardHeader>
                      <CardTitle className="text-base">{protocol}</CardTitle>
                      <CardDescription>
                        {deviceList.length} devices found
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-2">
                        {deviceList.map((device, idx) => (
                          <div key={idx} className="flex items-center justify-between p-2 border rounded">
                            <span className="text-sm">{device}</span>
                            <Dialog>
                              <DialogTrigger asChild>
                                <Button size="sm" variant="outline">
                                  Test Read
                                </Button>
                              </DialogTrigger>
                              <DialogContent>
                                <DialogHeader>
                                  <DialogTitle>Test Read Point</DialogTitle>
                                  <DialogDescription>
                                    Enter a point address to read from {device}
                                  </DialogDescription>
                                </DialogHeader>
                                <div className="space-y-4">
                                  <div>
                                    <Label>Point Address</Label>
                                    <Input
                                      placeholder="e.g., HR:100, AI:1"
                                      onKeyDown={(e) => {
                                        if (e.key === "Enter") {
                                          const input = e.currentTarget
                                          testReadPoint(protocol, device, input.value)
                                        }
                                      }}
                                    />
                                    <p className="text-xs text-muted-foreground mt-1">
                                      Examples: HR:100 (Holding Register), AI:1 (Analog Input)
                                    </p>
                                  </div>
                                </div>
                              </DialogContent>
                            </Dialog>
                          </div>
                        ))}
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  )
}