# App Spawner PowerShell Script
param(
    [string]$ProgramPath = (Read-Host "Enter program path to run (e.g., notepad.exe)"),
    [int]$Count = (Read-Host "Number of programs to spawn"),
    [int]$MaxHorizontalStack = (Read-Host "Maximum horizontal stack"),
    [int]$MaxVerticalStack = (Read-Host "Maximum vertical stack"),
    [int]$Spacing = (Read-Host "Spacing between windows (negative for overlap)")
)

# Add required .NET assemblies for Windows Forms
Add-Type -AssemblyName System.Windows.Forms

# Function to get cursor position
function Get-CursorPosition {
    $cursor = [System.Windows.Forms.Cursor]::Position
    return @{ X = $cursor.X; Y = $cursor.Y }
}

# Function to position a window
function Set-WindowPosition {
    param(
        [System.Diagnostics.Process]$Process,
        [int]$X,
        [int]$Y,
        [int]$Width,
        [int]$Height
    )
    
    # Wait for window to appear
    $timeout = 100
    while (-not $Process.MainWindowHandle -and $timeout -gt 0) {
        Start-Sleep -Milliseconds 100
        $Process.Refresh()
        $timeout--
    }
    
    if ($Process.MainWindowHandle -eq 0) {
        Write-Warning "Failed to get window handle for process $($Process.Id)"
        return
    }
    
    # Load required Win32 API functions
    $signature = @'
[DllImport("user32.dll")]
public static extern bool MoveWindow(IntPtr hWnd, int X, int Y, int nWidth, int nHeight, bool bRepaint);
'@
    
    $win32 = Add-Type -MemberDefinition $signature -Name "Win32MoveWindow" -Namespace Win32Functions -PassThru
    
    # Position the window
    [void]$win32::MoveWindow($Process.MainWindowHandle, $X, $Y, $Width, $Height, $true)
}

# Get starting position
Write-Host "Move your mouse to the desired starting position and press Enter..."
Read-Host "Press Enter when ready"
$StartPosition = Get-CursorPosition
Write-Host "Starting position: X=$($StartPosition.X), Y=$($StartPosition.Y)"

# Default window dimensions
$WindowWidth = 308
$WindowHeight = 265

# Calculate positions
$Positions = @()
$CurrentX = $StartPosition.X
$CurrentY = $StartPosition.Y
$HCount = 0
$VCount = 0

for ($i = 0; $i -lt $Count; $i++) {
    $Positions += @{ X = $CurrentX; Y = $CurrentY }
    
    $HCount++
    if ($HCount -ge $MaxHorizontalStack) {
        # Move to next row
        $HCount = 0
        $VCount++
        $CurrentX = $StartPosition.X
        $CurrentY += $WindowHeight - $Spacing
        
        if ($VCount -ge $MaxVerticalStack) {
            Write-Host "Reached maximum vertical stack limit at $($i+1) windows"
            break
        }
    } else {
        # Move horizontally
        $CurrentX += $WindowWidth - $Spacing
    }
}

# Spawn and position applications
$Processes = @()
for ($i = 0; $i -lt $Positions.Count; $i++) {
    $pos = $Positions[$i]
    Write-Host "Spawning window $($i+1) at ($($pos.X), $($pos.Y))"
    
    try {
        # Start the process
        $process = Start-Process -FilePath $ProgramPath -PassThru
        
        # Hide non-first windows initially
        if ($i -gt 0) {
            # Note: PowerShell has limitations in hiding windows of external processes
        }
        
        $Processes += @{ Process = $process; Position = $pos }
        Start-Sleep -Milliseconds 500  # Allow window to initialize
    } catch {
        Write-Warning "Failed to spawn window $($i+1): $_"
        break
    }
}

# Position all windows
Write-Host "`nPositioning $($Processes.Count) windows..."
for ($i = 0; $i -lt $Processes.Count; $i++) {
    $item = $Processes[$i]
    Write-Host "Positioning window $($i+1) at ($($item.Position.X), $($item.Position.Y))"
    Set-WindowPosition -Process $item.Process -X $item.Position.X -Y $item.Position.Y -Width $WindowWidth -Height $WindowHeight
    Start-Sleep -Milliseconds 100
}

Write-Host "`nOperation completed! Spawned $($Processes.Count) windows."
Write-Host "Close the windows manually when finished."