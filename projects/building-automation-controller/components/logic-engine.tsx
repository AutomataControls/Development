"use client"

import { useState, useEffect } from "react"
// Dynamic import for Tauri - will be null in web mode
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { FileCode, Play, Square, Upload, Trash2, Clock, CheckCircle, XCircle, Activity, Settings, Eye, Download } from 'lucide-react'

interface LogicFile {
  id: string
  name: string
  file_path: string
  equipment_type: string
  location_id: string
  equipment_id: string
  description: string
  last_modified: string
  is_active: boolean
  execution_interval: number
  last_execution?: string
  execution_count: number
  last_error?: string
}

interface LogicExecution {
  logic_id: string
  timestamp: string
  inputs: any
  outputs: any
  execution_time_ms: number
  success: boolean
  error_message?: string
}

interface LogicEngineProps {
  selectedBoard: string
}

export default function LogicEngine({ selectedBoard }: LogicEngineProps) {
  const [logicFiles, setLogicFiles] = useState<LogicFile[]>([])
  const [selectedLogic, setSelectedLogic] = useState<string>("")
  const [executionHistory, setExecutionHistory] = useState<LogicExecution[]>([])
  const [isExecuting, setIsExecuting] = useState(false)
  const [uploadDialogOpen, setUploadDialogOpen] = useState(false)
  const [logicFilePath, setLogicFilePath] = useState("")
  const [autoExecuteEnabled, setAutoExecuteEnabled] = useState(false)

  useEffect(() => {
    loadLogicFiles()
  }, [])

  useEffect(() => {
    if (selectedLogic) {
      loadExecutionHistory(selectedLogic)
    }
  }, [selectedLogic])

  useEffect(() => {
    let interval: NodeJS.Timeout
    if (autoExecuteEnabled && selectedLogic && selectedBoard) {
      interval = setInterval(() => {
        executeLogic(selectedLogic)
      }, 30000) // Execute every 30 seconds
    }
    return () => {
      if (interval) clearInterval(interval)
    }
  }, [autoExecuteEnabled, selectedLogic, selectedBoard])

  const loadLogicFiles = async () => {
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const files: LogicFile[] = await invoke("get_logic_files")
        setLogicFiles(files)
      } else {
        console.log("Web mode: Logic files loading disabled")
      }
    } catch (error) {
      console.error("Failed to load logic files:", error)
    }
  }

  const loadExecutionHistory = async (logicId: string) => {
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const history: LogicExecution[] = await invoke("get_logic_execution_history", {
          logicId,
        })
        setExecutionHistory(history.slice(-20)) // Show last 20 executions
      } else {
        console.log("Web mode: Execution history loading disabled")
      }
    } catch (error) {
      console.error("Failed to load execution history:", error)
    }
  }

  const uploadLogicFile = async () => {
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const logicId: string = await invoke("load_logic_file", {
          filePath: logicFilePath,
        })
        setUploadDialogOpen(false)
        setLogicFilePath("")
        loadLogicFiles()
        alert(`Logic file loaded successfully! ID: ${logicId}`)
      } else {
        console.log("Web mode: Logic file upload disabled")
        alert("Logic file upload is only available in desktop mode")
      }
    } catch (error) {
      console.error("Failed to upload logic file:", error)
      alert(`Failed to upload logic file: ${error}`)
    }
  }

  const toggleLogicActive = async (logicId: string, active: boolean) => {
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        await invoke("activate_logic_file", {
          logicId,
          active,
        })
        loadLogicFiles()
      } else {
        console.log("Web mode: Logic activation disabled")
      }
    } catch (error) {
      console.error("Failed to toggle logic active state:", error)
    }
  }

  const executeLogic = async (logicId: string) => {
    if (!selectedBoard) {
      alert("Please select a board first")
      return
    }

    setIsExecuting(true)
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const outputs = await invoke("execute_logic_file", {
          logicId,
          boardId: selectedBoard,
        })

        // Apply outputs to the board
        await invoke("apply_logic_outputs", {
          outputs,
          boardId: selectedBoard,
        })

        loadExecutionHistory(logicId)
        alert("Logic executed successfully!")
      } else {
        console.log("Web mode: Logic execution disabled")
        alert("Logic execution is only available in desktop mode")
      }
    } catch (error) {
      console.error("Failed to execute logic:", error)
      alert(`Logic execution failed: ${error}`)
    } finally {
      setIsExecuting(false)
    }
  }

  const deleteLogicFile = async (logicId: string) => {
    if (confirm("Are you sure you want to delete this logic file?")) {
      try {
        if (typeof window !== 'undefined' && (window as any).__TAURI__) {
          const { invoke } = await import("@tauri-apps/api/tauri");
          await invoke("delete_logic_file", { logicId })
          loadLogicFiles()
          if (selectedLogic === logicId) {
            setSelectedLogic("")
            setExecutionHistory([])
          }
        } else {
          console.log("Web mode: Logic file deletion disabled")
        }
      } catch (error) {
        console.error("Failed to delete logic file:", error)
      }
    }
  }

  const selectedLogicFile = logicFiles.find((lf) => lf.id === selectedLogic)

  return (
    <div className="space-y-6">
      {/* Header */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileCode className="w-5 h-5" />
            Logic Engine
            <Badge variant="outline" className="ml-auto">
              {logicFiles.length} Files
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="flex items-center space-x-2">
                <Switch
                  checked={autoExecuteEnabled}
                  onCheckedChange={setAutoExecuteEnabled}
                  disabled={!selectedLogic || !selectedBoard}
                />
                <Label>Auto Execute (30s interval)</Label>
              </div>
              {autoExecuteEnabled && (
                <Badge variant="default" className="animate-pulse">
                  <Activity className="w-3 h-3 mr-1" />
                  Running
                </Badge>
              )}
            </div>
            <div className="flex gap-2">
              <Dialog open={uploadDialogOpen} onOpenChange={setUploadDialogOpen}>
                <DialogTrigger asChild>
                  <Button>
                    <Upload className="w-4 h-4 mr-2" />
                    Load Logic File
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>Load Logic File</DialogTitle>
                  </DialogHeader>
                  <div className="space-y-4">
                    <div>
                      <Label>File Path</Label>
                      <Input
                        value={logicFilePath}
                        onChange={(e) => setLogicFilePath(e.target.value)}
                        placeholder="/path/to/logic-file.js"
                      />
                    </div>
                    <div className="flex justify-end gap-2">
                      <Button variant="outline" onClick={() => setUploadDialogOpen(false)}>
                        Cancel
                      </Button>
                      <Button onClick={uploadLogicFile} disabled={!logicFilePath}>
                        Load File
                      </Button>
                    </div>
                  </div>
                </DialogContent>
              </Dialog>
            </div>
          </div>
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Logic Files */}
        <Card>
          <CardHeader>
            <CardTitle>Logic Files</CardTitle>
          </CardHeader>
          <CardContent>
            <ScrollArea className="h-96">
              <div className="space-y-2">
                {logicFiles.map((logicFile) => (
                  <div
                    key={logicFile.id}
                    className={`p-3 border rounded-lg cursor-pointer transition-colors ${
                      selectedLogic === logicFile.id ? "border-blue-500 bg-blue-50" : "hover:bg-gray-50"
                    }`}
                    onClick={() => setSelectedLogic(logicFile.id)}
                  >
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <FileCode className="w-4 h-4" />
                        <span className="font-medium">{logicFile.name}</span>
                        {logicFile.is_active && (
                          <Badge variant="default" className="text-xs">
                            Active
                          </Badge>
                        )}
                      </div>
                      <div className="flex items-center gap-1">
                        <Switch
                          checked={logicFile.is_active}
                          onCheckedChange={(checked) => toggleLogicActive(logicFile.id, checked)}
                          size="sm"
                        />
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation()
                            deleteLogicFile(logicFile.id)
                          }}
                        >
                          <Trash2 className="w-3 h-3" />
                        </Button>
                      </div>
                    </div>
                    <div className="text-sm text-gray-600">
                      <div>Type: {logicFile.equipment_type}</div>
                      <div>Location: {logicFile.location_id}</div>
                      <div>Equipment: {logicFile.equipment_id}</div>
                      <div>Executions: {logicFile.execution_count}</div>
                      {logicFile.last_error && (
                        <div className="text-red-600 text-xs mt-1">Error: {logicFile.last_error}</div>
                      )}
                    </div>
                  </div>
                ))}
                {logicFiles.length === 0 && (
                  <div className="text-center text-gray-500 py-8">
                    <FileCode className="w-12 h-12 mx-auto mb-4 opacity-50" />
                    <p>No logic files loaded</p>
                    <p className="text-sm">Upload a JavaScript logic file to get started</p>
                  </div>
                )}
              </div>
            </ScrollArea>
          </CardContent>
        </Card>

        {/* Logic Details & Execution */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              {selectedLogicFile ? (
                <>
                  <Settings className="w-5 h-5" />
                  {selectedLogicFile.name}
                </>
              ) : (
                <>
                  <Eye className="w-5 h-5" />
                  Select Logic File
                </>
              )}
            </CardTitle>
          </CardHeader>
          <CardContent>
            {selectedLogicFile ? (
              <div className="space-y-4">
                {/* Logic File Details */}
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span className="text-gray-600">Equipment Type:</span>
                    <div className="font-medium">{selectedLogicFile.equipment_type}</div>
                  </div>
                  <div>
                    <span className="text-gray-600">Location ID:</span>
                    <div className="font-medium">{selectedLogicFile.location_id}</div>
                  </div>
                  <div>
                    <span className="text-gray-600">Equipment ID:</span>
                    <div className="font-medium">{selectedLogicFile.equipment_id}</div>
                  </div>
                  <div>
                    <span className="text-gray-600">Interval:</span>
                    <div className="font-medium">{selectedLogicFile.execution_interval}s</div>
                  </div>
                  <div className="col-span-2">
                    <span className="text-gray-600">Description:</span>
                    <div className="font-medium text-xs">{selectedLogicFile.description}</div>
                  </div>
                </div>

                {/* Execution Controls */}
                <div className="flex gap-2">
                  <Button
                    onClick={() => executeLogic(selectedLogic)}
                    disabled={isExecuting || !selectedBoard}
                    className="flex-1"
                  >
                    {isExecuting ? (
                      <>
                        <Square className="w-4 h-4 mr-2" />
                        Executing...
                      </>
                    ) : (
                      <>
                        <Play className="w-4 h-4 mr-2" />
                        Execute Now
                      </>
                    )}
                  </Button>
                </div>

                {!selectedBoard && (
                  <Alert>
                    <AlertDescription>Please select a board to execute logic</AlertDescription>
                  </Alert>
                )}

                {/* Execution History */}
                <div>
                  <h4 className="font-semibold mb-2 flex items-center gap-2">
                    <Clock className="w-4 h-4" />
                    Recent Executions
                  </h4>
                  <ScrollArea className="h-48">
                    <div className="space-y-2">
                      {executionHistory.map((execution, index) => (
                        <div
                          key={index}
                          className={`p-2 border rounded text-xs ${
                            execution.success ? "border-green-200 bg-green-50" : "border-red-200 bg-red-50"
                          }`}
                        >
                          <div className="flex items-center justify-between mb-1">
                            <div className="flex items-center gap-1">
                              {execution.success ? (
                                <CheckCircle className="w-3 h-3 text-green-600" />
                              ) : (
                                <XCircle className="w-3 h-3 text-red-600" />
                              )}
                              <span>{new Date(execution.timestamp).toLocaleTimeString()}</span>
                            </div>
                            <span>{execution.execution_time_ms}ms</span>
                          </div>
                          {execution.error_message && (
                            <div className="text-red-600">{execution.error_message}</div>
                          )}
                          {execution.success && execution.outputs && (
                            <div className="text-gray-600">
                              Outputs: {Object.keys(execution.outputs).length} values
                            </div>
                          )}
                        </div>
                      ))}
                      {executionHistory.length === 0 && (
                        <div className="text-center text-gray-500 py-4">No executions yet</div>
                      )}
                    </div>
                  </ScrollArea>
                </div>
              </div>
            ) : (
              <div className="text-center text-gray-500 py-8">
                <Eye className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>Select a logic file to view details</p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
