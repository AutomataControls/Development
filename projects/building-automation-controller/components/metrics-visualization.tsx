"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Badge } from "@/components/ui/badge"
import { Label } from "@/components/ui/label"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { TrendingUp, TrendingDown, Activity, Clock, Database, BarChart, LineChart, Download } from "lucide-react"
// Dynamic import for Tauri - will be null in web mode
import {
  ResponsiveContainer,
  LineChart as RechartsLineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  Area,
  AreaChart,
} from "recharts"

interface TrendData {
  channel_name: string
  units?: string
  data_points: DataPoint[]
  statistics: Statistics
}

interface DataPoint {
  timestamp: string
  value: number
  scaled_value?: number
}

interface Statistics {
  min: number
  max: number
  avg: number
  std_dev: number
  count: number
}

interface Metric {
  id: string
  board_id: string
  channel_type: string
  channel_index: number
  channel_name: string
  value: number
  scaled_value?: number
  units?: string
  timestamp: string
}

interface ChannelConfig {
  enabled: boolean
  name: string
  [key: string]: any
}

interface BoardConfig {
  universal_inputs?: ChannelConfig[]
  analog_outputs?: ChannelConfig[]
  [key: string]: any
}

interface MetricsVisualizationProps {
  boardId: string
  boardConfig?: BoardConfig
}

