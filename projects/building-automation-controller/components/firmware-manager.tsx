"use client"

import { useState, useEffect } from "react"
// Dynamic import for Tauri - will be null in web mode
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Progress } from "@/components/ui/progress"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Download, GitBranch, RefreshCw, CheckCircle, XCircle, AlertTriangle, Cpu, HardDrive, Wifi } from "lucide-react"

interface FirmwareRepo {
  name: string
  display_name: string
  repo_url: string
  local_path: string
  update_command: string
  is_cloned: boolean
  last_updated?: string
}

interface BoardInfo {
  board_type: string
  stack_level: number
  firmware_version: string
  status: string
  repo_name: string
}

interface FirmwareManagerProps {
  boards: BoardInfo[]
}

export default function FirmwareManager({ boards }: FirmwareManagerProps) {
  const [repos, setRepos] = useState<FirmwareRepo[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [operationStatus, setOperationStatus] = useState<Record<string, string>>({})
  const [updateProgress, setUpdateProgress] = useState<Record<string, number>>({})

  useEffect(() => {
    loadFirmwareRepos()
  }, [])

  const loadFirmwareRepos = async () => {
    setIsLoading(true)
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const repoList: FirmwareRepo[] = await invoke("check_firmware_repos")
        setRepos(repoList)
      } else {
        console.log("Web mode: Firmware repository loading disabled")
      }
    } catch (error) {
      console.error("Failed to load firmware repositories:", error)
    } finally {
      setIsLoading(false)
    }
  }

  const cloneRepository = async (repoName: string) => {
    setOperationStatus({ ...operationStatus, [repoName]: "Cloning..." })
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const result: string = await invoke("clone_firmware_repo", { repoName })
        setOperationStatus({ ...operationStatus, [repoName]: result })
        await loadFirmwareRepos() // Refresh the list
      } else {
        console.log("Web mode: Repository cloning disabled")
        setOperationStatus({ ...operationStatus, [repoName]: "Web mode: Operation not available" })
      }
    } catch (error) {
      setOperationStatus({ ...operationStatus, [repoName]: `Error: ${error}` })
    }
  }

  const installDrivers = async (repoName: string) => {
    setOperationStatus({ ...operationStatus, [repoName]: "Installing drivers..." })
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const result: string = await invoke("install_firmware_drivers", { repoName })
        setOperationStatus({ ...operationStatus, [repoName]: result })
      } else {
        console.log("Web mode: Driver installation disabled")
        setOperationStatus({ ...operationStatus, [repoName]: "Web mode: Operation not available" })
      }
    } catch (error) {
      setOperationStatus({ ...operationStatus, [repoName]: `Error: ${error}` })
    }
  }

  const pullUpdates = async (repoName: string) => {
    setOperationStatus({ ...operationStatus, [repoName]: "Pulling updates..." })
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const result: string = await invoke("pull_firmware_updates", { repoName })
        setOperationStatus({ ...operationStatus, [repoName]: result })
        await loadFirmwareRepos() // Refresh the list
      } else {
        console.log("Web mode: Pulling updates disabled")
        setOperationStatus({ ...operationStatus, [repoName]: "Web mode: Operation not available" })
      }
    } catch (error) {
      setOperationStatus({ ...operationStatus, [repoName]: `Error: ${error}` })
    }
  }

  const updateBoardFirmware = async (repoName: string, stackLevel: number) => {
    const key = `${repoName}_${stackLevel}`
    setOperationStatus({ ...operationStatus, [key]: "Updating firmware..." })
    setUpdateProgress({ ...updateProgress, [key]: 0 })

    // Simulate progress
    const progressInterval = setInterval(() => {
      setUpdateProgress((prev) => {
        const current = prev[key] || 0
        if (current < 90) {
          return { ...prev, [key]: current + 10 }
        }
        return prev
      })
    }, 500)

    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const result: string = await invoke("update_board_firmware", { repoName, stackLevel })
        clearInterval(progressInterval)
        setUpdateProgress({ ...updateProgress, [key]: 100 })
        setOperationStatus({ ...operationStatus, [key]: result })
      } else {
        console.log("Web mode: Firmware update disabled")
        clearInterval(progressInterval)
        setOperationStatus({ ...operationStatus, [key]: "Web mode: Operation not available" })
      }
    } catch (error) {
      clearInterval(progressInterval)
      setOperationStatus({ ...operationStatus, [key]: `Error: ${error}` })
    }
  }

  const getRepoIcon = (repo: FirmwareRepo) => {
    if (repo.is_cloned) {
      return <CheckCircle className="w-4 h-4 text-green-500" />
    } else {
      return <XCircle className="w-4 h-4 text-red-500" />
    }
  }

  const getBoardsForRepo = (repoName: string) => {
    return boards.filter((board) => {
      const boardRepoName = board.board_type.toLowerCase().replace(/[^a-z0-9]/g, "_")
      return boardRepoName.includes(repoName) || repoName.includes(boardRepoName.split("_")[0])
    })
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Download className="w-5 h-5" />
            Firmware Management System
            <Badge variant="outline" className="ml-auto">
              {repos.filter((r) => r.is_cloned).length}/{repos.length} Repositories Ready
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <p className="text-gray-600">
              Manage firmware repositories and update board firmware directly from the interface.
            </p>
            <Button onClick={loadFirmwareRepos} disabled={isLoading} variant="outline">
              <RefreshCw className={`w-4 h-4 mr-2 ${isLoading ? "animate-spin" : ""}`} />
              Refresh Status
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Repository Management */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <GitBranch className="w-5 h-5" />
            Repository Management
          </CardTitle>
        </CardHeader>
        <CardContent>
          <ScrollArea className="h-96">
            <div className="space-y-4">
              {repos.map((repo) => (
                <div key={repo.name} className="p-4 border rounded-lg">
                  <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-2">
                      {getRepoIcon(repo)}
                      <div>
                        <h4 className="font-medium">{repo.display_name}</h4>
                        <p className="text-sm text-gray-500">{repo.repo_url}</p>
                      </div>
                    </div>
                    <Badge variant={repo.is_cloned ? "default" : "secondary"}>
                      {repo.is_cloned ? "Cloned" : "Not Cloned"}
                    </Badge>
                  </div>

                  <div className="flex gap-2 mb-3">
                    {!repo.is_cloned ? (
                      <Button
                        onClick={() => cloneRepository(repo.name)}
                        disabled={operationStatus[repo.name]?.includes("...")}
                        size="sm"
                      >
                        <Download className="w-3 h-3 mr-1" />
                        Clone Repository
                      </Button>
                    ) : (
                      <>
                        <Button
                          onClick={() => pullUpdates(repo.name)}
                          disabled={operationStatus[repo.name]?.includes("...")}
                          size="sm"
                          variant="outline"
                        >
                          <RefreshCw className="w-3 h-3 mr-1" />
                          Pull Updates
                        </Button>
                        <Button
                          onClick={() => installDrivers(repo.name)}
                          disabled={operationStatus[repo.name]?.includes("...")}
                          size="sm"
                        >
                          <HardDrive className="w-3 h-3 mr-1" />
                          Install Drivers
                        </Button>
                      </>
                    )}
                  </div>

                  {operationStatus[repo.name] && (
                    <Alert className="mb-3">
                      <AlertTriangle className="h-4 w-4" />
                      <AlertDescription>{operationStatus[repo.name]}</AlertDescription>
                    </Alert>
                  )}

                  {/* Connected Boards */}
                  {repo.is_cloned && (
                    <div className="mt-3 pt-3 border-t">
                      <h5 className="text-sm font-medium mb-2">Connected Boards:</h5>
                      <div className="space-y-2">
                        {getBoardsForRepo(repo.name).map((board) => (
                          <div
                            key={`${board.board_type}_${board.stack_level}`}
                            className="flex items-center justify-between p-2 bg-gray-50 rounded"
                          >
                            <div className="flex items-center gap-2">
                              <Cpu className="w-3 h-3" />
                              <span className="text-sm">
                                {board.board_type} (Stack {board.stack_level})
                              </span>
                              <Badge variant="outline" className="text-xs">
                                v{board.firmware_version}
                              </Badge>
                            </div>
                            <div className="flex items-center gap-2">
                              {updateProgress[`${repo.name}_${board.stack_level}`] !== undefined && (
                                <Progress
                                  value={updateProgress[`${repo.name}_${board.stack_level}`]}
                                  className="w-20 h-2"
                                />
                              )}
                              <Button
                                onClick={() => updateBoardFirmware(repo.name, board.stack_level)}
                                disabled={operationStatus[`${repo.name}_${board.stack_level}`]?.includes("...")}
                                size="sm"
                                variant="outline"
                              >
                                <Wifi className="w-3 h-3 mr-1" />
                                Update
                              </Button>
                            </div>
                          </div>
                        ))}
                        {getBoardsForRepo(repo.name).length === 0 && (
                          <p className="text-sm text-gray-500">No compatible boards detected</p>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </ScrollArea>
        </CardContent>
      </Card>

      {/* Batch Operations */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <RefreshCw className="w-5 h-5" />
            Batch Operations
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Button
              onClick={() => {
                repos.filter((r) => !r.is_cloned).forEach((repo) => cloneRepository(repo.name))
              }}
              disabled={repos.every((r) => r.is_cloned)}
              className="w-full"
            >
              <Download className="w-4 h-4 mr-2" />
              Clone All Missing
            </Button>
            <Button
              onClick={() => {
                repos.filter((r) => r.is_cloned).forEach((repo) => pullUpdates(repo.name))
              }}
              disabled={repos.every((r) => !r.is_cloned)}
              variant="outline"
              className="w-full"
            >
              <RefreshCw className="w-4 h-4 mr-2" />
              Update All Repos
            </Button>
            <Button
              onClick={() => {
                repos.filter((r) => r.is_cloned).forEach((repo) => installDrivers(repo.name))
              }}
              disabled={repos.every((r) => !r.is_cloned)}
              variant="outline"
              className="w-full"
            >
              <HardDrive className="w-4 h-4 mr-2" />
              Install All Drivers
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Safety Notice */}
      <Alert>
        <AlertTriangle className="h-4 w-4" />
        <AlertDescription>
          <strong>Important:</strong> Firmware updates will temporarily interrupt board communication. Ensure critical
          systems have backup control before proceeding with updates.
        </AlertDescription>
      </Alert>
    </div>
  )
}
