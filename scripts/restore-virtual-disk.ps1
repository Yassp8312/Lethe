param(
    [Parameter(Mandatory = $true)]
    [string]$ImagePath,

    [Parameter(Mandatory = $true)]
    [UInt64]$ExpectedLength,

    [Parameter(Mandatory = $true)]
    [UInt64]$ExpectedWriteTime,

    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[A-Z0-9-]{1,32}$')]
    [string]$Label
)

$ErrorActionPreference = 'Stop'
$mounted = $false
$partition = $null
$mountPath = $null

try {
    $item = Get-Item -LiteralPath $ImagePath -Force
    if (-not $item.PSIsContainer -and
        [UInt64]$item.Length -eq $ExpectedLength -and
        [UInt64]$item.LastWriteTimeUtc.ToFileTimeUtc() -eq $ExpectedWriteTime) {
        # Identity matches the plan immediately before mounting.
    } else {
        throw 'RESTORE_IDENTITY_CHANGED'
    }

    $extension = [System.IO.Path]::GetExtension($item.FullName)
    if ($extension -notin @('.vhd', '.vhdx')) {
        throw 'RESTORE_TARGET_NOT_VIRTUAL_IMAGE'
    }

    $image = Mount-DiskImage -ImagePath $item.FullName -NoDriveLetter -PassThru
    $mounted = $true
    $disks = @($image | Get-Disk)
    if ($disks.Count -ne 1) {
        throw 'RESTORE_AMBIGUOUS_VIRTUAL_DISK'
    }
    $disk = $disks[0]
    $bus = [string]$disk.BusType
    if ($bus -notin @('File Backed Virtual', 'Virtual')) {
        throw 'RESTORE_NON_VIRTUAL_BUS_REJECTED'
    }
    if ($disk.IsBoot -or $disk.IsSystem) {
        throw 'RESTORE_SYSTEM_DISK_REJECTED'
    }
    if ($disk.IsOffline) {
        $disk | Set-Disk -IsOffline $false
        $disk = $image | Get-Disk
    }
    if ($disk.IsReadOnly) {
        $disk | Set-Disk -IsReadOnly $false
        $disk = $image | Get-Disk
    }

    $disk | Clear-Disk -RemoveData -RemoveOEM -Confirm:$false
    $initialized = $disk | Initialize-Disk -PartitionStyle GPT -PassThru
    $partition = $initialized | New-Partition -UseMaximumSize
    $formatted = $partition | Format-Volume -FileSystem exFAT -NewFileSystemLabel $Label -Confirm:$false -Force
    $mountPath = Join-Path ([System.IO.Path]::GetTempPath()) ("LetheRestoreMount-" + $PID)
    [void](New-Item -ItemType Directory -Path $mountPath)
    $partition | Add-PartitionAccessPath -AccessPath $mountPath
    $volume = Get-Volume -FilePath $mountPath
    if ($volume.FileSystemLabel -ne $Label) {
        $volume | Set-Volume -NewFileSystemLabel $Label
        $volume = Get-Volume -FilePath $mountPath
    }
    if ($volume.FileSystem -ne 'exFAT' -or $volume.FileSystemLabel -ne $Label) {
        throw 'RESTORE_RESULT_VERIFICATION_FAILED'
    }

    Write-Output ('RESTORE_OK=1')
    Write-Output ('DISK_NUMBER=' + $disk.Number)
    Write-Output ('DRIVE_LETTER=' + [string]$formatted.DriveLetter)
    Write-Output ('FILESYSTEM=' + $volume.FileSystem)
    Write-Output ('LABEL=' + $volume.FileSystemLabel)
} finally {
    if ($null -ne $partition -and $null -ne $mountPath) {
        $partition | Remove-PartitionAccessPath -AccessPath $mountPath -ErrorAction SilentlyContinue
    }
    if ($null -ne $mountPath -and (Test-Path -LiteralPath $mountPath)) {
        Remove-Item -LiteralPath $mountPath -Force -ErrorAction SilentlyContinue
    }
    if ($mounted) {
        Dismount-DiskImage -ImagePath $ImagePath -ErrorAction SilentlyContinue | Out-Null
    }
}