export default function MetricsVisualization({ boardId, boardConfig }: MetricsVisualizationProps) {
  const [selectedChannel, setSelectedChannel] = useState<string>("")
  const [selectedHours, setSelectedHours] = useState<string>("24")
  const [trendData, setTrendData] = useState<TrendData | null>(null)
  const [channels, setChannels] = useState<Array<{ type: string; index: number; name: string }>>([])
  const [loading, setLoading] = useState(false)
  const [autoRefresh, setAutoRefresh] = useState(false)

  // Build channel list from board config
  useEffect(() => {
    if (boardConfig) {
      const channelList: Array<{ type: string; index: number; name: string }> = []
      
      // Add universal inputs
      boardConfig.universal_inputs?.forEach((channel, index) => {
        if (channel.enabled) {
          channelList.push({
            type: "universal_input",
            index,
            name: channel.name
          })
        }
      })
      
      // Add analog outputs
      boardConfig.analog_outputs?.forEach((channel, index) => {
        if (channel.enabled) {
          channelList.push({
            type: "analog_output",
            index,
            name: channel.name
          })
        }
      })
      
      setChannels(channelList)
      
      // Select first channel by default
      if (channelList.length > 0 && !selectedChannel) {
        setSelectedChannel(`${channelList[0].type}:${channelList[0].index}`)
      }
    }
  }, [boardConfig])

  // Fetch trend data
  const fetchTrendData = async () => {
    if (!selectedChannel) return
    
    setLoading(true)
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const [channelType, channelIndex] = selectedChannel.split(":")
        const data = await invoke<TrendData>("get_trend_data", {
          boardId,
          channelType,
          channelIndex: parseInt(channelIndex),
          hours: parseInt(selectedHours)
        })
        
        setTrendData(data)
      } else {
        console.log("Web mode: Trend data fetching disabled")
      }
    } catch (error) {
      console.error("Failed to fetch trend data:", error)
    } finally {
      setLoading(false)
    }
  }

  // Fetch data on channel or hours change
  useEffect(() => {
    if (selectedChannel) {
      fetchTrendData()
    }
  }, [selectedChannel, selectedHours, boardId])

  // Auto-refresh
  useEffect(() => {
    if (autoRefresh) {
      const interval = setInterval(fetchTrendData, 30000) // 30 seconds
      return () => clearInterval(interval)
    }
  }, [autoRefresh, selectedChannel, selectedHours])

  // Format timestamp for display
  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp)
    const hours = selectedHours
    
    if (parseInt(hours) <= 24) {
      return date.toLocaleTimeString()
    } else {
      return date.toLocaleDateString() + " " + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    }
  }

  // Prepare chart data
  const chartData = trendData?.data_points.map(point => ({
    time: formatTimestamp(point.timestamp),
    value: point.scaled_value || point.value,
    rawValue: point.value
  })) || []

  // Export data as CSV
  const exportData = () => {
    if (!trendData) return
    
    const csv = [
      ["Timestamp", "Raw Value", "Scaled Value", "Units"],
      ...trendData.data_points.map(point => [
        point.timestamp,
        point.value.toString(),
        (point.scaled_value || point.value).toString(),
        trendData.units || ""
      ])
    ].map(row => row.join(",")).join("\n")
    
    const blob = new Blob([csv], { type: "text/csv" })
    const url = URL.createObjectURL(blob)
    const a = document.createElement("a")
    a.href = url
    a.download = `${trendData.channel_name}_${new Date().toISOString()}.csv`
    a.click()
    URL.revokeObjectURL(url)
  }

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Database className="w-5 h-5" />
              Metrics & Trend Analysis
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setAutoRefresh(!autoRefresh)}
                className={autoRefresh ? "bg-green-50" : ""}
              >
                <Activity className={`w-4 h-4 mr-2 ${autoRefresh ? "animate-pulse" : ""}`} />
                {autoRefresh ? "Auto-Refresh ON" : "Auto-Refresh OFF"}
              </Button>
              <Button variant="outline" size="sm" onClick={exportData} disabled={!trendData}>
                <Download className="w-4 h-4 mr-2" />
                Export CSV
              </Button>
            </div>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
            <div>
              <Label>Channel</Label>
              <Select value={selectedChannel} onValueChange={setSelectedChannel}>
                <SelectTrigger>
                  <SelectValue placeholder="Select a channel" />
                </SelectTrigger>
                <SelectContent>
                  {channels.map((channel) => (
                    <SelectItem key={`${channel.type}:${channel.index}`} value={`${channel.type}:${channel.index}`}>
                      {channel.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            
            <div>
              <Label>Time Range</Label>
              <Select value={selectedHours} onValueChange={setSelectedHours}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="1">Last Hour</SelectItem>
                  <SelectItem value="6">Last 6 Hours</SelectItem>
                  <SelectItem value="12">Last 12 Hours</SelectItem>
                  <SelectItem value="24">Last 24 Hours</SelectItem>
                  <SelectItem value="48">Last 2 Days</SelectItem>
                  <SelectItem value="72">Last 3 Days</SelectItem>
                  <SelectItem value="168">Last 7 Days</SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            <div className="flex items-end">
              <Button onClick={fetchTrendData} disabled={loading} className="w-full">
                <Activity className="w-4 h-4 mr-2" />
                {loading ? "Loading..." : "Refresh Data"}
              </Button>
            </div>
          </div>

          {trendData && (
            <>
              {/* Statistics Cards */}
              <div className="grid grid-cols-2 md:grid-cols-5 gap-4 mb-6">
                <Card className="bg-blue-50 border-blue-200">
                  <CardContent className="p-4">
                    <div className="text-sm text-blue-600 font-medium">Current</div>
                    <div className="text-2xl font-bold text-blue-800">
                      {trendData.data_points.length > 0 
                        ? (trendData.data_points[trendData.data_points.length - 1].scaled_value || 
                           trendData.data_points[trendData.data_points.length - 1].value).toFixed(2)
                        : "N/A"}
                      {trendData.units && <span className="text-lg ml-1">{trendData.units}</span>}
                    </div>
                  </CardContent>
                </Card>
                
                <Card className="bg-green-50 border-green-200">
                  <CardContent className="p-4">
                    <div className="text-sm text-green-600 font-medium">Average</div>
                    <div className="text-2xl font-bold text-green-800">
                      {trendData.statistics.avg.toFixed(2)}
                      {trendData.units && <span className="text-lg ml-1">{trendData.units}</span>}
                    </div>
                  </CardContent>
                </Card>
                
                <Card className="bg-red-50 border-red-200">
                  <CardContent className="p-4">
                    <div className="text-sm text-red-600 font-medium flex items-center gap-1">
                      <TrendingUp className="w-3 h-3" />
                      Maximum
                    </div>
                    <div className="text-2xl font-bold text-red-800">
                      {trendData.statistics.max.toFixed(2)}
                      {trendData.units && <span className="text-lg ml-1">{trendData.units}</span>}
                    </div>
                  </CardContent>
                </Card>
                
                <Card className="bg-purple-50 border-purple-200">
                  <CardContent className="p-4">
                    <div className="text-sm text-purple-600 font-medium flex items-center gap-1">
                      <TrendingDown className="w-3 h-3" />
                      Minimum
                    </div>
                    <div className="text-2xl font-bold text-purple-800">
                      {trendData.statistics.min.toFixed(2)}
                      {trendData.units && <span className="text-lg ml-1">{trendData.units}</span>}
                    </div>
                  </CardContent>
                </Card>
                
                <Card className="bg-gray-50 border-gray-200">
                  <CardContent className="p-4">
                    <div className="text-sm text-gray-600 font-medium">Std Dev</div>
                    <div className="text-2xl font-bold text-gray-800">
                      {trendData.statistics.std_dev.toFixed(2)}
                    </div>
                  </CardContent>
                </Card>
              </div>

              {/* Chart Tabs */}
              <Tabs defaultValue="line" className="space-y-4">
                <TabsList>
                  <TabsTrigger value="line">
                    <LineChart className="w-4 h-4 mr-2" />
                    Line Chart
                  </TabsTrigger>
                  <TabsTrigger value="area">
                    <BarChart className="w-4 h-4 mr-2" />
                    Area Chart
                  </TabsTrigger>
                </TabsList>

                <TabsContent value="line" className="space-y-4">
                  <Card>
                    <CardContent className="pt-6">
                      <ResponsiveContainer width="100%" height={400}>
                        <RechartsLineChart data={chartData}>
                          <CartesianGrid strokeDasharray="3 3" />
                          <XAxis 
                            dataKey="time" 
                            angle={-45}
                            textAnchor="end"
                            height={80}
                          />
                          <YAxis />
                          <Tooltip />
                          <Legend />
                          <Line 
                            type="monotone" 
                            dataKey="value" 
                            stroke="#3b82f6" 
                            strokeWidth={2}
                            name={`${trendData.channel_name} ${trendData.units ? `(${trendData.units})` : ""}`}
                            dot={false}
                          />
                        </RechartsLineChart>
                      </ResponsiveContainer>
                    </CardContent>
                  </Card>
                </TabsContent>

                <TabsContent value="area" className="space-y-4">
                  <Card>
                    <CardContent className="pt-6">
                      <ResponsiveContainer width="100%" height={400}>
                        <AreaChart data={chartData}>
                          <CartesianGrid strokeDasharray="3 3" />
                          <XAxis 
                            dataKey="time" 
                            angle={-45}
                            textAnchor="end"
                            height={80}
                          />
                          <YAxis />
                          <Tooltip />
                          <Legend />
                          <Area 
                            type="monotone" 
                            dataKey="value" 
                            stroke="#3b82f6" 
                            fill="#93c5fd"
                            name={`${trendData.channel_name} ${trendData.units ? `(${trendData.units})` : ""}`}
                          />
                        </AreaChart>
                      </ResponsiveContainer>
                    </CardContent>
                  </Card>
                </TabsContent>
              </Tabs>

              {/* Data Summary */}
              <Card className="mt-4">
                <CardContent className="pt-4">
                  <div className="flex items-center justify-between text-sm text-gray-600">
                    <div className="flex items-center gap-2">
                      <Clock className="w-4 h-4" />
                      <span>Data Points: {trendData.statistics.count}</span>
                    </div>
                    <div>
                      Last Updated: {trendData.data_points.length > 0 
                        ? new Date(trendData.data_points[trendData.data_points.length - 1].timestamp).toLocaleString()
                        : "N/A"}
                    </div>
                  </div>
                </CardContent>
              </Card>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  )
}