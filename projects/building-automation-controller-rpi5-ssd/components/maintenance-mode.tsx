"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Wrench, AlertTriangle, Clock, User, FileText, XCircle } from "lucide-react"
// Dynamic import for Tauri - will be null in web mode

interface MaintenanceMode {
  enabled: boolean
  started_at?: string
  duration_minutes: number
  reason: string
  authorized_by: string
}

interface MaintenanceModeProps {
  onStatusChange?: (enabled: boolean) => void
}

export default function MaintenanceMode({ onStatusChange }: MaintenanceModeProps) {
  const [maintenanceStatus, setMaintenanceStatus] = useState<MaintenanceMode | null>(null)
  const [dialogOpen, setDialogOpen] = useState(false)
  const [reason, setReason] = useState("")
  const [authorizedBy, setAuthorizedBy] = useState("")
  const [duration, setDuration] = useState("120") // Default 2 hours
  const [timeRemaining, setTimeRemaining] = useState<string>("")
  const [loading, setLoading] = useState(false)

  // Fetch maintenance status
  const fetchStatus = async () => {
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        const status = await invoke<MaintenanceMode>("get_maintenance_status")
        setMaintenanceStatus(status)
        onStatusChange?.(status.enabled)
      } else {
        // Mock data for web mode
        const mockStatus: MaintenanceMode = {
          enabled: false,
          duration_minutes: 120,
          reason: "",
          authorized_by: ""
        };
        setMaintenanceStatus(mockStatus)
        onStatusChange?.(false)
      }
    } catch (error) {
      console.error("Failed to get maintenance status:", error)
    }
  }

  // Update time remaining
  const updateTimeRemaining = () => {
    if (maintenanceStatus?.enabled && maintenanceStatus.started_at) {
      const startTime = new Date(maintenanceStatus.started_at)
      const now = new Date()
      const elapsed = Math.floor((now.getTime() - startTime.getTime()) / 1000 / 60) // minutes
      const remaining = maintenanceStatus.duration_minutes - elapsed

      if (remaining > 0) {
        const hours = Math.floor(remaining / 60)
        const minutes = remaining % 60
        setTimeRemaining(`${hours}h ${minutes}m`)
      } else {
        setTimeRemaining("Expired")
        fetchStatus() // Refresh to get updated status
      }
    }
  }

  useEffect(() => {
    fetchStatus()
    const interval = setInterval(() => {
      fetchStatus()
      updateTimeRemaining()
    }, 10000) // Update every 10 seconds

    return () => clearInterval(interval)
  }, [])

  useEffect(() => {
    updateTimeRemaining()
    const interval = setInterval(updateTimeRemaining, 1000) // Update countdown every second
    return () => clearInterval(interval)
  }, [maintenanceStatus])

  const enableMaintenanceMode = async () => {
    if (!reason.trim() || !authorizedBy.trim()) {
      alert("Please provide a reason and your name")
      return
    }

    setLoading(true)
    try {
      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        const { invoke } = await import("@tauri-apps/api/tauri");
        await invoke("enable_maintenance_mode", {
          reason: reason.trim(),
          authorizedBy: authorizedBy.trim(),
          durationMinutes: parseInt(duration)
        })
      } else {
        console.log("Web mode: Maintenance mode enable simulation")
      }
      
      await fetchStatus()
      setDialogOpen(false)
      setReason("")
      setAuthorizedBy("")
    } catch (error) {
      alert(`Failed to enable maintenance mode: ${error}`)
    } finally {
      setLoading(false)
    }
  }

  const disableMaintenanceMode = async () => {
    if (confirm("Are you sure you want to exit maintenance mode? Automatic control will resume.")) {
      setLoading(true)
      try {
        if (typeof window !== 'undefined' && (window as any).__TAURI__) {
          const { invoke } = await import("@tauri-apps/api/tauri");
          await invoke("disable_maintenance_mode")
        } else {
          console.log("Web mode: Maintenance mode disable simulation")
        }
        await fetchStatus()
      } catch (error) {
        alert(`Failed to disable maintenance mode: ${error}`)
      } finally {
        setLoading(false)
      }
    }
  }

  return (
    <>
      <Card className={maintenanceStatus?.enabled ? "border-orange-500" : ""}>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Wrench className="w-5 h-5" />
              Maintenance Mode
            </div>
            {maintenanceStatus?.enabled && (
              <Badge variant="destructive" className="animate-pulse">
                ACTIVE
              </Badge>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {maintenanceStatus?.enabled ? (
            <div className="space-y-4">
              <Alert className="border-orange-500 bg-orange-50">
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>
                  <strong>Maintenance mode is active!</strong> All automatic control logic and BMS commands are disabled. 
                  Manual control only.
                </AlertDescription>
              </Alert>

              <div className="grid grid-cols-2 gap-4 text-sm">
                <div className="space-y-1">
                  <div className="flex items-center gap-2 text-gray-600">
                    <Clock className="w-4 h-4" />
                    Time Remaining
                  </div>
                  <div className="text-xl font-bold text-orange-600">
                    {timeRemaining}
                  </div>
                </div>

                <div className="space-y-1">
                  <div className="flex items-center gap-2 text-gray-600">
                    <User className="w-4 h-4" />
                    Authorized By
                  </div>
                  <div className="font-medium">
                    {maintenanceStatus.authorized_by}
                  </div>
                </div>

                <div className="col-span-2 space-y-1">
                  <div className="flex items-center gap-2 text-gray-600">
                    <FileText className="w-4 h-4" />
                    Reason
                  </div>
                  <div className="font-medium">
                    {maintenanceStatus.reason}
                  </div>
                </div>
              </div>

              <div className="flex gap-2">
                <Button 
                  variant="destructive" 
                  onClick={disableMaintenanceMode}
                  disabled={loading}
                  className="flex-1"
                >
                  <XCircle className="w-4 h-4 mr-2" />
                  Exit Maintenance Mode
                </Button>
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              <p className="text-sm text-gray-600">
                Enable maintenance mode to temporarily disable all automatic control logic and BMS commands. 
                This allows manual control of outputs for testing and maintenance.
              </p>
              
              <Alert>
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>
                  Maintenance mode automatically expires after the specified duration (max 2 hours) for safety.
                </AlertDescription>
              </Alert>

              <Button 
                onClick={() => setDialogOpen(true)}
                className="w-full"
                variant="outline"
              >
                <Wrench className="w-4 h-4 mr-2" />
                Enable Maintenance Mode
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Enable Maintenance Mode</DialogTitle>
          </DialogHeader>
          
          <div className="space-y-4">
            <Alert className="border-orange-500 bg-orange-50">
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>
                This will disable all automatic control logic and BMS commands. 
                Only manual control will be available.
              </AlertDescription>
            </Alert>

            <div className="space-y-2">
              <Label htmlFor="authorized-by">Your Name *</Label>
              <Input
                id="authorized-by"
                value={authorizedBy}
                onChange={(e) => setAuthorizedBy(e.target.value)}
                placeholder="Enter your name"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="reason">Reason for Maintenance *</Label>
              <Input
                id="reason"
                value={reason}
                onChange={(e) => setReason(e.target.value)}
                placeholder="e.g., Testing new sensor calibration"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="duration">Duration (minutes)</Label>
              <select
                id="duration"
                value={duration}
                onChange={(e) => setDuration(e.target.value)}
                className="w-full rounded-md border border-input bg-background px-3 py-2"
              >
                <option value="30">30 minutes</option>
                <option value="60">1 hour</option>
                <option value="90">1.5 hours</option>
                <option value="120">2 hours (max)</option>
              </select>
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setDialogOpen(false)}>
              Cancel
            </Button>
            <Button 
              variant="destructive" 
              onClick={enableMaintenanceMode}
              disabled={loading || !reason.trim() || !authorizedBy.trim()}
            >
              Enable Maintenance Mode
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}